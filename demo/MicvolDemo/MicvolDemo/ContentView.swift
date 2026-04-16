import SwiftUI

struct ContentView: View {
    @State private var devices: [MicvolBridge.InputDevice] = []
    @State private var selectedDeviceId: UInt32 = 0
    @State private var currentVolume: Float = 0
    @State private var isMuted: Bool = false
    @State private var guardHandle: OpaquePointer? = nil
    @State private var logs: [String] = []

    private var isGuardActive: Bool { guardHandle != nil }

    var body: some View {
        VStack(spacing: 0) {
            // 标题
            HStack {
                Text("micvol Demo")
                    .font(.title.bold())
                Spacer()
                Text("Input Volume Control")
                    .foregroundColor(.secondary)
            }
            .padding()

            Divider()

            // 设备列表
            VStack(alignment: .leading, spacing: 8) {
                Text("Input Devices")
                    .font(.headline)

                ForEach(devices) { device in
                    HStack {
                        Image(systemName: device.isDefault ? "mic.fill" : "mic")
                            .foregroundColor(device.isDefault ? .blue : .secondary)
                        Text(device.name)
                            .fontWeight(device.id == selectedDeviceId ? .bold : .regular)
                        Spacer()
                        Text("\(device.channels)ch")
                            .foregroundColor(.secondary)
                            .font(.caption)
                        if device.isDefault {
                            Text("Default")
                                .font(.caption)
                                .padding(.horizontal, 6)
                                .padding(.vertical, 2)
                                .background(Color.blue.opacity(0.2))
                                .cornerRadius(4)
                        }
                    }
                    .padding(.vertical, 4)
                    .padding(.horizontal, 8)
                    .background(device.id == selectedDeviceId ? Color.blue.opacity(0.1) : Color.clear)
                    .cornerRadius(6)
                    .onTapGesture {
                        selectedDeviceId = device.id
                        refreshVolume()
                    }
                }
            }
            .padding()

            Divider()

            // 音量控制
            VStack(spacing: 12) {
                HStack {
                    Text("Input Volume")
                        .font(.headline)
                    Spacer()
                    Text(String(format: "%.0f%%", currentVolume * 100))
                        .font(.title2.monospacedDigit())
                        .foregroundColor(isGuardActive ? .green : .primary)
                }

                HStack {
                    Image(systemName: "mic")
                    Slider(value: Binding(
                        get: { currentVolume },
                        set: { newValue in
                            currentVolume = newValue
                            do {
                                try MicvolBridge.setVolume(deviceId: selectedDeviceId, volume: newValue)
                                addLog("Set volume to \(String(format: "%.0f%%", newValue * 100))")
                            } catch {
                                addLog("Error: \(error.localizedDescription)")
                            }
                        }
                    ), in: 0...1)
                    .disabled(isGuardActive)
                    Image(systemName: "mic.fill")
                }

                HStack(spacing: 16) {
                    // Maximize 按钮
                    Button(action: toggleGuard) {
                        HStack {
                            Image(systemName: isGuardActive ? "arrow.uturn.backward" : "arrow.up.to.line")
                            Text(isGuardActive ? "Restore" : "Maximize")
                        }
                        .frame(maxWidth: .infinity)
                    }
                    .controlSize(.large)
                    .buttonStyle(.borderedProminent)
                    .tint(isGuardActive ? .orange : .blue)

                    // Mute 按钮
                    Button(action: toggleMute) {
                        HStack {
                            Image(systemName: isMuted ? "mic.slash.fill" : "mic.fill")
                            Text(isMuted ? "Unmute" : "Mute")
                        }
                        .frame(maxWidth: .infinity)
                    }
                    .controlSize(.large)
                    .buttonStyle(.bordered)
                    .disabled(isGuardActive)
                }
            }
            .padding()

            Divider()

            // 日志
            VStack(alignment: .leading) {
                Text("Log")
                    .font(.caption.bold())
                    .foregroundColor(.secondary)
                ScrollViewReader { proxy in
                    ScrollView {
                        LazyVStack(alignment: .leading, spacing: 2) {
                            ForEach(Array(logs.enumerated()), id: \.offset) { idx, log in
                                Text(log)
                                    .font(.system(.caption, design: .monospaced))
                                    .foregroundColor(.secondary)
                                    .id(idx)
                            }
                        }
                    }
                    .onChange(of: logs.count) { _ in
                        if let last = logs.indices.last {
                            proxy.scrollTo(last, anchor: .bottom)
                        }
                    }
                }
            }
            .padding(.horizontal)
            .padding(.bottom, 8)
            .frame(maxHeight: 120)
        }
        .onAppear(perform: loadDevices)
    }

    private func loadDevices() {
        do {
            devices = try MicvolBridge.inputDevices()
            if let defaultDevice = devices.first(where: { $0.isDefault }) {
                selectedDeviceId = defaultDevice.id
            } else if let first = devices.first {
                selectedDeviceId = first.id
            }
            refreshVolume()
            addLog("Loaded \(devices.count) input device(s)")
        } catch {
            addLog("Error loading devices: \(error.localizedDescription)")
        }
    }

    private func refreshVolume() {
        do {
            currentVolume = try MicvolBridge.getVolume(deviceId: selectedDeviceId)
            isMuted = try MicvolBridge.getMute(deviceId: selectedDeviceId)
        } catch {
            addLog("Error reading volume: \(error.localizedDescription)")
        }
    }

    private func toggleGuard() {
        if let handle = guardHandle {
            // Restore
            do {
                try MicvolBridge.guardRestore(handle)
                guardHandle = nil
                refreshVolume()
                addLog("Guard restored, volume back to \(String(format: "%.0f%%", currentVolume * 100))")
            } catch {
                addLog("Error restoring: \(error.localizedDescription)")
            }
        } else {
            // Maximize
            do {
                let before = currentVolume
                guardHandle = try MicvolBridge.guardMaximize(deviceId: selectedDeviceId)
                refreshVolume()
                addLog("Guard active! Volume \(String(format: "%.0f%%", before * 100)) -> 100%")
            } catch {
                addLog("Error maximizing: \(error.localizedDescription)")
            }
        }
    }

    private func toggleMute() {
        do {
            try MicvolBridge.setVolume(deviceId: selectedDeviceId, volume: isMuted ? 0.5 : 0)
            if !isMuted {
                // mute via set_mute
            }
            isMuted = !isMuted
            refreshVolume()
            addLog(isMuted ? "Muted" : "Unmuted")
        } catch {
            addLog("Error: \(error.localizedDescription)")
        }
    }

    private func addLog(_ message: String) {
        let time = DateFormatter.localizedString(from: Date(), dateStyle: .none, timeStyle: .medium)
        logs.append("[\(time)] \(message)")
    }
}
