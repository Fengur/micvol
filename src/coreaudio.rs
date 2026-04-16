//! CoreAudio HAL FFI 安全封装。所有 unsafe 集中在此文件。

use crate::error::{Error, Result};
use coreaudio_sys::*;
use std::mem;
use std::ptr;

/// 检查 OSStatus，非 0 则返回 Error::CoreAudio。
fn check(status: OSStatus) -> Result<()> {
    if status == 0 {
        Ok(())
    } else {
        Err(Error::CoreAudio(status))
    }
}

/// 读取一个固定大小的属性值。
pub fn get_property<T: Sized>(
    object_id: AudioObjectID,
    address: &AudioObjectPropertyAddress,
) -> Result<T> {
    let mut value = mem::MaybeUninit::<T>::uninit();
    let mut size = mem::size_of::<T>() as u32;
    check(unsafe {
        AudioObjectGetPropertyData(
            object_id,
            address as *const _,
            0,
            ptr::null(),
            &mut size,
            value.as_mut_ptr() as *mut _,
        )
    })?;
    Ok(unsafe { value.assume_init() })
}

/// 设置一个固定大小的属性值。
pub fn set_property<T: Sized>(
    object_id: AudioObjectID,
    address: &AudioObjectPropertyAddress,
    value: &T,
) -> Result<()> {
    check(unsafe {
        AudioObjectSetPropertyData(
            object_id,
            address as *const _,
            0,
            ptr::null(),
            mem::size_of::<T>() as u32,
            value as *const T as *const _,
        )
    })
}

/// 读取一个变长属性（如设备列表），返回 Vec<T>。
pub fn get_property_array<T: Sized + Clone>(
    object_id: AudioObjectID,
    address: &AudioObjectPropertyAddress,
) -> Result<Vec<T>> {
    // 先查大小
    let mut size: u32 = 0;
    check(unsafe {
        AudioObjectGetPropertyDataSize(
            object_id,
            address as *const _,
            0,
            ptr::null(),
            &mut size,
        )
    })?;

    let count = size as usize / mem::size_of::<T>();
    if count == 0 {
        return Ok(Vec::new());
    }

    let mut buf: Vec<T> = Vec::with_capacity(count);
    check(unsafe {
        AudioObjectGetPropertyData(
            object_id,
            address as *const _,
            0,
            ptr::null(),
            &mut size,
            buf.as_mut_ptr() as *mut _,
        )
    })?;
    unsafe { buf.set_len(count) };
    Ok(buf)
}

/// 检查某个属性是否存在。
pub fn has_property(
    object_id: AudioObjectID,
    address: &AudioObjectPropertyAddress,
) -> bool {
    unsafe { AudioObjectHasProperty(object_id, address as *const _) != 0 }
}

/// 获取系统默认输入设备 ID。
pub fn default_input_device_id() -> Result<AudioDeviceID> {
    let address = AudioObjectPropertyAddress {
        mSelector: kAudioHardwarePropertyDefaultInputDevice,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster,
    };
    let device_id: AudioDeviceID = get_property(kAudioObjectSystemObject, &address)?;
    if device_id == kAudioDeviceUnknown {
        return Err(Error::NoDefaultInputDevice);
    }
    Ok(device_id)
}

/// 获取系统中所有音频设备 ID。
pub fn all_device_ids() -> Result<Vec<AudioDeviceID>> {
    let address = AudioObjectPropertyAddress {
        mSelector: kAudioHardwarePropertyDevices,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster,
    };
    get_property_array(kAudioObjectSystemObject, &address)
}

/// 获取设备名称。
pub fn device_name(device_id: AudioDeviceID) -> Result<String> {
    let address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyDeviceNameCFString,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMaster,
    };
    let cf_ref: CFStringRef = get_property(device_id, &address)?;
    if cf_ref.is_null() {
        return Ok(String::from("Unknown"));
    }
    let name = unsafe { cfstring_to_string(cf_ref) };
    unsafe { CFRelease(cf_ref as *const _) };
    Ok(name)
}

