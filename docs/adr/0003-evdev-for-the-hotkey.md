---
status: accepted
---

# Read /dev/input directly for the Hotkey

The target is Ubuntu 24.04 (GNOME 46 on Wayland). Detecting key *release* globally there has exactly one route: reading `/dev/input/event*` with evdev, which needs `input` group membership. The GlobalShortcuts portal (press and release, no special permissions) only gained a GNOME backend in GNOME 48, gnome-shell's grab API is closed to third-party apps, and gsettings custom shortcuts fire on press only. evdev also works on X11, other compositors and the console, so it doubles as the fallback everywhere.

## Consequences

- Reading does not consume the key, so the Hotkey also reaches the focused app. We accept this and choose a default Hotkey that is harmless in common apps, rather than building a grab-and-reemit keyboard proxy (which needs uinput and turns a ptw crash into a dead keyboard).
- Found on first host run (2026-08-29): the compositor keeps seeing the chord's modifier for as long as it is held, and combines it with everything the Typist sends meanwhile. Every letter became Alt+letter and every space Alt+Space, GNOME's window menu. So the Typist holds text back while any modifier is down. With a chord like Alt+z that means the text lands at release; streaming while you talk needs a Hotkey the compositor never sees as a modifier: a lone non-modifier key, a toggle, or the proxy declined above. Decision pending in `docs/issues.md`.
- `input` group membership lets any process running as the user read every keystroke; `ptw setup` says so.
- When the host moves to GNOME 48+, the GlobalShortcuts portal should become the preferred path behind a runtime check.
