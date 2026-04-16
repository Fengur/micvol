//! # micvol
//!
//! macOS 麦克风硬件音量控制库。
//!
//! 通过 CoreAudio HAL 直接操作硬件音量，在 A/D 转换阶段获得最优信噪比。
//! 核心功能是 [`VolumeGuard`]：创建时将麦克风音量拉满，离开作用域自动恢复。
//!
//! ## 快速上手
//!
//! ```no_run
//! let device = micvol::default_input_device().unwrap();
//!
//! {
//!     let _guard = micvol::VolumeGuard::maximize(&device.id).unwrap();
//!     // ... 在此录音，硬件增益已最大化 ...
//! }
//! // 音量已自动恢复
//! ```

mod coreaudio;
pub mod error;
mod ffi;
mod guard;
mod manager;
pub mod types;

pub use error::Error;
pub use guard::VolumeGuard;
pub use types::*;

/// 枚举系统中所有音频输入设备。
pub fn input_devices() -> error::Result<Vec<DeviceInfo>> {
    manager::input_devices()
}

/// 获取系统默认输入设备。
pub fn default_input_device() -> error::Result<DeviceInfo> {
    manager::default_input_device()
}

/// 获取设备当前音量。
pub fn get_volume(device: &DeviceId) -> error::Result<Volume> {
    manager::get_volume(device)
}

/// 设置设备音量。
pub fn set_volume(device: &DeviceId, volume: Volume) -> error::Result<()> {
    manager::set_volume(device, volume)
}

/// 获取设备静音状态。
pub fn get_mute(device: &DeviceId) -> error::Result<MuteState> {
    manager::get_mute(device)
}

/// 设置设备静音状态。
pub fn set_mute(device: &DeviceId, state: MuteState) -> error::Result<()> {
    manager::set_mute(device, state)
}
