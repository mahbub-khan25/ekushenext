//! C ABI / FFI bindings for `ekushe-core`.
//!
//! Exposes a panic-safe C API for macOS InputMethodKit, Linux IBus/Fcitx5, Android JNI, and iOS.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic::catch_unwind;
use std::ptr;

use ekushe_core::{
    convert::{bijoy_to_unicode, unicode_to_bijoy},
    Ekushe,
};

pub struct EkusheHandle {
    inner: Ekushe,
}

#[no_mangle]
pub extern "C" fn ekushe_new() -> *mut EkusheHandle {
    catch_unwind(|| {
        let handle = Box::new(EkusheHandle {
            inner: Ekushe::default(),
        });
        Box::into_raw(handle)
    })
    .unwrap_or(ptr::null_mut())
}

/// # Safety
/// `handle` must be a valid pointer returned by `ekushe_new`.
#[no_mangle]
pub unsafe extern "C" fn ekushe_free(handle: *mut EkusheHandle) {
    if !handle.is_null() {
        let _ = catch_unwind(|| {
            drop(Box::from_raw(handle));
        });
    }
}

/// Feed a character and return a newly allocated JSON string (`SessionOutput`).
/// Caller must free with `ekushe_string_free`.
///
/// # Safety
/// `handle` must be a valid pointer returned by `ekushe_new`.
#[no_mangle]
pub unsafe extern "C" fn ekushe_push_char(
    handle: *mut EkusheHandle,
    codepoint: u32,
    altgr: bool,
) -> *mut c_char {
    if handle.is_null() {
        return ptr::null_mut();
    }
    let Some(ch) = char::from_u32(codepoint) else {
        return ptr::null_mut();
    };
    let h = &mut *handle;
    let out = h.inner.push_char(ch, altgr);
    let json = serde_json::to_string(&out).unwrap_or_else(|_| "{}".to_string());
    CString::new(json)
        .map(CString::into_raw)
        .unwrap_or(ptr::null_mut())
}

/// Convert Bijoy ANSI text to Unicode Bangla. Caller frees with `ekushe_string_free`.
///
/// # Safety
/// `input` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn ekushe_bijoy_to_unicode(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return ptr::null_mut();
    }
    let Ok(s) = CStr::from_ptr(input).to_str() else {
        return ptr::null_mut();
    };
    let out = bijoy_to_unicode(s);
    CString::new(out)
        .map(CString::into_raw)
        .unwrap_or(ptr::null_mut())
}

/// Convert Unicode Bangla text to Bijoy ANSI. Caller frees with `ekushe_string_free`.
///
/// # Safety
/// `input` must be a valid NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn ekushe_unicode_to_bijoy(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return ptr::null_mut();
    }
    let Ok(s) = CStr::from_ptr(input).to_str() else {
        return ptr::null_mut();
    };
    let out = unicode_to_bijoy(s);
    CString::new(out)
        .map(CString::into_raw)
        .unwrap_or(ptr::null_mut())
}

/// Free a C string returned by any `ekushe_*` function.
///
/// # Safety
/// `s` must be a pointer returned by an `ekushe_*` function.
#[no_mangle]
pub unsafe extern "C" fn ekushe_string_free(s: *mut c_char) {
    if !s.is_null() {
        let _ = CString::from_raw(s);
    }
}
