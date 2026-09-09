---
status: accepted
supersedes: the first consequence of 0003
---

# Grab the keyboard and forward keys through a virtual one

ADR 0003 accepted that the Hotkey leaks to the focused app rather than build a keyboard proxy. The first run on the host (2026-08-29) showed the leak is not the problem; the *held modifier* is. While Alt+z is held, the compositor believes Alt is down and combines it with every keystroke the Typist sends: letters became Alt+letter shortcuts and spaces opened GNOME's window menu. No text can stream through a compositor that thinks a modifier is down.

The ways out were a Hotkey the compositor ignores (Pause, F13: scarce on laptops), a Toggle mode (two taps, no push-to-talk), or making the chord invisible: grab every keyboard with `EVIOCGRAB` and re-emit everything except the chord through a uinput keyboard. We chose the grab to keep hold-to-talk on an ordinary chord.

A chord modifier press cannot be forwarded on arrival: until the next key, Alt is either the start of Alt+z or of Alt+Tab. The Key proxy defers it for a moment (the Deferral). A following non-chord key forwards it immediately, so Alt+Tab is unaffected; the chord key swallows it; the deadline forwards it, and if the chord completes after that the proxy sends a synthetic Alt release so typing is still clean. The one casualty is Alt+click inside the window, which loses its Alt.

The Deferral started at 150 ms on the belief that chords land within 100 ms. On the host (2026-09-09) the chord keys landed 225-255 ms apart often enough to look random, and each slow chord reached Firefox as a bare Alt press and release, which it takes for a menu bar toggle. The default is now 500 ms and `deferral_ms` in the config overrides it. A longer window costs only Alt+mouse gestures started within it; Alt+key chords are never delayed.

## Consequences

- Needs write access to `/dev/uinput`; `ptw setup` installs a udev rule for the `input` group. Without it ptw falls back to ungrabbed reading, where text waits for the modifiers to come up (typed at release, like Handy).
- A hung ptw could hold the keyboard. Mitigations: the reader thread that owns the grab also runs the proxy and writes to uinput, so the daemon loop, the portal, the tray and the Engine are not on the key path; a reader crash drops the device and the kernel releases the grab. Killing the daemon always frees the keyboard.
- The grab waits until every key is up, so a release the compositor is waiting for is never swallowed.
- Chord keys pressed in the wrong order (z before Alt) reach the app before the chord is recognised; only modifier presses are deferred, because deferring letters would delay all typing.
- The chord never reaches the apps, so lone modifiers (`RightCtrl`) are now usable Hotkeys without flashing menus, at the price of that modifier.
