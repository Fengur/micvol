//! VolumeGuard — RAII 音量归一化。
//!
//! 对应原始 InputDeviceSetting.mm 的 backup → set → restore 模式，
//! 用 Rust 的 Drop trait 保证即使 panic 也能恢复音量。

use crate::error::Result;
use crate::manager;
use crate::types::*;

/// RAII 音量归一化守卫。
///
/// 创建时备份当前音量并设置为目标值，离开作用域时自动恢复。
///
/// # 用法
///
/// ```no_run
/// let device = micvol::default_input_device().unwrap();
/// {
///     let _guard = micvol::VolumeGuard::maximize(&device.id).unwrap();
///     // ... 录音，硬件增益已最大化 ...
/// }
/// // 音量已自动恢复
/// ```
pub struct VolumeGuard {
    device_id: DeviceId,
    original_volume: Volume,
    original_mute: MuteState,
    restored: bool,
}

impl VolumeGuard {
    /// 将设备音量拉满至 1.0。最常用的场景。
    pub fn maximize(device: &DeviceId) -> Result<Self> {
        Self::with_volume(device, Volume::max())
    }

    /// 将设备音量设置为指定值。
    pub fn with_volume(device: &DeviceId, target: Volume) -> Result<Self> {
        let original_volume = manager::get_volume(device)?;
        let original_mute = manager::get_mute(device)?;

        manager::set_volume(device, target)?;
        if target.as_f32() > 0.0 {
            manager::set_mute(device, MuteState::Unmuted)?;
        }

        log::info!(
            "micvol: volume {} -> {}, mute {:?} -> Unmuted (device {:?})",
            original_volume,
            target,
            original_mute,
            device,
        );

        Ok(Self {
            device_id: *device,
            original_volume,
            original_mute,
            restored: false,
        })
    }

    /// 手动提前恢复，不等 Drop。可检查恢复是否成功。
    pub fn restore(mut self) -> Result<()> {
        self.do_restore()
    }

    /// 获取备份的原始音量。
    pub fn original_volume(&self) -> Volume {
        self.original_volume
    }

    fn do_restore(&mut self) -> Result<()> {
        if self.restored {
            return Ok(());
        }
        self.restored = true;

        let vol_result = manager::set_volume(&self.device_id, self.original_volume);
        let mute_result = manager::set_mute(&self.device_id, self.original_mute);

        log::info!(
            "micvol: restored volume to {}, mute to {:?} (device {:?})",
            self.original_volume,
            self.original_mute,
            self.device_id,
        );

        vol_result?;
        mute_result?;
        Ok(())
    }
}

impl Drop for VolumeGuard {
    fn drop(&mut self) {
        if let Err(e) = self.do_restore() {
            log::error!(
                "micvol: failed to restore volume for device {:?}: {}",
                self.device_id,
                e,
            );
        }
    }
}
