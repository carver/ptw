---
status: accepted
---

# Build ptw instead of using Handy

Handy (https://github.com/cjpais/Handy) is the closest existing tool and the feel we want, and since 0.9.0 it runs streaming Engines. But as of 2026-08-25 its streaming output only feeds a preview overlay; text reaches the focused app on release, by clipboard paste. On GNOME Wayland the overlay is a plain window that steals focus (so Handy disables it), hold-to-talk needs a gsettings toggle shortcut instead, and pasting needs ydotool/dotool. The one thing we want most, words typed as they are committed while still talking, is not there. We build ptw, and borrow Handy's proven choices (evdev hotkeys, Nemotron/Moonshine streaming models, n-gram fuzzy Correction of Custom words).
