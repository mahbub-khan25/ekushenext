//! `ekushe-tsf`: Windows Text Services Framework (TSF) IME In-Process Server (`ekushe_tsf.dll`).
//!
//! Exports standard COM server entrypoints (`DllMain`, `DllGetClassObject`, `DllCanUnloadNow`,
//! `DllRegisterServer`, `DllUnregisterServer`) and registers the Ekushe Bangla (`bn-BD`, `0x0845`)
//! Text Input Processor profile.

use std::ffi::c_void;
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};

use windows_sys::core::{GUID, HRESULT};
use windows_sys::Win32::Foundation::{BOOL, CLASS_E_CLASSNOTAVAILABLE, E_INVALIDARG, HINSTANCE, S_FALSE, S_OK, TRUE};
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteTreeW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
    KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ,
};

/// Ekushe Text Service CLSID: `{E21B0945-2102-1952-B09C-0AD55559B326}`
pub const EKUSHE_CLSID_STR: &str = "{E21B0945-2102-1952-B09C-0AD55559B326}";
/// Ekushe Language Profile GUID: `{E21B0945-2102-1952-B09C-0AD55559B327}`
pub const EKUSHE_PROFILE_GUID_STR: &str = "{E21B0945-2102-1952-B09C-0AD55559B327}";

pub const EKUSHE_CLSID: GUID = GUID {
    data1: 0xE21B_0945,
    data2: 0x2102,
    data3: 0x1952,
    data4: [0xB0, 0x9C, 0x0A, 0xD5, 0x55, 0x59, 0xB3, 0x26],
};

static ACTIVE_OBJECTS: AtomicUsize = AtomicUsize::new(0);
static mut DLL_INSTANCE: HINSTANCE = ptr::null_mut();

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinst_dll: HINSTANCE,
    fdw_reason: u32,
    _lpv_reserved: *mut c_void,
) -> BOOL {
    if fdw_reason == 1 {
        // DLL_PROCESS_ATTACH
        DLL_INSTANCE = hinst_dll;
    }
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    if ACTIVE_OBJECTS.load(Ordering::SeqCst) == 0 {
        S_OK
    } else {
        S_FALSE
    }
}

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(
    rclsid: *const GUID,
    _riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() || rclsid.is_null() {
        return E_INVALIDARG;
    }
    *ppv = ptr::null_mut();
    CLASS_E_CLASSNOTAVAILABLE
}

/// Registers Ekushe COM class & CTF TIP registry entries under `HKCU\Software\Classes`
/// so registration works even without UAC administrator elevation.
#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    let clsid_key = format!("Software\\Classes\\CLSID\\{EKUSHE_CLSID_STR}");
    let inproc_key = format!("{clsid_key}\\InprocServer32");
    let tip_key = format!(
        "Software\\Microsoft\\CTF\\TIP\\{EKUSHE_CLSID_STR}\\LanguageProfile\\0x00000845\\{EKUSHE_PROFILE_GUID_STR}"
    );

    if set_reg_sz(HKEY_CURRENT_USER, &clsid_key, None, "Ekushe Next Bangla Input Method").is_err() {
        return -1;
    }
    if set_reg_sz(HKEY_CURRENT_USER, &inproc_key, Some("ThreadingModel"), "Apartment").is_err() {
        return -1;
    }
    let _ = set_reg_sz(
        HKEY_CURRENT_USER,
        &tip_key,
        Some("Description"),
        "Ekushe Next Bangla (একুশে নেক্সট বাংলা)",
    );
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    let clsid_key = wide_null(&format!("Software\\Classes\\CLSID\\{EKUSHE_CLSID_STR}"));
    let tip_key = wide_null(&format!("Software\\Microsoft\\CTF\\TIP\\{EKUSHE_CLSID_STR}"));
    let _ = RegDeleteTreeW(HKEY_CURRENT_USER, clsid_key.as_ptr());
    let _ = RegDeleteTreeW(HKEY_CURRENT_USER, tip_key.as_ptr());
    S_OK
}

unsafe fn set_reg_sz(root: HKEY, subkey: &str, name: Option<&str>, value: &str) -> Result<(), u32> {
    let subkey_w = wide_null(subkey);
    let mut hkey: HKEY = ptr::null_mut();
    let status = RegCreateKeyExW(
        root,
        subkey_w.as_ptr(),
        0,
        ptr::null(),
        REG_OPTION_NON_VOLATILE,
        KEY_WRITE,
        ptr::null(),
        &mut hkey,
        ptr::null_mut(),
    );
    if status != 0 {
        return Err(status);
    }

    let val_w = wide_null(value);
    let name_w = name.map(wide_null);
    let name_ptr = name_w.as_ref().map_or(ptr::null(), |v| v.as_ptr());

    let status = RegSetValueExW(
        hkey,
        name_ptr,
        0,
        REG_SZ,
        val_w.as_ptr() as *const u8,
        (val_w.len() * 2) as u32,
    );
    RegCloseKey(hkey);
    if status == 0 {
        Ok(())
    } else {
        Err(status)
    }
}

fn wide_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
