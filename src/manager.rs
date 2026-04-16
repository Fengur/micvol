//! 设备管理：枚举、查询、音量控制。

use crate::coreaudio;
use crate::error::Result;
use crate::types::*;

/// 枚举系统中所有音频输入设备。
pub fn input_devices() -> Result<Vec<DeviceInfo>> {
    let default_id = coreaudio::default_input_device_id().ok();
    let all_ids = coreaudio::all_device_ids()?;

    let mut devices = Vec::new();
    for id in all_ids {
        let channels = match coreaudio::input_channel_count(id) {
            Ok(ch) if ch > 0 => ch,
            _ => continue, // 不是输入设备，跳过
        };

        let name = coreaudio::device_name(id).unwrap_or_else(|_| "Unknown".into());
        let is_default = default_id == Some(id);

        devices.push(DeviceInfo {
            id: DeviceId(id),
            name,
            channels,
            is_default,
        });
    }

    Ok(devices)
}

/// 获取系统默认输入设备。
pub fn default_input_device() -> Result<DeviceInfo> {
    let id = coreaudio::default_input_device_id()?;
    let name = coreaudio::device_name(id).unwrap_or_else(|_| "Unknown".into());
    let channels = coreaudio::input_channel_count(id).unwrap_or(0);

    Ok(DeviceInfo {
        id: DeviceId(id),
        name,
        channels,
        is_default: true,
    })
}

/// 获取设备当前音量。
pub fn get_volume(device: &DeviceId) -> Result<Volume> {
    let raw = coreaudio::get_volume_scalar(device.0)?;
    Volume::new(raw.clamp(0.0, 1.0))
}

/// 设置设备音量。
pub fn set_volume(device: &DeviceId, volume: Volume) -> Result<()> {
    coreaudio::set_volume_scalar(device.0, volume.as_f32())
}

/// 获取设备静音状态。
pub fn get_mute(device: &DeviceId) -> Result<MuteState> {
    let muted = coreaudio::get_mute(device.0)?;
    Ok(if muted {
        MuteState::Muted
    } else {
        MuteState::Unmuted
    })
}

/// 设置设备静音状态。
pub fn set_mute(device: &DeviceId, state: MuteState) -> Result<()> {
    coreaudio::set_mute(device.0, state == MuteState::Muted)
}
