# micvol

macOS microphone **input volume** control library.

[中文文档](README_CN.md)

![micvol demo](demo.gif)

## What is "input volume"?

This is **not** the output volume you adjust with keyboard volume keys. This is the **input volume** slider in **System Settings > Sound > Input**.

`micvol` lets you programmatically control this slider.

## Features

- **VolumeGuard** — maximize input volume, automatically restore on drop
- **Device enumeration** — list all audio input devices
- **Input volume control** — get/set volume (0.0–1.0)
- **Mute control** — get/set mute state

## Install

**Rust:**

```toml
[dependencies]
micvol = "0.1"
```

**Swift / ObjC (static library):**

```bash
./scripts/build_release.sh
# outputs dist/libmicvol.a + dist/micvol.h
```

**CocoaPods:**

```ruby
pod 'Micvol', :git => 'https://github.com/Fengur/micvol.git'
```

## Usage

### Rust

```rust
let device = micvol::default_input_device()?;
{
    let _guard = micvol::VolumeGuard::maximize(&device.id)?;
    // ... record audio — input volume is maximized ...
}
// guard dropped: input volume automatically restored
```

### C / Swift / ObjC

```c
#include "micvol.h"

uint32_t device_id;
char *name;
micvol_default_input_device(&device_id, &name);

MicvolGuard guard;
micvol_guard_maximize(device_id, &guard);
// ... record audio ...
micvol_guard_restore(guard);

micvol_free_string(name);
```

## API

```rust
// Device enumeration
micvol::input_devices() -> Result<Vec<DeviceInfo>>
micvol::default_input_device() -> Result<DeviceInfo>

// Input volume control
micvol::get_volume(&DeviceId) -> Result<Volume>
micvol::set_volume(&DeviceId, Volume) -> Result<()>

// Mute control
micvol::get_mute(&DeviceId) -> Result<MuteState>
micvol::set_mute(&DeviceId, MuteState) -> Result<()>

// VolumeGuard
micvol::VolumeGuard::maximize(&DeviceId) -> Result<VolumeGuard>
micvol::VolumeGuard::with_volume(&DeviceId, Volume) -> Result<VolumeGuard>
```

## Demo App

A SwiftUI demo app is included in `demo/MicvolDemo/`. To build:

```bash
./scripts/build_release.sh
cd demo/MicvolDemo
xcodegen generate
open MicvolDemo.xcodeproj
```

## License

MIT
