use std::mem::size_of;
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

const NO_BATTERY: u8 = 128;
const UNKNOWN_PERCENT: u8 = 255;

pub fn idle_ms() -> u32 {
    let mut info = LASTINPUTINFO {
        cbSize: size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    if !unsafe { GetLastInputInfo(&mut info) }.as_bool() {
        return 0;
    }
    unsafe { GetTickCount() }.wrapping_sub(info.dwTime)
}

/// Returns (on mains power, battery percent if known).
pub fn battery() -> (bool, Option<u8>) {
    let mut status = SYSTEM_POWER_STATUS::default();
    if unsafe { GetSystemPowerStatus(&mut status) }.is_err() {
        return (true, None);
    }
    let on_ac = status.ACLineStatus == 1;
    let percent =
        if status.BatteryFlag & NO_BATTERY != 0 || status.BatteryLifePercent == UNKNOWN_PERCENT {
            None
        } else {
            Some(status.BatteryLifePercent)
        };
    (on_ac, percent)
}
