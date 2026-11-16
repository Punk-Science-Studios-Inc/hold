# Hold: Windows Tray Keep-Awake and Presence App

*   **Author:** Darryl Wright
*   **Status:** Proposed
*   **Date:** 2026-11-16

--------------------------------------------------------------------------------

## Overview

Hold is a small Windows tray app, written in Rust, that stops the computer from
going to sleep while the user is away. It also tries to keep the user showing as
Active in the Slack desktop app.

The tray icon is an open eye when Hold is on and a closed eye when it is off.
Left-click switches between the two. The right-click menu has a "Keep display
on" checkbox and Quit. Hold starts at login, and it is on when it starts.

Hold does two separate jobs, and each uses its own Windows mechanism:

1.  **Keep the machine awake.** Hold asks Windows not to sleep. This is the
    job the app must do.
2.  **Look active.** When the user has been idle for a while, Hold sends a mouse
    move that leaves the pointer where it started. Windows counts it as user
    input. Slack relies on that count to decide whether the user is Away. This
    job is best-effort.

Hold is published to the public Chocolatey community feed, so anyone can install
it with `choco install hold`.

## The Problem

The author starts long-running tasks, such as builds and downloads, and walks
away. After the idle timeout, Windows puts the machine to sleep and the tasks
stop. While the author is away, Slack also switches them to Away.

The tool the author uses today is the Caffeine browser extension. It only works
while the browser is open, and on the author's machine the browser uses too much
memory. Other options exist, but each has a cost:

*   **PowerToys Awake** does not ship with Windows, so it must be found and
    installed first. It also does not simulate input, so Slack still shows Away.
*   **Zhorn's Caffeine** is an unknown third-party binary.
*   **`powercfg` overrides** are more complicated than the need.

The proposed solution is a single small executable with one click to turn it on
or off. It holds a Windows power request to prevent sleep, and while the user is
idle it sends a mouse move about once a minute to keep Slack's idle timer from
running out.

## Goals

*   **G1 (required): Keep the machine awake while Hold is on.**
    *   While Hold is on, the machine does not sleep from the idle timeout. This
        holds on mains power and on battery, with the lid open.
    *   **Check:** `powercfg /requests` lists Hold while it is on, and does not
        list it while it is off or after Quit.
    *   **Check:** with the battery sleep timeout set to 1 minute, the machine
        is still awake after 10 minutes unplugged.
*   **G2 (best-effort): Keep Slack showing the user as Active.** This applies
    while the session is unlocked and the user is idle.
    *   About every 60 seconds, if the user has been idle for about 50 seconds,
        Hold sends a mouse move. The pointer ends exactly where it started.
    *   **Check:** another person watches the user's Slack status from a second
        device. With Hold off and hands off, the user goes Away. With Hold on,
        over the same time, the user stays Active.
    *   G2 is built only if the step-1 Slack test passes.
*   **G3: Tray control.**
    *   Open eye means on and closed eye means off. A left-click switches
        between them.
    *   The right-click menu has "Keep display on", a checkbox that is checked
        by default, and Quit.
    *   When the box is unchecked, the display may turn off while the system
        stays awake.
    *   The checkbox setting is remembered per user across restarts.
*   **G4: Low-battery cutoff.**
    *   On battery, when the charge first drops below 10%, Hold turns itself
        off and shows one Windows notification saying why.
    *   When power is reconnected, Hold turns itself back on, but only if the
        cutoff is what turned it off.
    *   If the user turns Hold back on while still below 10%, it stays on.
*   **G5: Starts at login and runs one copy.**
    *   Hold starts at login and is on when it starts.
    *   Starting a second copy exits quietly.
    *   The tray icon comes back after Explorer restarts.
*   **G6: Public Chocolatey package.**
    *   Install adds the start-at-login entry, and uninstall removes it.
    *   Upgrade and uninstall stop the running app first.
    *   Nothing is left behind after uninstall: no process, no start-at-login
        entry, and no program files.
*   **G7: Small footprint.** One `.exe` with no installer runtime and no
    background service.
    *   *Proposed target, to be confirmed:* an exe under 1 MB and under 5 MB of
        RAM at idle.

## Non-Goals

*   Keeping the machine awake or Slack Active with the lid closed. Lid close is
    a Windows power-button setting that Hold does not override.
*   Keeping Slack Active while the session is locked. Injected input does not
    reach the lock screen.
*   Overriding Group Policy, device management (MDM) policy, a lock timeout set
    by policy, or Windows' critical-battery action.
*   Any feedback beyond the two icons and the low-battery notification. There
    is no degraded icon, no status tooltip and no log file.
*   A settings window, a configurable interval or threshold, or any setting
    other than "Keep display on".
*   A "Start at login" menu item. Start-at-login is controlled by
    install and uninstall.
*   The Slack API or any network access.
*   macOS, Linux or ARM64 builds.
*   Moving the pointer by a physical distance such as 1 cm.

## Open Questions

*   **Does Slack read Windows idle time, and does an injected mouse move reset
    it?** Check this in staging step 1. G2 depends on it.
*   **Does a zero-distance mouse move reset the idle timer?** If not, use a
    quick out-and-back of a few pixels, sent in a single call.
*   **Does Slack go Away as soon as the screen locks, whatever the input?** This
    is to observe, not decide. Also record Slack's current auto-away time.
*   **Does injected input still reset the idle timer when an elevated window is
    in front?** For example, an admin terminal running the long task.
