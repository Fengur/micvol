/// 演示 VolumeGuard：将麦克风音量拉满，3 秒后自动恢复。
fn main() {
    env_logger::init();

    let device = micvol::default_input_device().expect("no default input device");
    let before = micvol::get_volume(&device.id).expect("cannot read volume");
    println!("Device: {}", device.name);
    println!("Original volume: {}", before);

    {
        let _guard =
            micvol::VolumeGuard::maximize(&device.id).expect("failed to maximize volume");
        let during = micvol::get_volume(&device.id).expect("cannot read volume");
        println!("Volume during guard: {}", during);

        println!("Recording for 3 seconds...");
        std::thread::sleep(std::time::Duration::from_secs(3));

        println!("Guard dropping, restoring volume...");
    }

    let after = micvol::get_volume(&device.id).expect("cannot read volume");
    println!("Volume after restore: {}", after);
}
