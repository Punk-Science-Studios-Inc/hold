pub const IDLE_THRESHOLD_MS: u32 = 50_000;
pub const BATTERY_CUTOFF: u8 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffReason {
    None,
    User,
    Battery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconKind {
    Open,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub on: bool,
    pub off_reason: OffReason,
    pub keep_display: bool,
    pub locked: bool,
    pub last_battery: Option<u8>,
}

impl State {
    pub fn new(keep_display: bool) -> Self {
        Self {
            on: false,
            off_reason: OffReason::None,
            keep_display,
            locked: false,
            last_battery: None,
        }
    }

    pub fn icon(&self) -> IconKind {
        if self.on {
            IconKind::Open
        } else {
            IconKind::Closed
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Start,
    Click,
    ToggleDisplay,
    Quit,
    Tick { idle_ms: u32 },
    Lock,
    Unlock,
    Resume,
    Power { on_ac: bool, percent: Option<u8> },
    TaskbarCreated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    ApplyPower { display: bool },
    ClearPower,
    Inject,
    ShowIcon(IconKind),
    ReAddIcon(IconKind),
    SaveDisplay(bool),
    NotifyLowBattery,
    ExitLoop,
}

pub fn update(state: &mut State, event: Event) -> Vec<Effect> {
    match event {
        Event::Start => turn_on(state),
        Event::Click if state.on => turn_off(state, OffReason::User),
        Event::Click => turn_on(state),
        Event::ToggleDisplay => {
            state.keep_display = !state.keep_display;
            let mut effects = vec![Effect::SaveDisplay(state.keep_display)];
            if state.on {
                effects.push(Effect::ApplyPower {
                    display: state.keep_display,
                });
            }
            effects
        }
        Event::Quit => vec![Effect::ClearPower, Effect::ExitLoop],
        Event::Tick { idle_ms } => {
            if state.on && !state.locked && idle_ms >= IDLE_THRESHOLD_MS {
                vec![Effect::Inject]
            } else {
                vec![]
            }
        }
        Event::Lock => {
            state.locked = true;
            vec![]
        }
        Event::Unlock => {
            state.locked = false;
            vec![]
        }
        Event::Resume if state.on => vec![Effect::ApplyPower {
            display: state.keep_display,
        }],
        Event::Resume => vec![],
        Event::Power { on_ac, percent } => power(state, on_ac, percent),
        Event::TaskbarCreated => vec![Effect::ReAddIcon(state.icon())],
    }
}

fn power(state: &mut State, on_ac: bool, percent: Option<u8>) -> Vec<Effect> {
    let previous = std::mem::replace(&mut state.last_battery, percent);

    if on_ac {
        if !state.on && state.off_reason == OffReason::Battery {
            return turn_on(state);
        }
        return vec![];
    }

    let crossed = matches!(
        (previous, percent),
        (Some(p), Some(c)) if p >= BATTERY_CUTOFF && c < BATTERY_CUTOFF
    );
    if state.on && crossed {
        let mut effects = turn_off(state, OffReason::Battery);
        effects.push(Effect::NotifyLowBattery);
        return effects;
    }
    vec![]
}

fn turn_on(state: &mut State) -> Vec<Effect> {
    state.on = true;
    state.off_reason = OffReason::None;
    vec![
        Effect::ApplyPower {
            display: state.keep_display,
        },
        Effect::ShowIcon(IconKind::Open),
    ]
}

fn turn_off(state: &mut State, reason: OffReason) -> Vec<Effect> {
    state.on = false;
    state.off_reason = reason;
    vec![Effect::ClearPower, Effect::ShowIcon(IconKind::Closed)]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn started() -> State {
        let mut s = State::new(true);
        update(&mut s, Event::Start);
        s
    }

    fn battery(s: &mut State, percent: u8) -> Vec<Effect> {
        update(
            s,
            Event::Power {
                on_ac: false,
                percent: Some(percent),
            },
        )
    }

    fn plug_in(s: &mut State) -> Vec<Effect> {
        update(
            s,
            Event::Power {
                on_ac: true,
                percent: s.last_battery,
            },
        )
    }

    #[test]
    fn start_turns_on() {
        let mut s = State::new(true);
        let fx = update(&mut s, Event::Start);
        assert!(s.on);
        assert_eq!(
            fx,
            vec![
                Effect::ApplyPower { display: true },
                Effect::ShowIcon(IconKind::Open)
            ]
        );
    }

    #[test]
    fn click_turns_off_as_user() {
        let mut s = started();
        let fx = update(&mut s, Event::Click);
        assert!(!s.on);
        assert_eq!(s.off_reason, OffReason::User);
        assert_eq!(
            fx,
            vec![Effect::ClearPower, Effect::ShowIcon(IconKind::Closed)]
        );
    }

    #[test]
    fn click_turns_back_on() {
        let mut s = started();
        update(&mut s, Event::Click);
        let fx = update(&mut s, Event::Click);
        assert!(s.on);
        assert_eq!(s.off_reason, OffReason::None);
        assert!(fx.contains(&Effect::ApplyPower { display: true }));
    }

    #[test]
    fn tick_under_threshold_does_nothing() {
        let mut s = started();
        let fx = update(
            &mut s,
            Event::Tick {
                idle_ms: IDLE_THRESHOLD_MS - 1,
            },
        );
        assert!(fx.is_empty());
    }

    #[test]
    fn tick_at_threshold_injects() {
        let mut s = started();
        let fx = update(
            &mut s,
            Event::Tick {
                idle_ms: IDLE_THRESHOLD_MS,
            },
        );
        assert_eq!(fx, vec![Effect::Inject]);
    }

    #[test]
    fn tick_while_locked_does_nothing() {
        let mut s = started();
        update(&mut s, Event::Lock);
        assert!(update(&mut s, Event::Tick { idle_ms: 120_000 }).is_empty());
        update(&mut s, Event::Unlock);
        assert_eq!(
            update(&mut s, Event::Tick { idle_ms: 120_000 }),
            vec![Effect::Inject]
        );
    }

    #[test]
    fn tick_while_off_does_nothing() {
        let mut s = started();
        update(&mut s, Event::Click);
        assert!(update(&mut s, Event::Tick { idle_ms: 120_000 }).is_empty());
    }

    #[test]
    fn crossing_below_cutoff_turns_off_and_notifies() {
        let mut s = started();
        battery(&mut s, 10);
        let fx = battery(&mut s, 9);
        assert!(!s.on);
        assert_eq!(s.off_reason, OffReason::Battery);
        assert_eq!(
            fx,
            vec![
                Effect::ClearPower,
                Effect::ShowIcon(IconKind::Closed),
                Effect::NotifyLowBattery
            ]
        );
    }

    #[test]
    fn falling_further_does_not_fire_again() {
        let mut s = started();
        battery(&mut s, 10);
        battery(&mut s, 9);
        assert!(battery(&mut s, 8).is_empty());
    }

    #[test]
    fn user_turning_on_below_cutoff_stays_on() {
        let mut s = started();
        battery(&mut s, 10);
        battery(&mut s, 9);
        battery(&mut s, 8);
        update(&mut s, Event::Click);
        assert!(battery(&mut s, 7).is_empty());
        assert!(s.on);
    }

    #[test]
    fn rising_and_falling_again_fires_again() {
        let mut s = started();
        battery(&mut s, 10);
        battery(&mut s, 9);
        update(&mut s, Event::Click);
        battery(&mut s, 10);
        battery(&mut s, 9);
        assert!(!s.on);
        assert_eq!(s.off_reason, OffReason::Battery);
    }

    #[test]
    fn plugging_in_after_cutoff_turns_on() {
        let mut s = started();
        battery(&mut s, 10);
        battery(&mut s, 9);
        let fx = plug_in(&mut s);
        assert!(s.on);
        assert!(fx.contains(&Effect::ShowIcon(IconKind::Open)));
    }

    #[test]
    fn plugging_in_after_user_off_stays_off() {
        let mut s = started();
        update(&mut s, Event::Click);
        assert!(plug_in(&mut s).is_empty());
        assert!(!s.on);
    }

    #[test]
    fn unknown_percent_never_cuts_off() {
        let mut s = started();
        battery(&mut s, 10);
        let fx = update(
            &mut s,
            Event::Power {
                on_ac: false,
                percent: None,
            },
        );
        assert!(fx.is_empty());
        assert!(s.on);
    }

    #[test]
    fn first_reading_below_cutoff_does_not_fire() {
        let mut s = started();
        assert!(battery(&mut s, 5).is_empty());
        assert!(s.on);
    }

    #[test]
    fn on_mains_low_percent_does_nothing() {
        let mut s = started();
        update(
            &mut s,
            Event::Power {
                on_ac: true,
                percent: Some(10),
            },
        );
        let fx = update(
            &mut s,
            Event::Power {
                on_ac: true,
                percent: Some(9),
            },
        );
        assert!(fx.is_empty());
        assert!(s.on);
    }

    #[test]
    fn toggle_display_while_on_saves_and_reapplies() {
        let mut s = started();
        let fx = update(&mut s, Event::ToggleDisplay);
        assert!(!s.keep_display);
        assert_eq!(
            fx,
            vec![
                Effect::SaveDisplay(false),
                Effect::ApplyPower { display: false }
            ]
        );
    }

    #[test]
    fn toggle_display_while_off_only_saves() {
        let mut s = started();
        update(&mut s, Event::Click);
        let fx = update(&mut s, Event::ToggleDisplay);
        assert_eq!(fx, vec![Effect::SaveDisplay(false)]);
    }

    #[test]
    fn resume_reapplies_only_when_on() {
        let mut s = started();
        assert_eq!(
            update(&mut s, Event::Resume),
            vec![Effect::ApplyPower { display: true }]
        );
        update(&mut s, Event::Click);
        assert!(update(&mut s, Event::Resume).is_empty());
    }

    #[test]
    fn quit_clears_and_exits() {
        let mut s = started();
        assert_eq!(
            update(&mut s, Event::Quit),
            vec![Effect::ClearPower, Effect::ExitLoop]
        );
    }

    #[test]
    fn taskbar_created_readds_current_icon() {
        let mut s = started();
        assert_eq!(
            update(&mut s, Event::TaskbarCreated),
            vec![Effect::ReAddIcon(IconKind::Open)]
        );
        update(&mut s, Event::Click);
        assert_eq!(
            update(&mut s, Event::TaskbarCreated),
            vec![Effect::ReAddIcon(IconKind::Closed)]
        );
    }
}