*   **Code signing.** An unsigned app that injects input may be blocked by
    Chocolatey moderators or flagged by antivirus. Decide whether to buy a
    signing certificate if that happens.
*   **Are the G7 footprint targets right?**

## Critical User Journeys (CUJs)

### CUJ 1: Walk away from a long task

*   **Given**: Hold is on (open eye), "Keep display on" is checked, the session
    is unlocked, and a build is running.
*   **When**: The user leaves the machine for an hour.
*   **Then**:
    *   The machine does not sleep and the display stays on.
    *   About every 60 seconds, Hold sends one mouse move. The pointer ends
        where it started.
    *   Slack shows the user as Active the whole time (G2).
    *   The build finishes.

### CUJ 2: Turn Hold off and on

*   **Given**: Hold is on.
*   **When**: The user left-clicks the tray icon, then left-clicks it again.
*   **Then**:
    *   After the first click, the icon is a closed eye, `powercfg /requests`
        no longer lists Hold, and no input is sent.
    *   After the second click, the icon is an open eye and the request is
        back.

### CUJ 3: Let the display turn off

*   **Given**: Hold is on and "Keep display on" is checked.
*   **When**: The user unchecks "Keep display on" and walks away.
*   **Then**:
    *   The display turns off after the Windows display timeout. The system
        stays awake.
    *   After a reboot, the box is still unchecked.

### CUJ 4: Low battery, then the charger

*   **Given**: Hold is on, the laptop is unplugged, and the charge is 11%.
*   **When**: The charge drops to 9%, and later the user plugs in the charger.
*   **Then**:
    *   At 9%, the icon changes to a closed eye and one notification says Hold
        turned off because the battery is below 10%. Windows' own sleep
        settings apply again.
    *   When the charger is plugged in, the icon changes back to an open eye.
    *   If the user had turned Hold off themselves before plugging in, it
        stays off.

### CUJ 5: Install, upgrade and uninstall from Chocolatey

*   **Given**: An admin terminal on a machine without Hold.
*   **When**: The user runs `choco install hold`, logs out and back in, later
    runs `choco upgrade hold` while Hold is running, and finally runs
    `choco uninstall hold`.
*   **Then**:
    *   After install, typing `hold` in a terminal starts it, and Hold starts
        on its own at the next login for every user.
    *   Upgrade stops the running app, replaces the exe, and does not
        restart Hold. It starts again at the next login, or when the user types
        `hold`.
    *   Uninstall stops the app. Afterwards there is no Hold process, no
        start-at-login entry and no program files.
    *   The per-user display setting under `HKCU` stays behind after uninstall.
        See Technical Plan, section 7.

## Technical Plan (Architecture)

### 1. Shape of the program

Hold is one Rust executable built for `x86_64-pc-windows-msvc` as a Windows GUI
program, so no console window opens. It calls Windows directly through
Microsoft's `windows` crate and has no other runtime dependency.

Everything runs on one thread:

*   Hold creates one hidden top-level window (`WS_EX_TOOLWINDOW`).
*   It runs one message loop.
*   Every event arrives as a message to that window: tray clicks, menu choices,
    the timer, screen lock and unlock, sleep and resume, and battery changes.

Hold needs one thread because `SetThreadExecutionState` is tied to the thread
that calls it. With a single thread, every call comes from the same place. If
that thread or the process ends, for example when Hold crashes or is stopped
for an upgrade, Windows releases the request.

```text
                   ┌──────────────────── Windows ────────────────────┐
                   │ tray clicks   TaskbarCreated   WM_TIMER (60 s)   │
                   │ WM_WTSSESSION_CHANGE   WM_POWERBROADCAST         │
                   └───────────────────────┬─────────────────────────┘
                                           │ messages
                                           v
                     ┌─────────────────────────────────────┐
                     │  window procedure (win32 layer)     │
                     │  turns each message into an Event   │
                     └──────────────────┬──────────────────┘
                                        │ Event
                                        v
                     ┌─────────────────────────────────────┐
                     │  core: State + update(event) ->     │
                     │  list of Effects   (pure, no Win32) │
                     └──────────────────┬──────────────────┘
                                        │ Effects
          ┌──────────────┬──────────────┼──────────────┬───────────────┐
          v              v              v              v               v
   ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐
   │ keep_awake │ │  injector  │ │    tray    │ │  settings  │ │  notify    │
   │ SetThread- │ │ SendInput  │ │ Shell_     │ │ HKCU reg   │ │ balloon on │
   │ Execution- │ │ mouse move │ │ NotifyIcon │ │ value      │ │ tray icon  │
   │ State      │ │            │ │ + menu     │ │            │ │            │
   └────────────┘ └────────────┘ └────────────┘ └────────────┘ └────────────┘
```

### 2. Core state and decisions (no Windows calls)

The core is plain Rust with no Windows types, so unit tests can drive it.

State:

| Field          | Type                        | Meaning |
| -------------- | --------------------------- | ------- |
| `on`           | bool                        | The eye is open. |
| `off_reason`   | `None` / `User` / `Battery` | Why Hold is off. Used for the turn-back-on rule. |
| `keep_display` | bool                        | "Keep display on" checkbox. Loaded at start. |
| `locked`       | bool                        | The session is locked. |
| `last_battery` | optional percent            | The last reading, used to spot when the charge first drops below 10%. |

Events and what they do:

