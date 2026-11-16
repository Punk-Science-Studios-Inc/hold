use std::ffi::c_void;
use std::mem::size_of;
use windows::core::w;
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegGetValueW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
    KEY_SET_VALUE, REG_DWORD, REG_OPTION_NON_VOLATILE, RRF_RT_REG_DWORD,
};

const KEY: windows::core::PCWSTR = w!("Software\\Hold");
const VALUE: windows::core::PCWSTR = w!("KeepDisplayOn");

pub fn load_keep_display() -> bool {
    let mut data: u32 = 1;
    let mut size = size_of::<u32>() as u32;
    let result = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            KEY,
            VALUE,
            RRF_RT_REG_DWORD,
            None,
            Some(&mut data as *mut u32 as *mut c_void),
            Some(&mut size),
        )
    };
    result.is_err() || data != 0
}

pub fn save_keep_display(keep: bool) {
    let mut key = HKEY::default();
    let opened = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            KEY,
            None,
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            None,
            &mut key,
            None,
        )
    };
    if opened.is_err() {
        return;
    }
    let data = (keep as u32).to_le_bytes();
    unsafe {
        let _ = RegSetValueExW(key, VALUE, None, REG_DWORD, Some(&data));
        let _ = RegCloseKey(key);
    }
}
