# ptw (Push to Whisper)

Hold a key, talk, and the words appear in whatever app has focus while you
are still talking. Linux-first, CPU-only, offline. Think
[Handy](https://github.com/cjpais/Handy), but streaming: text is typed as
the recognizer commits it, not pasted when you let go.

Status: pre-alpha, built for one machine (Ubuntu 24.04, GNOME 46 on
Wayland). Vocabulary is in `CONTEXT.md`; the reasoning behind the odd
choices is in `docs/adr/`; the interview that started it is in
`docs/planning-log.md`.

## How it works

- The Hotkey (default `Alt+z`, in your keyboard layout) is read straight
  from `/dev/input`, because that is the only way to see a key *release*
  on GNOME 46 (ADR 0003). ptw grabs the keyboard and passes every other
  key on through a virtual one, so the desktop never sees the chord and
  text can stream while Alt is held (ADR 0006). Any other key pressed
  during a hold cancels the dictation.
- Audio goes to a streaming Engine on the CPU. Only text the Engine has
  committed gets typed; nothing typed is ever retracted (ADR 0002).
- Typing goes through the XDG RemoteDesktop portal, which asks once and
  remembers (ADR 0004). A uinput keyboard is the fallback for other
  compositors.
- Custom words (names, jargon) are fixed by fuzzy matching with a
  phonetic boost, so "Kaitlin" becomes "Caitlyn". Words are held back
  just long enough for a multi-word entry to match.
- A tray icon shows a red dot while listening and opens the settings.

## Build

```sh
sudo apt install build-essential cmake pkg-config libasound2-dev libxkbcommon-dev
cargo install --path crates/ptw
```

The Engine is Nemotron Speech Streaming EN 0.6B on transcribe.cpp (ADR
0005): about 1 GB of RAM, 0.3x real time on two threads of an Alder Lake
laptop, a word committed roughly half a second after you say it.

## Set up

```sh
sudo usermod -aG input $USER   # read keyboards; then log out and back in:
                                # systemd user services only get login-time groups
ptw setup                       # config, 700 MB model, udev rule, systemd unit
systemctl --user enable --now ptw
ptw doctor                      # what works, what does not
```

The first hold pops a GNOME dialog asking to allow remote control. Say
yes once; ptw saves the token.

`ptw settings` opens the dialog. The config is plain TOML at
`~/.config/ptw/config.toml` and the daemon reloads the hotkey and custom
words when the file changes. A desktop shortcut bound to `ptw toggle`
gives a tap-to-start, tap-to-stop mode with no `input` group needed.

## Develop

```sh
cargo test --workspace
ptw transcribe --realtime some.wav   # the daemon's text path, no microphone
```

`crates/ptw-core` has no desktop dependencies and holds everything with
interesting logic: hotkey chords, the key proxy, correction, hold-back,
the dictation loop, the resampler. `crates/ptw` is the daemon, typists, tray, CLI and
settings dialog. Recordings for tuning live in `tests/data/` and stay
out of git.