| Event | Effect |
| --- | --- |
| `Start` | `on = true`. Apply power flags. Show the open eye. |
| `Click` | Toggle `on`. When turning off, set `off_reason = User`. Apply power flags. Swap the icon. |
| `ToggleDisplay` | Flip `keep_display`. Save it. If `on`, apply power flags again. |
| `Quit` | Clear power flags. Remove the icon. Exit the loop. |
| `Tick(idle_ms)` | If `on`, not `locked`, and `idle_ms >= 50_000`, send one mouse move. Otherwise, do nothing. |
| `Lock` / `Unlock` | Set `locked`. Power flags do not change. |
| `Resume` | If `on`, apply power flags again. Restart the timer. |
| `Power(on_ac, percent)` | See the battery rules below. |
| `TaskbarCreated` | Add the icon again with its current image. |

The power flags are `ES_CONTINUOUS | ES_SYSTEM_REQUIRED`, plus
`ES_DISPLAY_REQUIRED` when `keep_display` is true. Off and Quit use
`ES_CONTINUOUS` alone, which clears the flags.

Battery rules:

*   **Cutoff:** if `on`, the machine is not on mains power, and the charge
    crosses from 10% or higher to below 10%, set `on = false` and
    `off_reason = Battery`. Clear the flags, show the closed eye and the
    notification.
*   **Reconnect:** if the machine is on mains power and `off_reason == Battery`,
    set `on = true` and apply the flags. Show the open eye.
*   The cutoff fires only when the charge crosses 10%. If the user turns Hold
    back on below 10%, nothing turns it off again unless the charge rises to
    10% or more and falls again.
*   An unknown percent (255) or a desktop with no battery never triggers the
    cutoff.

### 3. Timer and idle check

*   A 60-second `SetTimer` runs for the whole life of the app, including while
    Hold is off.
*   On each tick, Hold reads idle time as
    `GetTickCount().wrapping_sub(LASTINPUTINFO.dwTime)` in `u32`. The
    `wrapping_sub` handles the tick counter wrapping every 49.7 days.
*   The 50-second threshold sits below the 60-second interval because Hold's
    own move resets the idle clock. Each later tick therefore sees about 60
    seconds of idle time and fires again.
*   A threshold equal to the interval would skip every other tick, and moves
    would come about every 120 seconds.
*   The tick also reads the battery, as a fallback in case a battery-change
    message is missed.

### 4. Mouse move

There are two styles, set by a compile-time constant. The step-1 test decides
which one ships. Neither is a user setting.

*   **`Zero`**: one `INPUT` with `MOUSEEVENTF_MOVE`, `dx = 0` and `dy = 0`. The
    pointer does not move.
*   **`Pair`**: two `INPUT`s in a single `SendInput` call, `+N` then `-N`
    pixels on x, with N between 1 and 5. The net movement is zero within one
    call.

If Windows' "Enhance pointer precision" setting makes `Pair` drift the pointer,
the fallback is to read `GetCursorPos` and send absolute moves out and back.
That fallback is only built if testing shows drift.

`SetCursorPos` is not used, because Windows does not count it as user input.

### 5. Tray icon and menu

*   **Adding the icon:** `Shell_NotifyIconW` with `NIM_ADD`, then `NIM_SETVERSION`
    with `NOTIFYICON_VERSION_4`. The icon is identified by its window handle and
    ID, not a GUID, because a GUID ties the icon to one exe path.
*   **Clicks:** a left-click arrives as `NIN_SELECT`, and the keyboard
    equivalent as `NIN_KEYSELECT`. Both toggle.
*   **Menu:** `WM_CONTEXTMENU` opens it. Hold calls `SetForegroundWindow`, then
    `TrackPopupMenu`, so the menu closes when the user clicks elsewhere.
*   **Menu items:** "Keep display on", checked when on, and Quit.
*   **Tooltip:** the fixed text "Hold".
*   **Explorer restart:** Hold registers the `TaskbarCreated` message at start.
    When it arrives, Hold adds the icon again.
*   **Icons:** two `.ico` files, open eye and closed eye, embedded as resources.
    Each has 16, 20, 24 and 32 px images, loaded at the system small-icon size.
*   **Windows 11 overflow:** new tray icons start in the hidden overflow area.
    The README tells users to pin it.

### 6. Session, power and battery messages

*   **Lock and unlock:** `WTSRegisterSessionNotification` sends Hold
    `WM_WTSSESSION_CHANGE`. `WTS_SESSION_LOCK` and `WTS_SESSION_UNLOCK` become
    `Lock` and `Unlock`.
*   **Sleep and battery:** `WM_POWERBROADCAST` delivers two events.
    *   `PBT_APMRESUMEAUTOMATIC` becomes `Resume`.
    *   `PBT_APMPOWERSTATUSCHANGE` makes Hold call `GetSystemPowerStatus` and
        raise `Power(on_ac, percent)`.
*   **Notification:** the low-battery message is a balloon on the tray icon
    (`NIM_MODIFY` with `NIF_INFO`). Windows 10 and 11 show it as a standard
    notification. Hold does not need to register an app identity for that.

### 7. Settings

*   "Keep display on" is stored as a `DWORD` named `KeepDisplayOn` under
    `HKCU\Software\Hold`. 1 means on. A missing value means on.
*   Hold reads it at start and writes it when the user changes it.
*   Uninstall runs elevated and cannot reliably reach every user's `HKCU`, so
    the value stays behind. It is one small value and does nothing once Hold
    is removed. The README tells users how to delete it.