/// 统计设备的输入通道数。参照原始 AudioDevice.cpp:244-268。
pub fn input_channel_count(device_id: AudioDeviceID) -> Result<u32> {
    let address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyStreamConfiguration,
        mScope: kAudioDevicePropertyScopeInput,
        mElement: 0,
    };

    // 获取 AudioBufferList 大小
    let mut size: u32 = 0;
    check(unsafe {
        AudioObjectGetPropertyDataSize(
            device_id,
            &address as *const _,
            0,
            ptr::null(),
            &mut size,
        )
    })?;

    if size == 0 {
        return Ok(0);
    }

    // 分配并读取 AudioBufferList
    let buf = vec![0u8; size as usize];
    let buf_list = buf.as_ptr() as *mut AudioBufferList;
    let mut actual_size = size;
    check(unsafe {
        AudioObjectGetPropertyData(
            device_id,
            &address as *const _,
            0,
            ptr::null(),
            &mut actual_size,
            buf_list as *mut _,
        )
    })?;

    let mut channels = 0u32;
    unsafe {
        let num_buffers = (*buf_list).mNumberBuffers;
        let buffers_ptr = &(*buf_list).mBuffers as *const AudioBuffer;
        for i in 0..num_buffers as usize {
            channels += (*buffers_ptr.add(i)).mNumberChannels;
        }
    }
    Ok(channels)
}

/// 读取设备音量标量 (0.0-1.0)。参照原始 AudioDevice.cpp:94。
pub fn get_volume_scalar(device_id: AudioDeviceID) -> Result<f32> {
    let address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyVolumeScalar,
        mScope: kAudioDevicePropertyScopeInput,
        mElement: kAudioObjectPropertyElementMaster,
    };

    // 有些设备 master channel 不支持，尝试 channel 1
    if has_property(device_id, &address) {
        return get_property(device_id, &address);
    }

    let address_ch1 = AudioObjectPropertyAddress {
        mElement: 1,
        ..address
    };
    if has_property(device_id, &address_ch1) {
        return get_property(device_id, &address_ch1);
    }

    Err(Error::NotSupported(
        "device has no volume control".into(),
    ))
}

/// 设置设备音量标量。参照原始 AudioDevice.cpp:105-106。
pub fn set_volume_scalar(device_id: AudioDeviceID, volume: f32) -> Result<()> {
    let address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyVolumeScalar,
        mScope: kAudioDevicePropertyScopeInput,
        mElement: kAudioObjectPropertyElementMaster,
    };

    if has_property(device_id, &address) {
        return set_property(device_id, &address, &volume);
    }

    // 回退到 channel 1
    let address_ch1 = AudioObjectPropertyAddress {
        mElement: 1,
        ..address
    };
    if has_property(device_id, &address_ch1) {
        return set_property(device_id, &address_ch1, &volume);
    }

    Err(Error::NotSupported(
        "device has no volume control".into(),
    ))
}

/// 获取静音状态。参照原始 AudioDevice.cpp:110。
pub fn get_mute(device_id: AudioDeviceID) -> Result<bool> {
    let address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyMute,
        mScope: kAudioDevicePropertyScopeInput,
        mElement: kAudioObjectPropertyElementMaster,
    };

    if !has_property(device_id, &address) {
        // 没有静音控制 = 未静音
        return Ok(false);
    }

    let muted: u32 = get_property(device_id, &address)?;
    Ok(muted != 0)
}

/// 设置静音状态。参照原始 AudioDevice.cpp:112-113。
pub fn set_mute(device_id: AudioDeviceID, muted: bool) -> Result<()> {
    let address = AudioObjectPropertyAddress {
        mSelector: kAudioDevicePropertyMute,
        mScope: kAudioDevicePropertyScopeInput,
        mElement: kAudioObjectPropertyElementMaster,
    };

    if !has_property(device_id, &address) {
        return Ok(()); // 没有静音控制，忽略
    }

    let value: u32 = if muted { 1 } else { 0 };
    set_property(device_id, &address, &value)
}

// --- CFString 工具 ---

use coreaudio_sys::{CFRelease, CFStringRef};

unsafe fn cfstring_to_string(cf_ref: CFStringRef) -> String {
    use coreaudio_sys::{
        kCFStringEncodingUTF8, CFStringGetCStringPtr, CFStringGetLength,
        CFStringGetCString, CFIndex,
    };

    let ptr = CFStringGetCStringPtr(cf_ref, kCFStringEncodingUTF8);
    if !ptr.is_null() {
        return std::ffi::CStr::from_ptr(ptr)
            .to_string_lossy()
            .into_owned();
    }

    // 回退：手动拷贝
    let len = CFStringGetLength(cf_ref) as usize;
    let buf_size = len * 4 + 1; // UTF-8 最大 4 字节/字符
    let mut buf = vec![0i8; buf_size];
    CFStringGetCString(
        cf_ref,
        buf.as_mut_ptr(),
        buf_size as CFIndex,
        kCFStringEncodingUTF8,
    );
    std::ffi::CStr::from_ptr(buf.as_ptr())
        .to_string_lossy()
        .into_owned()
}
