use windows::Win32::System::Power::{
    SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED,
};

pub fn apply(display: bool) {
    let mut flags = ES_CONTINUOUS | ES_SYSTEM_REQUIRED;
    if display {
        flags |= ES_DISPLAY_REQUIRED;
    }
    unsafe { SetThreadExecutionState(flags) };
}

pub fn clear() {
    unsafe { SetThreadExecutionState(ES_CONTINUOUS) };
}