### 8. Single instance

*   At start, Hold creates the named mutex `Local\Hold.SingleInstance`.
*   If it already exists, Hold exits at once and does not signal the running
    copy.
*   The name has no version number, so an old and a new version can never run
    side by side.

### 9. Distribution

*   **Release:** each release puts `hold.exe` on GitHub Releases with its
    SHA256.
*   **Chocolatey package:** the package, with id `hold` (subject to the name
    being free on the community feed), downloads that exe into
    `lib\hold\tools\` and checks the SHA256.
*   **Install script:**
    *   Writes `HKLM\Software\Microsoft\Windows\CurrentVersion\Run`, value
        `Hold`, pointing at the real exe in `lib\hold\tools\`, not the
        Chocolatey shim.
    *   Does not start Hold, because the script runs elevated.
*   **Shim:** a `hold.exe.gui` file tells Chocolatey to make the `hold` shim a
    GUI launcher, so typing `hold` starts the app without holding the terminal.
*   **Before upgrade or uninstall:** `chocolateyBeforeModify.ps1` stops every
    `hold.exe` process in every session, so Windows releases the file.
*   **Uninstall script:** removes the `HKLM` Run value.
*   **Community feed checks:** the package includes `VERIFICATION.txt`, as the
    feed rules require. Moderators and VirusTotal check every version. The
    signing question is open (see Open Questions).

## Alternatives Considered

Most of these alternatives already meet G1 (stay awake). What justifies building
Hold is G2 (look active in Slack) together with a small footprint and no
browser.

### Alternative A: Microsoft PowerToys Awake

*   **Pros**: Maintained by Microsoft. Has a tray icon, plus timed and
    indefinite modes.
*   **Cons**: Does not ship with Windows; the whole PowerToys suite has to be
    found and installed. It does not simulate input, so Slack still shows
    Away.
*   **Reason for Rejection**: Rejected by author: not part of Windows, and does
    not meet G2.

### Alternative B: Zhorn's Caffeine

*   **Pros**: A small tray app that sends a fake F15 keypress. That probably
    keeps Slack Active *(unverified)*.
*   **Cons**: An unknown third-party binary from outside a package manager.
*   **Reason for Rejection**: Rejected by author: unknown third-party binary.

### Alternative C: `powercfg /requestsoverride` or changing the standby timeouts

*   **Pros**: Built into Windows, no code at all. Setting the standby timeout to
    0 (`powercfg /change standby-timeout-ac 0` and `-dc 0`) meets G1 for good.
*   **Cons**: `requestsoverride` is complicated to set up. Neither option can be
    switched on and off with one click, and neither meets G2.
*   **Reason for Rejection**: Rejected by author: too complicated for the need,
    and no Slack presence.

### Alternative D: Caffeine browser extension (the current tool)

*   **Pros**: Already in use. One click.
*   **Cons**: Only works while the browser runs. On the author's machine the
    browser uses too much memory to keep open. No Slack presence.
*   **Reason for Rejection**: Rejected by author: the browser's memory cost.

### Alternative E: Fake F15 keypress instead of a mouse move

*   **Pros**: The approach Caffeine uses, and known to reset the idle timer.
*   **Cons**: A keypress goes to whichever window has focus. F13 to F24 can set
    off shortcuts in some apps and remote-desktop tools. A mouse move has no
    target window.
*   **Reason for Rejection**: Rejected by author in favour of a mouse move.

### Alternative F: Slack API (`users.setPresence`)

*   **Pros**: No input injection, so no antivirus or UIPI problems.
*   **Cons**: Only accepts `auto` or `away`, so it cannot force Active
    *(unverified against current docs)*. It needs a user token with the
    `users:write` scope and network access.
*   **Reason for Rejection**: Rejected by author: cannot meet G2, and adds
    network access and a credential.

### Alternative G: Go instead of Rust

*   **Pros**: The author's default for small tools, and quicker to write. Go has
    pure-Go Windows tray libraries.
*   **Cons**: The runtime and garbage collector add a few MB of RAM *(estimate,
    unmeasured)* and a larger exe. Win32 calls go through community bindings
    rather than Microsoft's.
*   **Reason for Rejection**: Rejected by author: Rust has Microsoft's own
    `windows` crate and no runtime. The memory difference is small, so this
    was a preference, not a requirement.

### Alternative H: `tray-icon` and `muda` crates for the tray layer

*   **Pros**: Less tray and menu code, and the crates already handle the menu
    quirks.
*   **Cons**: Hold would still need its own hidden window for the lock, sleep
    and battery messages. That means two windows and two event sources, a
    bigger set of dependencies, and a larger exe. Re-adding the icon after an
    Explorer restart is unconfirmed.
*   **Reason for Rejection**: Rejected by author: direct Win32 keeps one window
    and one message loop.

### Alternative I: Power request objects (`PowerCreateRequest` / `PowerSetRequest`)

*   **Pros**: Not tied to a thread. A reason string shows in
    `powercfg /requests`. The display and system requests can be switched
    separately.
*   **Cons**: More calls, a reason structure to build, and a handle that every
    path must clear.
*   **Reason for Rejection**: Rejected by author: `SetThreadExecutionState`
    needs less code, and the single-thread design already meets its
    thread rule.

### Alternative J: Distribute with a Startup-folder shortcut or a private feed

*   **Pros**: Avoids community-feed moderation, VirusTotal review and the
    antivirus risk for an unsigned app.
*   **Cons**: Only the author can use it.
*   **Reason for Rejection**: Rejected by author: Hold should be available to
    the public.

### Alternative K: Low-level input hook to tell real input from Hold's own

*   **Pros**: Measures the user's own idle time exactly, without the
    threshold-below-interval rule.
*   **Cons**: A global input hook adds complexity and antivirus suspicion.
*   **Reason for Rejection**: Rejected by author: the 50 s threshold on a 60 s
    tick solves the same problem.

--------------------------------------------------------------------------------

## Non-Functional Requirements

*   **Security and trust:**
    *   Hold sends synthetic mouse input and starts at login. Antivirus tools
        look for both of these.
    *   Hold makes no network calls, reads no files, and writes one registry
        value.
    *   Every release is built in public on GitHub Actions from a tagged commit,
        and the SHA256 is published, so users and moderators can trace the exe
        back to its source.
    *   The exe is unsigned until the signing question is settled.
*   **Footprint:** under 1 MB exe and under 5 MB RAM at idle (proposed G7
    targets). It wakes once every 60 seconds and does no other work.
*   **Rollout and rollback:**
    *   Roll back with `choco install hold --version <previous>`.
    *   No data format changes between versions. The only stored value is
        `KeepDisplayOn`.

--------------------------------------------------------------------------------

### Future Improvements

*   **Code signing**, if moderators or antivirus tools flag the unsigned exe.
*   **A degraded icon state**, showing when injected input does not reset the
    idle timer. It was excluded from v1 by the author's "no extra feedback"
    decision.
*   **Light and dark icon variants** that follow the Windows taskbar theme.

--------------------------------------------------------------------------------

## Detailed Implementation Plan

Repository: `github.com/Punk-Science-Studios-Inc/hold`, MIT licence, default branch `main`.
Crate versions are pinned to the latest release on the day the project is
created and are locked in `Cargo.lock`.

### Build order

Each step is checked before the next one starts.

1.  **Presence test (spike).** Run `scripts/presence-spike.ps1`.
    *   It decides `Zero` or `Pair`, or drops G2 if neither works.
    *   It answers the Open Questions about Slack, the screen lock and UIPI.
    *   Record the results in `README.md`, under "Tested behaviour".
2.  **Skeleton.** Cargo project, resources, manifest and static CRT.
    *   **Check:** no console window appears, and the exe does not import
        `vcruntime140.dll`.
3.  **Core.** `src/core.rs` with unit tests.
    *   **Check:** `cargo test` passes.
4.  **Tray, window and single instance.**
    *   **Check:** clicks toggle the icon, the menu closes, the icon comes
        back after Explorer is killed and restarted, and a second launch exits.
5.  **Keep-awake.**
    *   **Check:** `powercfg /requests` with Hold on, off, and after Quit;
        the battery test from G1; the display checkbox test (CUJ 3).
6.  **Injector.**
    *   **Check:** the pointer ends where it started, nothing is sent while
        the user is typing, and the Slack test from G2 passes.
7.  **Battery rules.**
    *   **Check:** CUJ 4 on a laptop.
8.  **Packaging and CI.**
    *   **Check:** `choco pack`, then install, upgrade with Hold running, and
        uninstall on a clean Windows VM (CUJ 5). Then a tag creates a release.

### Target Files to Create/Modify

All files are new. Paths are relative to the repository root.

#### 1. `Cargo.toml` (Create)

*   **Type / Scope**: Build configuration.
*   **Proposed Changes**:
    *   Package `hold`, edition 2021, licence `MIT`.
    *   One dependency: `windows`, with only the features the app uses:
        *   `Win32_Foundation`
        *   `Win32_UI_WindowsAndMessaging`
        *   `Win32_UI_Shell`
        *   `Win32_UI_Input_KeyboardAndMouse`
        *   `Win32_System_Power`
        *   `Win32_System_SystemInformation`
        *   `Win32_System_Threading`
        *   `Win32_System_RemoteDesktop`
        *   `Win32_System_Registry`
        *   `Win32_System_LibraryLoader`
        *   `Win32_Graphics_Gdi`
    *   One build dependency: `embed-resource`.
    *   Release profile: `opt-level = "z"`, `lto = true`, `codegen-units = 1`,
        `panic = "abort"`, `strip = true`. These keep the exe small for G7.

#### 2. `Cargo.lock` (Create)

*   **Type / Scope**: Lock file.
*   **Proposed Changes**: Committed, so CI and local builds use the same
    crate versions.

#### 3. `rust-toolchain.toml` (Create)

*   **Type / Scope**: Toolchain pin.
*   **Proposed Changes**:
    *   Stable channel, pinned to the version current on the day the
        project is created.
    *   Target `x86_64-pc-windows-msvc`.
    *   Components `rustfmt` and `clippy`.

#### 4. `.cargo/config.toml` (Create)

*   **Type / Scope**: Build configuration.
*   **Proposed Changes**:
    *   For the msvc target, add `rustflags = ["-C", "target-feature=+crt-static"]`.
    *   This builds the C runtime into the exe, so it does not need the Visual
        C++ redistributable.

#### 5. `build.rs` (Create)

*   **Type / Scope**: Build script.
*   **Proposed Changes**:
    *   Calls `embed_resource::compile("res/hold.rc", embed_resource::NONE)`.
        The call's signature depends on the pinned `embed-resource` version.
    *   This puts the icons and the manifest inside the exe.

#### 6. `res/hold.rc` (Create)

*   **Type / Scope**: Windows resource script.
*   **Proposed Changes**:
    *   Icon resource ID 1: `eye_open.ico`.
    *   Icon resource ID 2: `eye_closed.ico`.
    *   Manifest resource 1: `hold.exe.manifest`.
    *   A `VERSIONINFO` block with the product name, the version, and the
        description "Keeps Windows awake".

#### 7. `res/hold.exe.manifest` (Create)

*   **Type / Scope**: Application manifest.
*   **Proposed Changes**:
    *   `asInvoker` execution level. Hold must never run elevated.
    *   `PerMonitorV2` DPI awareness, so the icons draw sharply.
    *   The supported OS entry for Windows 10 and 11.

#### 8. `res/eye_open.svg`, `res/eye_closed.svg` (Create)

*   **Type / Scope**: Icon source art.
*   **Proposed Changes**:
    *   Simple line-art eyes: open with an iris, and closed as a curved lid with
        lashes.
    *   A light fill with a dark outline, so each icon reads on both light and
        dark taskbars.

#### 9. `res/eye_open.ico`, `res/eye_closed.ico` (Create)

*   **Type / Scope**: Icon binaries, committed.
*   **Proposed Changes**:
    *   Exported once from the SVGs, at 16, 20, 24, 32 and 48 px.
    *   The README records the export command, so the icons can be rebuilt.

#### 10. `src/main.rs` (Create)

*   **Type / Scope**: Entry point.
*   **Proposed Changes**:
    *   `#![windows_subsystem = "windows"]`.
    *   Create the mutex `Local\Hold.SingleInstance`. If it already exists,
        return at once.
    *   Load settings.
    *   Call `window::run(initial_state)`, which owns the message loop.
    *   On exit, close the mutex handle.

