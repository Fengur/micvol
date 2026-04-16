# micvol

macOS 麦克风硬件**输入音量**控制库。

[English](README.md)

## 什么是"输入音量"？

这**不是**你按键盘音量键调节的那个输出音量，而是麦克风 A/D 转换器的增益等级——在 **系统设置 > 声音 > 输入 > 输入音量** 里的那个滑块。

```
系统设置 > 声音 > 输入
┌─────────────────────────────────────┐
│ 名称                 类型           │
│ MacBook Pro麦克风    内建           │
│ AirPods Pro         蓝牙      ◄──  │
│                                     │
│ 输入音量  ●━━━━━━━━━━━━━━━━━━━━●   │ ← 就是这个滑块
│ 输入电平  ▮▮▮▮▮▮▯▯▯▯▯▯▯▯▯▯       │
└─────────────────────────────────────┘
```

`micvol` 通过 CoreAudio HAL 以编程方式控制这个滑块。

## 为什么需要这个？

当输入音量设为 30% 时，A/D 转换器只使用了 30% 的动态范围。之后在软件层做放大（AGC、数字增益）会连噪声一起放大——已经丢失的信息无法恢复。

在录音**开始前**将硬件输入音量拉满至 100%，就能在源头捕获最大动态范围。`micvol` 用 RAII 一行代码搞定：

```rust
let device = micvol::default_input_device()?;
{
    let _guard = micvol::VolumeGuard::maximize(&device.id)?;
    // ... 在此录音，输入增益已最大化 ...
}
// guard 离开作用域，输入音量自动恢复
```

即使程序在 guard 内部 panic，`Drop` 仍会执行并恢复音量。

## 特性

- **VolumeGuard** — RAII 一行代码完成输入音量的备份/拉满/恢复
- **设备枚举** — 列出所有音频输入设备及其名称、通道数、音量
- **输入音量控制** — 读写硬件输入音量标量 (0.0-1.0)
- **静音控制** — 读写输入静音状态
- 基于 CoreAudio HAL (`AudioObjectGetPropertyData` / `AudioObjectSetPropertyData`)

## 安装

```toml
[dependencies]
micvol = "0.1"
```

## 示例

列出所有输入设备：

```sh
cargo run --example list_devices
```

```
Audio input devices:
  MacBook Pro麦克风 [1ch] volume=100% mute=Unmuted
  AirPods Pro [1ch] volume=73% mute=Unmuted (default)
```

演示 VolumeGuard：

```sh
cargo run --example normalize
```

```
Device: AirPods Pro
Original volume: 73%
Volume during guard: 100%
Recording for 3 seconds...
Guard dropping, restoring volume...
Volume after restore: 73%
```

## API

```rust
// 设备枚举
micvol::input_devices() -> Result<Vec<DeviceInfo>>
micvol::default_input_device() -> Result<DeviceInfo>

// 输入音量控制
micvol::get_volume(&DeviceId) -> Result<Volume>
micvol::set_volume(&DeviceId, Volume) -> Result<()>

// 静音控制
micvol::get_mute(&DeviceId) -> Result<MuteState>
micvol::set_mute(&DeviceId, MuteState) -> Result<()>

// RAII 守卫
micvol::VolumeGuard::maximize(&DeviceId) -> Result<VolumeGuard>
micvol::VolumeGuard::with_volume(&DeviceId, Volume) -> Result<VolumeGuard>
```

## 许可证

MIT
