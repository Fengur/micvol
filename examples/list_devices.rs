/// 枚举系统中所有音频输入设备。
fn main() {
    let devices = micvol::input_devices().expect("failed to enumerate devices");

    if devices.is_empty() {
        println!("No input devices found.");
        return;
    }

    println!("Audio input devices:");
    for dev in &devices {
        let volume = micvol::get_volume(&dev.id)
            .map(|v| format!("{}", v))
            .unwrap_or_else(|_| "N/A".into());
        let mute = micvol::get_mute(&dev.id)
            .map(|m| format!("{:?}", m))
            .unwrap_or_else(|_| "N/A".into());

        println!(
            "  {} [{}ch] volume={} mute={}{}",
            dev.name,
            dev.channels,
            volume,
            mute,
            if dev.is_default { " (default)" } else { "" },
        );
    }
}
