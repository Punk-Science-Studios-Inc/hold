#![windows_subsystem = "windows"]

mod core;
mod injector;
mod keep_awake;
mod power;
mod settings;
mod tray;
mod window;

use windows::core::w;
use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS};
use windows::Win32::System::Threading::CreateMutexW;

fn main() -> windows::core::Result<()> {
    let mutex = unsafe { CreateMutexW(None, false, w!("Local\\Hold.SingleInstance"))? };
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        unsafe { CloseHandle(mutex)? };
        return Ok(());
    }

    let state = core::State::new(settings::load_keep_display());
    let result = window::run(state);
    unsafe { CloseHandle(mutex)? };
    result
}
