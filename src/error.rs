/// micvol 的错误类型。
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// CoreAudio API 返回的 OSStatus 错误码。
    #[error("CoreAudio error (OSStatus {0})")]
    CoreAudio(i32),

    /// 找不到指定的设备。
    #[error("device not found")]
    DeviceNotFound,

    /// 系统没有默认输入设备。
    #[error("no default input device")]
    NoDefaultInputDevice,

    /// 音量值超出 [0.0, 1.0] 范围。
    #[error("volume {0} out of range [0.0, 1.0]")]
    VolumeOutOfRange(f32),

    /// 设备不支持此操作（例如某些 USB 麦克风无硬件音量控制）。
    #[error("not supported: {0}")]
    NotSupported(String),
}

pub type Result<T> = std::result::Result<T, Error>;
