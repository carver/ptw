---
status: accepted
---

# Read /dev/input directly for the Hotkey

The target is Ubuntu 24.04 (GNOME 46 on Wayland). Detecting key *release* globally there has exactly one route: reading `/dev/input/event*` with evdev, which needs `input` group membership. The GlobalShortcuts portal (press and release, no special permissions) only gained a GNOME backend in GNOME 48, gnome-shell's grab API is closed to third-party apps, and gsettings custom shortcuts fire on press only. evdev also works on X11, other compositors and the console, so it doubles as the fallback everywhere.

## Consequences

- Reading does not consume the key, so the Hotkey also reaches the focused app. We accept this and choose a default Hotkey that is harmless in common apps, rather than building a grab-and-reemit keyboard proxy (which needs uinput and turns a ptw crash into a dead keyboard).
- `input` group membership lets any process running as the user read every keystroke; `ptw setup` says so.
- When the host moves to GNOME 48+, the GlobalShortcuts portal should become the preferred path behind a runtime check.