#### 11. `src/core.rs` (Create)

*   **Type / Scope**: Pure logic, no Windows types.
*   **Proposed Changes**: Defines the following.
    *   **`State`** fields:
        *   `on: bool`
        *   `off_reason: OffReason`, one of `None`, `User` or `Battery`
        *   `keep_display: bool`
        *   `locked: bool`
        *   `last_battery: Option<u8>`
    *   **`Event`**:
        *   `Start`
        *   `Click`
        *   `ToggleDisplay`
        *   `Quit`
        *   `Tick { idle_ms: u32 }`
        *   `Lock`
        *   `Unlock`
        *   `Resume`
        *   `Power { on_ac: bool, percent: Option<u8> }`
        *   `TaskbarCreated`
    *   **`Effect`**:
        *   `ApplyPower { display: bool }`
        *   `ClearPower`
        *   `Inject`
        *   `ShowIcon(IconKind)`, where `IconKind` is `Open` or `Closed`
        *   `ReAddIcon`
        *   `SetDisplayCheck(bool)`
        *   `SaveDisplay(bool)`
        *   `NotifyLowBattery`
        *   `ExitLoop`
    *   **Constants:** `IDLE_THRESHOLD_MS = 50_000`, `BATTERY_CUTOFF = 10`.
    *   **`fn update(state: &mut State, event: Event) -> Vec<Effect>`** carries
        out the event table and the battery rules from Technical Plan,
        section 2.
    *   **Unit tests** in `#[cfg(test)] mod tests`, one per rule:
        *   A click turns Hold off, sets `User`, and clears the power request.
        *   A click turns Hold back on and applies the power request.
        *   A tick under 50 s sends nothing.
        *   A tick at 50 s or more while on and unlocked sends one move.
        *   A tick while locked sends nothing.
        *   A tick while off sends nothing.
        *   A drop from 10% to 9% on battery while on triggers the cutoff,
            sets `Battery`, and notifies.
        *   9% to 8% does not trigger again.
        *   Turning Hold on at 8% leaves it on at 7%.
        *   Mains power after a cutoff turns Hold back on.
        *   Mains power after a user turn-off leaves Hold off.
        *   An unknown percent (`None`) never triggers the cutoff.
        *   On mains power, a drop below 10% does nothing.
        *   `ToggleDisplay` while on saves the setting and applies the power
            request with the new display flag.
        *   `ToggleDisplay` while off saves the setting but applies no power
            request.
        *   `Resume` while on applies the power request again.
        *   `Quit` clears the power request and exits the loop.

