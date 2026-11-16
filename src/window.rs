use std::cell::{Cell, RefCell};
use windows::core::{w, Result};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::RemoteDesktop::{
    WTSRegisterSessionNotification, WTSUnRegisterSessionNotification, NOTIFY_FOR_THIS_SESSION,
};
use windows::Win32::UI::Shell::{NINF_KEY, NIN_SELECT};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, KillTimer,
    PostQuitMessage, RegisterClassW, RegisterWindowMessageW, SetTimer, TranslateMessage, MSG,
    PBT_APMPOWERSTATUSCHANGE, PBT_APMRESUMEAUTOMATIC, WINDOW_STYLE, WM_COMMAND, WM_CONTEXTMENU,
    WM_POWERBROADCAST, WM_TIMER, WM_WTSSESSION_CHANGE, WNDCLASSW, WS_EX_TOOLWINDOW,
    WTS_SESSION_LOCK, WTS_SESSION_UNLOCK,
};

use crate::core::{self, Effect, Event, State};
use crate::{injector, keep_awake, power, settings, tray};

const TIMER_ID: usize = 1;
const TICK_MS: u32 = 60_000;
const NIN_KEYSELECT: u32 = NIN_SELECT | NINF_KEY;

struct App {
    hwnd: HWND,
    state: State,
    icons: tray::Icons,
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
    static TASKBAR_CREATED: Cell<u32> = const { Cell::new(0) };
}

pub fn run(state: State) -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class = w!("HoldWindow");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: class,
            ..Default::default()
        };
        RegisterClassW(&wc);
        TASKBAR_CREATED.set(RegisterWindowMessageW(w!("TaskbarCreated")));

        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            class,
            w!("Hold"),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        let icons = tray::Icons::load()?;
        tray::add(hwnd, &icons, state.icon());
        APP.set(Some(App { hwnd, state, icons }));

        let _ = WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION);
        SetTimer(Some(hwnd), TIMER_ID, TICK_MS, None);
        dispatch(Event::Start);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let _ = KillTimer(Some(hwnd), TIMER_ID);
        let _ = WTSUnRegisterSessionNotification(hwnd);
        tray::remove(hwnd);
        let _ = DestroyWindow(hwnd);
    }
    Ok(())
}

/// Runs the event through the core, then carries out its effects with no borrow held,
/// because some effects pump messages and re-enter the window procedure.
fn dispatch(event: Event) {
    let Some((hwnd, effects)) = APP.with_borrow_mut(|app| {
        app.as_mut()
            .map(|a| (a.hwnd, core::update(&mut a.state, event)))
    }) else {
        return;
    };
    for effect in effects {
        match effect {
            Effect::ApplyPower { display } => keep_awake::apply(display),
            Effect::ClearPower => keep_awake::clear(),
            Effect::Inject => injector::inject(),
            Effect::ShowIcon(kind) => APP.with_borrow(|a| {
                if let Some(a) = a {
                    tray::set_icon(hwnd, &a.icons, kind);
                }
            }),
            Effect::ReAddIcon(kind) => APP.with_borrow(|a| {
                if let Some(a) = a {
                    tray::add(hwnd, &a.icons, kind);
                }
            }),
            Effect::SaveDisplay(keep) => settings::save_keep_display(keep),
            Effect::NotifyLowBattery => tray::notify_low_battery(hwnd),
            Effect::ExitLoop => unsafe { PostQuitMessage(0) },
        }
    }
}

fn power_event() -> Event {
    let (on_ac, percent) = power::battery();
    Event::Power { on_ac, percent }
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        tray::CALLBACK => match (lparam.0 & 0xFFFF) as u32 {
            NIN_SELECT | NIN_KEYSELECT => dispatch(Event::Click),
            WM_CONTEXTMENU => {
                let keep = APP.with_borrow(|a| a.as_ref().is_some_and(|a| a.state.keep_display));
                tray::show_menu(hwnd, keep);
            }
            _ => {}
        },
        WM_COMMAND => match wparam.0 & 0xFFFF {
            tray::MENU_DISPLAY => dispatch(Event::ToggleDisplay),
            tray::MENU_QUIT => dispatch(Event::Quit),
            _ => {}
        },
        WM_TIMER if wparam.0 == TIMER_ID => {
            dispatch(power_event());
            dispatch(Event::Tick {
                idle_ms: power::idle_ms(),
            });
        }
        WM_WTSSESSION_CHANGE => match wparam.0 as u32 {
            WTS_SESSION_LOCK => dispatch(Event::Lock),
            WTS_SESSION_UNLOCK => dispatch(Event::Unlock),
            _ => {}
        },
        WM_POWERBROADCAST => {
            match wparam.0 as u32 {
                PBT_APMRESUMEAUTOMATIC => {
                    dispatch(Event::Resume);
                    unsafe { SetTimer(Some(hwnd), TIMER_ID, TICK_MS, None) };
                }
                PBT_APMPOWERSTATUSCHANGE => dispatch(power_event()),
                _ => {}
            }
            return LRESULT(1);
        }
        m if m != 0 && m == TASKBAR_CREATED.get() => dispatch(Event::TaskbarCreated),
        _ => return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
    LRESULT(0)
}
