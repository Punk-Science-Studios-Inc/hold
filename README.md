# Hold

A small Windows tray app that stops the computer from sleeping while you are
away.

| Icon | Meaning |
| --- | --- |
| Open eye | Hold is on. The machine stays awake. |
| Closed eye | Hold is off. Windows' own sleep settings apply. |

* **Left-click** the icon to switch Hold on or off.
* **Right-click** for:
  * **Keep display on** (checked by default). Uncheck it to let the display
    turn off while the machine stays awake. The choice is remembered.
  * **Quit**.

Hold starts at login and is on when it starts.

## Install

```powershell
choco install hold
```

Hold starts at your next login. To start it now, type `hold` in a terminal.

On Windows 11, new tray icons go to the hidden overflow area (the `^` arrow).
To keep the eye visible, open **Settings > Personalization > Taskbar > Other
system tray icons** and turn on **Hold**.

## What it does

* **Keeps the machine awake.** While on, Hold asks Windows not to sleep
  (`SetThreadExecutionState`). With "Keep display on" checked, it also keeps
  the display on.
* **Looks active.** Once you have been idle for 50 seconds, Hold sends a
  synthetic mouse move of zero distance about once a minute. The pointer does
  not move, but Windows counts it as input and resets its idle timer, so apps
  that watch for idle time see you as present.
* **Low battery.** On battery, when the charge drops below 10%, Hold turns
  itself off and shows a notification. When you plug in, it turns back on. If
  you turned it off yourself, it stays off.

Hold makes no network calls. It stores one value:
`HKCU\Software\Hold\KeepDisplayOn`.

## Limits

* **Lid close** follows the Windows lid setting. Hold does not override it.
* **Lock screen:** once the session locks, synthetic input no longer reaches
  your desktop, and chat apps may show you as Away.
* **Policy:** a lock or sleep timeout enforced by Group Policy or device
  management wins.
* **Elevated windows:** if an admin window is in front, Windows may drop
  synthetic input from Hold.
* **Every user:** Chocolatey installs the start-at-login entry for every user
  on the machine.

## Tested behaviour

Measured on Windows 11 with `scripts/presence-spike.ps1`:

* A zero-distance mouse move resets Windows idle time (about 10,000 ms back to
  about 60 ms), and the pointer does not move.
* A +1/−1 pixel pair also resets idle time, but left the pointer 1 pixel off
  with "Enhance pointer precision" on. Hold uses the zero-distance move.
* Not tested: whether Slack or other chat apps keep you Active.

## Uninstall

```powershell
choco uninstall hold
```

This stops Hold and removes the start-at-login entry. To remove the remembered
display setting as well:

```powershell
Remove-Item HKCU:\Software\Hold -Recurse
```

## Build

Needs Rust (the version is pinned in `rust-toolchain.toml`) and the Visual
Studio C++ Build Tools.

```powershell
cargo test
cargo build --release   # target\release\hold.exe
```

The icons are drawn by `scripts/make-icons.py` (needs Pillow):

```powershell
python scripts/make-icons.py
```

## Release

1. Set `version` in `Cargo.toml` and commit.
2. Tag and push: `git tag v0.1.0; git push origin v0.1.0`.

The release workflow builds the exe, attaches it and its SHA256 to a GitHub
Release, then packs and pushes the Chocolatey package. It needs the repository
secret `CHOCOLATEY_API_KEY`. If the push fails, re-run the job; the existing
GitHub Release is reused.

## Design

See [docs/designs/hold.md](docs/designs/hold.md).

## Licence

MIT. See [LICENSE](LICENSE).