#### 12. `src/window.rs` (Create)

*   **Type / Scope**: Win32 layer: the window and the message loop.
*   **Proposed Changes**:
    *   **`run(state)`**:
        *   Register the window class.
        *   Create the hidden `WS_EX_TOOLWINDOW` window.
        *   Register the `TaskbarCreated` message.
        *   Call `WTSRegisterSessionNotification` with
            `NOTIFY_FOR_THIS_SESSION`.
        *   `SetTimer`, timer ID 1, every 60,000 ms.
        *   Process `Event::Start`.
        *   Run `GetMessage` / `DispatchMessage` until the loop is told to
            exit.
    *   **The window procedure** turns each message into an `Event`:
        *   `WM_APP + 1`, the tray callback: `NIN_SELECT` and `NIN_KEYSELECT`
            become `Click`. `WM_CONTEXTMENU` opens the menu.
        *   `WM_COMMAND`, menu ID 1, becomes `ToggleDisplay`. Menu ID 2 becomes
            `Quit`.
        *   `WM_TIMER` becomes `Tick` with the idle time from `power.rs`. It
            also raises `Power` with the current battery reading.
        *   `WM_WTSSESSION_CHANGE` becomes `Lock` or `Unlock`.
        *   `WM_POWERBROADCAST` becomes `Resume` or `Power`.
        *   The registered `TaskbarCreated` message becomes `TaskbarCreated`.
    *   **The state is kept in one `static` behind a `RefCell`**, with no
        threads. The procedure calls `core::update`, then runs each `Effect`
        through the modules below.
    *   **On exit:** `KillTimer`, `WTSUnRegisterSessionNotification`, then
        `DestroyWindow`.

