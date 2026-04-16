# micvol

Hardware-level microphone volume control for macOS.

## Why?

When you record audio with the mic volume at 30%, the A/D converter only uses 30% of its dynamic range. Amplifying this signal in software (AGC, digital gain) amplifies the noise floor too.

By setting the **hardware** volume to 100% *before* recording starts, you capture the full dynamic range at the source. `micvol` makes this a one-liner with RAII:

```rust
let device = micvol::default_input_device()?;
{
    let _guard = micvol::VolumeGuard::maximize(&device.id)?;
    // ... record audio here — hardware gain is maximized ...
}
// guard dropped: original volume automatically restored
```

If your program panics inside the guard, `Drop` still runs and restores the volume.

## Features

- **VolumeGuard** — RAII backup/maximize/restore in one line
- **Device enumeration** — list all audio input devices with name, channels, volume
- **Volume control** — get/set hardware volume scalar (0.0–1.0)
- **Mute control** — get/set mute state
- Built on CoreAudio HAL (`AudioObjectGetPropertyData` / `AudioObjectSetPropertyData`)

## Install

```toml
[dependencies]
micvol = "0.1"
```

## Examples

List all input devices:

```sh
cargo run --example list_devices
```

```
Audio input devices:
  MacBook Pro Microphone [1ch] volume=100% mute=Unmuted
  AirPods Pro [1ch] volume=73% mute=Unmuted (default)
```

Demonstrate VolumeGuard:

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
// Device enumeration
micvol::input_devices() -> Result<Vec<DeviceInfo>>
micvol::default_input_device() -> Result<DeviceInfo>

// Volume control
micvol::get_volume(&DeviceId) -> Result<Volume>
micvol::set_volume(&DeviceId, Volume) -> Result<()>

// Mute control
micvol::get_mute(&DeviceId) -> Result<MuteState>
micvol::set_mute(&DeviceId, MuteState) -> Result<()>

// RAII guard
micvol::VolumeGuard::maximize(&DeviceId) -> Result<VolumeGuard>
micvol::VolumeGuard::with_volume(&DeviceId, Volume) -> Result<VolumeGuard>
```

## License

MIT
