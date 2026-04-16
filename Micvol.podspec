Pod::Spec.new do |s|
  s.name         = "Micvol"
  s.version      = "0.1.0"
  s.summary      = "macOS microphone hardware input volume control via CoreAudio HAL"
  s.description  = <<-DESC
    Control the macOS microphone input volume at the hardware level.
    Provides device enumeration, volume/mute control, and a VolumeGuard
    that automatically restores the original volume when released.
  DESC
  s.homepage     = "https://github.com/Fengur/micvol"
  s.license      = { :type => "MIT", :file => "LICENSE-MIT" }
  s.author       = { "Fengur" => "fengur@users.noreply.github.com" }

  s.platform     = :osx, "10.15"
  s.source       = { :git => "https://github.com/Fengur/micvol.git", :tag => "v#{s.version}" }

  s.vendored_libraries = "dist/libmicvol.a"
  s.source_files       = "include/micvol.h"
  s.public_header_files = "include/micvol.h"

  s.frameworks   = "CoreAudio", "AudioToolbox", "CoreFoundation"
  s.libraries    = "resolv"

  s.pod_target_xcconfig = {
    "HEADER_SEARCH_PATHS" => "$(PODS_ROOT)/Micvol/include"
  }
end