#### 13. `src/keep_awake.rs` (Create)

*   **Type / Scope**: Win32 wrapper.
*   **Proposed Changes**:
    *   `apply(display: bool)` calls `SetThreadExecutionState` with
        `ES_CONTINUOUS | ES_SYSTEM_REQUIRED`, plus `ES_DISPLAY_REQUIRED` if
        `display` is true.
    *   `clear()` calls it with `ES_CONTINUOUS` alone.
    *   Both are only ever called from the window procedure, so they always run
        on the message-loop thread.

#### 14. `src/injector.rs` (Create)

*   **Type / Scope**: Win32 wrapper.
*   **Proposed Changes**:
    *   `const STYLE: MoveStyle`, set from the step-1 result: `Zero`, or
        `Pair(n)` with n between 1 and 5.
    *   `inject()` builds one `INPUT` (`Zero`) or two (`Pair`) with
        `MOUSEEVENTF_MOVE` and makes one `SendInput` call. The return value is
        ignored, as D7 requires no feedback.

#### 15. `src/power.rs` (Create)

*   **Type / Scope**: Win32 wrapper.
*   **Proposed Changes**:
    *   **`idle_ms() -> u32`** calls `GetLastInputInfo`, with `cbSize` set,
        and returns `GetTickCount().wrapping_sub(dwTime)`.
    *   **`battery() -> (bool, Option<u8>)`** calls `GetSystemPowerStatus`.
        *   `on_ac` is `ACLineStatus == 1`.
        *   `percent` is `None` when `BatteryLifePercent == 255`, or when the
            no-battery bit (`BatteryFlag & 128`) is set.

#### 16. `src/tray.rs` (Create)

*   **Type / Scope**: Win32 wrapper.
*   **Proposed Changes**:
    *   **`add(hwnd, icon)`**: `NIM_ADD` with uID 1 and callback `WM_APP + 1`,
        then `NIM_SETVERSION` with version 4. The tooltip is "Hold".
    *   **`set_icon(icon)`**: `NIM_MODIFY` with the loaded `HICON`. Icons are
        loaded once with `LoadImageW` at `SM_CXSMICON` size.
    *   **`remove()`**: `NIM_DELETE`.
    *   **`show_menu(hwnd, keep_display)`**:
        *   Builds a popup menu with "Keep display on", checked to match
            `keep_display`, then a separator, then "Quit".
        *   Calls `SetForegroundWindow`, then `TrackPopupMenu` at the cursor,
            then posts `WM_NULL`.
    *   **`notify_low_battery()`**:
        *   `NIM_MODIFY` with `NIF_INFO`, title "Hold turned off", text
            "Battery below 10%. Hold will turn back on when you plug in."
        *   `NIIF_INFO` icon.

#### 17. `src/settings.rs` (Create)

*   **Type / Scope**: Win32 wrapper.
*   **Proposed Changes**:
    *   **`load_keep_display() -> bool`** reads the `DWORD` `KeepDisplayOn`
        under `HKCU\Software\Hold`. A missing value or a read error means
        `true`.
    *   **`save_keep_display(bool)`** creates the key if needed and writes 1 or
        0. A write error is ignored; the setting then only lasts until Hold
        exits.

#### 18. `scripts/presence-spike.ps1` (Create)

*   **Type / Scope**: Manual test tool for build step 1. Not shipped.
*   **Proposed Changes**:
    *   Uses `Add-Type` to declare `SendInput`, `GetLastInputInfo` and
        `GetTickCount`.
    *   Takes `-Style Zero|Pair`, `-Pixels N`, `-Minutes M`.
    *   Every 60 seconds, it reads idle time before and after the move and
        prints both with a timestamp, so the log shows whether the move reset
        the clock.
    *   The tester watches Slack from a second device during the run. The
        README lists the control run, the test run and the elevated-window run.

#### 19. `choco/hold.nuspec` (Create)

*   **Type / Scope**: Chocolatey package manifest.
*   **Proposed Changes**:
    *   Id `hold`, title "Hold".
    *   The version is replaced from the git tag by the release workflow.
    *   Authors "Darryl Wright".
    *   `projectUrl`, `licenseUrl`, `projectSourceUrl` and `releaseNotes` point
        at the GitHub repo.
    *   Tags `keep-awake caffeine sleep tray`.
    *   Summary and description in plain English. The description states that
        Hold sends synthetic mouse input.
    *   The files element includes `tools\**`.

#### 20. `choco/tools/chocolateyInstall.ps1` (Create)

