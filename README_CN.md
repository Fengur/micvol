# micvol

macOS 麦克风**输入音量**控制库。

[English](README.md)

![micvol 演示](demo.gif)

## 什么是"输入音量"？

这**不是**键盘音量键调节的输出音量，而是 **系统设置 > 声音 > 输入** 里的输入音量滑块。

`micvol` 让你用代码控制这个滑块。

## 特性

- **VolumeGuard** — 拉满输入音量，离开作用域自动恢复
- **设备枚举** — 列出所有音频输入设备
- **输入音量控制** — 读写音量 (0.0-1.0)
- **静音控制** — 读写静音状态

## 安装

**Rust：**

```toml
[dependencies]
micvol = "0.1"
```

**Swift / ObjC（静态库）：**

```bash
./scripts/build_release.sh
# 输出 dist/libmicvol.a + dist/micvol.h
```

**CocoaPods：**

```ruby
pod 'Micvol', :git => 'https://github.com/Fengur/micvol.git'
```

## 用法

### Rust

```rust
let device = micvol::default_input_device()?;
{
    let _guard = micvol::VolumeGuard::maximize(&device.id)?;
    // ... 录音，输入音量已拉满 ...
}
// guard 离开作用域，输入音量自动恢复
```

### C / Swift / ObjC

```c
#include "micvol.h"

uint32_t device_id;
char *name;
micvol_default_input_device(&device_id, &name);

MicvolGuard guard;
micvol_guard_maximize(device_id, &guard);
// ... 录音 ...
micvol_guard_restore(guard);

micvol_free_string(name);
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

// VolumeGuard
micvol::VolumeGuard::maximize(&DeviceId) -> Result<VolumeGuard>
micvol::VolumeGuard::with_volume(&DeviceId, Volume) -> Result<VolumeGuard>
```

## Demo App

仓库内含 SwiftUI 演示应用，位于 `demo/MicvolDemo/`。构建方式：

```bash
./scripts/build_release.sh
cd demo/MicvolDemo
xcodegen generate
open MicvolDemo.xcodeproj
```

## 技术细节

如需了解实现原理和技术文档，请联系作者。

## 许可证

MIT
