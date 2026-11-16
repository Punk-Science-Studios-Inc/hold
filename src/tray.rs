use std::mem::size_of;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIIF_INFO, NIM_ADD,
    NIM_DELETE, NIM_MODIFY, NIM_SETVERSION, NOTIFYICONDATAW, NOTIFYICONDATAW_0,
    NOTIFYICON_VERSION_4,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos, GetSystemMetrics, LoadImageW,
    PostMessageW, SetForegroundWindow, TrackPopupMenu, HICON, IMAGE_ICON, LR_DEFAULTCOLOR,
    MF_CHECKED, MF_SEPARATOR, MF_STRING, MF_UNCHECKED, SM_CXSMICON, SM_CYSMICON, TPM_BOTTOMALIGN,
    TPM_RIGHTBUTTON, WM_APP, WM_NULL,
};

use crate::core::IconKind;

pub const CALLBACK: u32 = WM_APP + 1;
pub const MENU_DISPLAY: usize = 1;
pub const MENU_QUIT: usize = 2;
const ICON_ID: u32 = 1;

pub struct Icons {
    open: HICON,
    closed: HICON,
}

impl Icons {
    pub fn load() -> windows::core::Result<Self> {
        Ok(Self {
            open: load_icon(1)?,
            closed: load_icon(2)?,
        })
    }

    fn get(&self, kind: IconKind) -> HICON {
        match kind {
            IconKind::Open => self.open,
            IconKind::Closed => self.closed,
        }
    }
}

fn load_icon(id: u16) -> windows::core::Result<HICON> {
    unsafe {
        let module = GetModuleHandleW(None)?;
        let handle = LoadImageW(
            Some(module.into()),
            PCWSTR(id as usize as *const u16),
            IMAGE_ICON,
            GetSystemMetrics(SM_CXSMICON),
            GetSystemMetrics(SM_CYSMICON),
            LR_DEFAULTCOLOR,
        )?;
        Ok(HICON(handle.0))
    }
}

fn base(hwnd: HWND) -> NOTIFYICONDATAW {
    NOTIFYICONDATAW {
        cbSize: size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: ICON_ID,
        ..Default::default()
    }
}

fn copy_wide(dest: &mut [u16], text: &str) {
    let max = dest.len() - 1;
    for (slot, unit) in dest.iter_mut().zip(text.encode_utf16().take(max)) {
        *slot = unit;
    }
}

pub fn add(hwnd: HWND, icons: &Icons, kind: IconKind) {
    let mut data = base(hwnd);
    data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_SHOWTIP;
    data.uCallbackMessage = CALLBACK;
    data.hIcon = icons.get(kind);
    copy_wide(&mut data.szTip, "Hold");
    data.Anonymous = NOTIFYICONDATAW_0 {
        uVersion: NOTIFYICON_VERSION_4,
    };
    unsafe {
        let _ = Shell_NotifyIconW(NIM_ADD, &data);
        let _ = Shell_NotifyIconW(NIM_SETVERSION, &data);
    }
}

pub fn set_icon(hwnd: HWND, icons: &Icons, kind: IconKind) {
    let mut data = base(hwnd);
    data.uFlags = NIF_ICON;
    data.hIcon = icons.get(kind);
    unsafe {
        let _ = Shell_NotifyIconW(NIM_MODIFY, &data);
    }
}

pub fn remove(hwnd: HWND) {
    let data = base(hwnd);
    unsafe {
        let _ = Shell_NotifyIconW(NIM_DELETE, &data);
    }
}

pub fn notify_low_battery(hwnd: HWND) {
    let mut data = base(hwnd);
    data.uFlags = NIF_INFO;
    data.dwInfoFlags = NIIF_INFO;
    copy_wide(&mut data.szInfoTitle, "Hold turned off");
    copy_wide(
        &mut data.szInfo,
        "Battery below 10%. Hold will turn back on when you plug in.",
    );
    unsafe {
        let _ = Shell_NotifyIconW(NIM_MODIFY, &data);
    }
}

pub fn show_menu(hwnd: HWND, keep_display: bool) {
    unsafe {
        let Ok(menu) = CreatePopupMenu() else { return };
        let check = if keep_display {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        };
        let _ = AppendMenuW(menu, MF_STRING | check, MENU_DISPLAY, w!("Keep display on"));
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
        let _ = AppendMenuW(menu, MF_STRING, MENU_QUIT, w!("Quit"));

        let mut point = POINT::default();
        let _ = GetCursorPos(&mut point);
        let _ = SetForegroundWindow(hwnd);
        let _ = TrackPopupMenu(
            menu,
            TPM_RIGHTBUTTON | TPM_BOTTOMALIGN,
            point.x,
            point.y,
            None,
            hwnd,
            None,
        );
        let _ = PostMessageW(Some(hwnd), WM_NULL, WPARAM_ZERO, LPARAM_ZERO);
        let _ = DestroyMenu(menu);
    }
}

const WPARAM_ZERO: windows::Win32::Foundation::WPARAM = windows::Win32::Foundation::WPARAM(0);
const LPARAM_ZERO: windows::Win32::Foundation::LPARAM = windows::Win32::Foundation::LPARAM(0);