*   **Type / Scope**: Package install script.
*   **Proposed Changes**:
    *   The exe is embedded in the package, which the community feed prefers
        to a download.
    *   Sets `HKLM:\Software\Microsoft\Windows\CurrentVersion\Run`, value
        `Hold`, to the full quoted path of `$toolsDir\hold.exe`.
    *   Does not start Hold.
    *   Prints one line: "Hold starts at next login, or type `hold` to start
        it now."

#### 21. `choco/tools/chocolateyBeforeModify.ps1` (Create)

*   **Type / Scope**: Package script, run before upgrade and uninstall.
*   **Proposed Changes**:
    *   Stops every process named `hold` whose path is under the package's
        tools folder, with `-Force`.
    *   Stopping the process also releases its keep-awake request.

#### 22. `choco/tools/chocolateyUninstall.ps1` (Create)

*   **Type / Scope**: Package uninstall script.
*   **Proposed Changes**: Removes the `Hold` value from the `HKLM` Run key, if
    it is there.

#### 23. `choco/tools/hold.exe.gui` (Create)

*   **Type / Scope**: Empty marker file.
*   **Proposed Changes**: Tells Chocolatey the `hold` shim launches a GUI app
    and should not wait for it to exit.

#### 24. `choco/tools/VERIFICATION.txt`, `choco/tools/LICENSE.txt` (Create)

*   **Type / Scope**: Community feed requirements for packages that embed
    binaries.
*   **Proposed Changes**:
    *   `VERIFICATION.txt` gives the GitHub Release download URL and the SHA256.
        The release workflow fills in both.
    *   `LICENSE.txt` is a copy of the MIT licence.

#### 25. `.github/workflows/ci.yml` (Create)

*   **Type / Scope**: CI on push and pull request.
*   **Proposed Changes**: On `windows-latest`, run:
    *   `cargo fmt --check`
    *   `cargo clippy --release -- -D warnings`
    *   `cargo test`
    *   `cargo build --release`
    *   a check that `target/release/hold.exe` is under 1 MB

#### 26. `.github/workflows/release.yml` (Create)

*   **Type / Scope**: Release on tag `v*`.
*   **Proposed Changes**: On `windows-latest`:
    1.  Build release and run the tests.
    2.  Compute the SHA256 of `hold.exe`.
    3.  Create the GitHub Release for the tag, with `hold.exe` and
        `hold.exe.sha256`.
    4.  Copy the exe into `choco/tools/`, then write the version, URL and
        hash into the nuspec and `VERIFICATION.txt`.
    5.  `choco pack`.
    6.  `choco push` to `https://push.chocolatey.org/` with the secret
        `CHOCOLATEY_API_KEY`.
    *   If the push fails, the GitHub Release is left in place. Re-running the
        job from step 4 publishes the package.

#### 27. `README.md` (Create)

*   **Type / Scope**: User documentation.
*   **Proposed Changes**: Covers:
    *   What Hold does, including that it sends synthetic mouse input.
    *   How to install with `choco install hold`, and how to start it now with
        `hold`.
    *   The two icons and the menu.
    *   The 10% battery rule.
    *   How to pin the icon in the Windows 11 tray.
    *   Limits: lid close, the lock screen, policy, and elevated windows.
    *   How to delete the leftover `HKCU\Software\Hold` key.
    *   "Tested behaviour", with the step-1 results.
    *   The icon export command.
    *   How to build and release.

#### 28. `LICENSE` (Create)

*   **Type / Scope**: Licence.
*   **Proposed Changes**: MIT, copyright "2026 Darryl Wright".

#### 29. `.gitignore` (Create)

*   **Type / Scope**: Git configuration.
*   **Proposed Changes**: Ignores `target/`, `choco/*.nupkg` and
    `choco/tools/hold.exe`.

#### 30. `docs/designs/hold.md` (Create)

*   **Type / Scope**: Design record.
*   **Proposed Changes**: The final copy of this design document.

### Verification Plan

*   **Compilation/Build**: `cargo build --release`. The exe is under 1 MB and
    has no `vcruntime140.dll` import, checked with `dumpbin /dependents`.
*   **Unit Tests**: `cargo test`. Every rule in `core.rs` has a test (see file
    11).
*   **Static Analysis & Linting**: `cargo fmt --check` and
    `cargo clippy -- -D warnings`.
*   **Test Coverage Requirement**: every row of the event table and every
    battery rule has at least one unit test. There is no percentage target,
    because the Win32 wrappers are only tested by hand.
*   **Manual / E2E**:
    *   **G1:** `powercfg /requests` with Hold on, off, and after Quit. Then
        the battery test: run `powercfg /change standby-timeout-dc 1` and
        `monitor-timeout-dc 1`, unplug for 10 minutes, and check that the
        machine is awake and the System log has no Kernel-Power event 42.
    *   **G2:** the Slack test with a control run, a test run and an
        elevated-window run, watched from a second device. Record the Slack
        version.
    *   **G3:** uncheck "Keep display on" and the display turns off while the
        system stays awake. The setting survives a reboot.
    *   **G4:** CUJ 4 on a laptop.
    *   **G5:** `taskkill /f /im explorer.exe`, then start Explorer, and the
        icon comes back. A second `hold.exe` exits.
    *   **G6:** CUJ 5 on a clean Windows VM. After uninstall, `Get-Process hold`
        finds nothing, the `HKLM` Run value is gone, and
        `C:\ProgramData\chocolatey\lib\hold` is gone.
    *   **G7:** Task Manager shows Hold's memory at idle under 5 MB.

*Drafted with EGM*
