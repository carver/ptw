# Planning log

Record of the design interview that started the project (2026-08-28). Research behind it: `docs/research/asr-backends.md`, `docs/research/desktop-integration.md`. Vocabulary: `CONTEXT.md`. Decisions with trade-offs: `docs/adr/`.

## Facts established

- Host: Ubuntu 24.04, GNOME 46 on Wayland. Intel Alder Lake, 16 threads, AVX2 + AVX-VNNI, no AVX-512, 15 GB RAM. CPU only.
- Development happens in a Docker sandbox that shares the host CPU but has no audio devices, input devices, or display. The Engine is testable here with recorded audio; Hotkey, Typist and tray need the host.
- Handy does not type while you speak (see ADR 0001).
- Best streaming Engine candidates on this CPU: Nemotron Speech Streaming EN 0.6B (via `sherpa-onnx` or `transcribe-cpp`) and Moonshine v2 (C API FFI). Whisper-family, Kyutai, Voxtral, Vosk, WhisperLive and SimulStreaming rejected; reasons in the research report.
- A streaming transducer commits tokens monotonically, so typing only Committed text costs roughly the Lookahead, not seconds.

## Decisions

| Topic | Decision |
|---|---|
| Target | GNOME 46 Wayland on Ubuntu 24.04. X11 only where free. |
| Language | Rust binary; C/C++ libs linked in are fine; Python only if it clearly wins (it did not). |
| Iteration | Engine tested in the sandbox with the user's recordings in `tests/data/`; host-only parts driven by the user via `cargo run` and `ptw doctor`. |
| Streaming output | Type only Committed text (ADR 0002). Emit whenever Committed text grows, coalesced per decode tick. |
| Hotkey | Hold only in v1; `ptw toggle` verb shipped for a gsettings shortcut, no UI for it. On release: stop capture, Flush, idle. Mechanism: evdev (ADR 0003). Default Alt+z (harmless in shells and browsers; VS Code binds it to word wrap, which the user does not use). Any other key pressed during a Hold aborts the Dictation, so a lone modifier can be chosen as the Hotkey without chords triggering it. |
| Typist | RemoteDesktop portal, uinput behind a setting (ADR 0004). Trailing space after each Dictation; Model's own punctuation; no filler removal. |
| Custom words | Word list in the UI. Correction by fuzzy n-gram match (Handy style) with a Hold-back; Engine-level biasing added when the Engine offers it. |
| Engine | Behind a trait. Nemotron Speech Streaming EN 0.6B Q8_0 on transcribe.cpp, 560 ms Lookahead, 2 threads (ADR 0005, from `docs/research/engine-benchmark.md`). 160 ms stays a setting. Moonshine dropped; sherpa-onnx waits for its hotwords PR. |
| Models | Downloaded by `ptw setup` with checksum into `~/.local/share/ptw/models/`; daemon refuses to start without one. |
| Daemon | Resident, Model preloaded, ~0% idle CPU, systemd user service. One process: hotkey listener, audio capture, Engine, Typist, tray, D-Bus. |
| Feedback | Tray icon state plus quiet audio cues on start/stop (on by default). Live-text notification banner as an off-by-default setting. No overlay on GNOME. |
| Settings | TOML at `~/.config/ptw/config.toml` is the source of truth; egui/eframe dialog in a `ptw settings` subprocess edits it over D-Bus. |
| Audio | cpal (PipeWire) resampled to 16 kHz mono; default source. |
| Distribution | Build from source; structure for a `.deb` later. MIT. Binary `ptw`. |
| Code layout | Cargo workspace: a host-independent engine crate and a desktop crate. |

## Open

- Nothing from the interview. Next: run it on the host and tune thresholds and thread count with real use.
