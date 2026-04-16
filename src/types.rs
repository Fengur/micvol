use crate::error::{Error, Result};
use coreaudio_sys::AudioDeviceID;
use std::fmt;

/// 音频设备标识符，封装 CoreAudio 的 AudioDeviceID。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceId(pub(crate) AudioDeviceID);

impl DeviceId {
    pub fn raw(&self) -> AudioDeviceID {
        self.0
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AudioDevice({})", self.0)
    }
}

/// 音量标量，范围 [0.0, 1.0]。
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Volume(f32);

impl Volume {
    pub fn new(value: f32) -> Result<Self> {
        if !(0.0..=1.0).contains(&value) {
            return Err(Error::VolumeOutOfRange(value));
        }
        Ok(Self(value))
    }

    pub fn max() -> Self {
        Self(1.0)
    }

    pub fn muted() -> Self {
        Self(0.0)
    }

    pub fn as_f32(&self) -> f32 {
        self.0
    }
}

impl fmt::Display for Volume {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.0}%", self.0 * 100.0)
    }
}

/// 静音状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MuteState {
    Muted,
    Unmuted,
}

/// 音频输入设备信息。
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: DeviceId,
    pub name: String,
    pub channels: u32,
    pub is_default: bool,
}

impl fmt::Display for DeviceInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}, {}ch{})",
            self.name,
            self.id,
            self.channels,
            if self.is_default { ", default" } else { "" }
        )
    }
}
