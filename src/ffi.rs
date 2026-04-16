//! C FFI 导出层。
//!
//! 所有函数返回 0 表示成功，负数表示错误码。
//! VolumeGuard 以不透明指针形式暴露。

use crate::{manager, types::*};
use std::ffi::CString;
use std::os::raw::c_char;

// --- 错误码 ---

const MICVOL_OK: i32 = 0;
const MICVOL_ERR_COREAUDIO: i32 = -1;
const MICVOL_ERR_DEVICE_NOT_FOUND: i32 = -2;
const MICVOL_ERR_NO_DEFAULT_INPUT: i32 = -3;
const MICVOL_ERR_VOLUME_OUT_OF_RANGE: i32 = -4;
const MICVOL_ERR_NOT_SUPPORTED: i32 = -5;
const MICVOL_ERR_NULL_POINTER: i32 = -6;

fn error_code(e: &crate::error::Error) -> i32 {
    match e {
        crate::error::Error::CoreAudio(_) => MICVOL_ERR_COREAUDIO,
        crate::error::Error::DeviceNotFound => MICVOL_ERR_DEVICE_NOT_FOUND,
        crate::error::Error::NoDefaultInputDevice => MICVOL_ERR_NO_DEFAULT_INPUT,
        crate::error::Error::VolumeOutOfRange(_) => MICVOL_ERR_VOLUME_OUT_OF_RANGE,
        crate::error::Error::NotSupported(_) => MICVOL_ERR_NOT_SUPPORTED,
    }
}

// --- C 兼容的设备信息结构 ---

/// C 侧的设备信息。name 需要调用 micvol_free_string 释放。
#[repr(C)]
pub struct MicvolDeviceInfo {
    pub device_id: u32,
    pub name: *mut c_char,
    pub channels: u32,
    pub is_default: i32,
}

// --- 设备枚举 ---

/// 获取默认输入设备。
/// device_id 和 name 为输出参数。name 需要 micvol_free_string 释放。
#[unsafe(no_mangle)]
pub extern "C" fn micvol_default_input_device(
    device_id: *mut u32,
    name: *mut *mut c_char,
) -> i32 {
    if device_id.is_null() || name.is_null() {
        return MICVOL_ERR_NULL_POINTER;
    }
    match manager::default_input_device() {
        Ok(info) => {
            unsafe {
                *device_id = info.id.raw();
                *name = CString::new(info.name).unwrap_or_default().into_raw();
            }
            MICVOL_OK
        }
        Err(e) => error_code(&e),
    }
}

/// 枚举所有输入设备。
/// buf: 调用方分配的数组；buf_len: 数组容量；count: 实际写入的数量。
/// 每个 MicvolDeviceInfo 的 name 需要 micvol_free_string 释放。
#[unsafe(no_mangle)]
pub extern "C" fn micvol_input_devices(
    buf: *mut MicvolDeviceInfo,
    buf_len: u32,
    count: *mut u32,
) -> i32 {
    if buf.is_null() || count.is_null() {
        return MICVOL_ERR_NULL_POINTER;
    }
    match manager::input_devices() {
        Ok(devices) => {
            let n = devices.len().min(buf_len as usize);
            for (i, dev) in devices.into_iter().take(n).enumerate() {
                let c_name = CString::new(dev.name).unwrap_or_default().into_raw();
                unsafe {
                    let slot = &mut *buf.add(i);
                    slot.device_id = dev.id.raw();
                    slot.name = c_name;
                    slot.channels = dev.channels;
                    slot.is_default = if dev.is_default { 1 } else { 0 };
                }
            }
            unsafe { *count = n as u32 };
            MICVOL_OK
        }
        Err(e) => error_code(&e),
    }
}

// --- 音量控制 ---

#[unsafe(no_mangle)]
pub extern "C" fn micvol_get_volume(device_id: u32, volume: *mut f32) -> i32 {
    if volume.is_null() {
        return MICVOL_ERR_NULL_POINTER;
    }
    let id = DeviceId::from_raw(device_id);
    match manager::get_volume(&id) {
        Ok(v) => {
            unsafe { *volume = v.as_f32() };
            MICVOL_OK
        }
        Err(e) => error_code(&e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn micvol_set_volume(device_id: u32, volume: f32) -> i32 {
    let id = DeviceId::from_raw(device_id);
    let vol = match Volume::new(volume) {
        Ok(v) => v,
        Err(e) => return error_code(&e),
    };
    match manager::set_volume(&id, vol) {
        Ok(()) => MICVOL_OK,
        Err(e) => error_code(&e),
    }
}

// --- 静音控制 ---

#[unsafe(no_mangle)]
pub extern "C" fn micvol_get_mute(device_id: u32, muted: *mut i32) -> i32 {
    if muted.is_null() {
        return MICVOL_ERR_NULL_POINTER;
    }
    let id = DeviceId::from_raw(device_id);
    match manager::get_mute(&id) {
        Ok(state) => {
            unsafe {
                *muted = if state == MuteState::Muted { 1 } else { 0 };
            }
            MICVOL_OK
        }
        Err(e) => error_code(&e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn micvol_set_mute(device_id: u32, muted: i32) -> i32 {
    let id = DeviceId::from_raw(device_id);
    let state = if muted != 0 {
        MuteState::Muted
    } else {
        MuteState::Unmuted
    };
    match manager::set_mute(&id, state) {
        Ok(()) => MICVOL_OK,
        Err(e) => error_code(&e),
    }
}

// --- VolumeGuard ---

/// 创建 VolumeGuard 并将音量拉满。返回不透明句柄。
/// 必须通过 micvol_guard_restore 释放。
#[unsafe(no_mangle)]
pub extern "C" fn micvol_guard_maximize(device_id: u32, guard: *mut *mut std::ffi::c_void) -> i32 {
    if guard.is_null() {
        return MICVOL_ERR_NULL_POINTER;
    }
    let id = DeviceId::from_raw(device_id);
    match crate::VolumeGuard::maximize(&id) {
        Ok(g) => {
            let boxed = Box::new(g);
            unsafe { *guard = Box::into_raw(boxed) as *mut std::ffi::c_void };
            MICVOL_OK
        }
        Err(e) => error_code(&e),
    }
}

/// 恢复音量并释放 guard。调用后 guard 指针失效。
#[unsafe(no_mangle)]
pub extern "C" fn micvol_guard_restore(guard: *mut std::ffi::c_void) -> i32 {
    if guard.is_null() {
        return MICVOL_ERR_NULL_POINTER;
    }
    let boxed = unsafe { Box::from_raw(guard as *mut crate::VolumeGuard) };
    match boxed.restore() {
        Ok(()) => MICVOL_OK,
        Err(e) => error_code(&e),
    }
}

// --- 内存管理 ---

/// 释放由 micvol 分配的字符串。
#[unsafe(no_mangle)]
pub extern "C" fn micvol_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)) };
    }
}
