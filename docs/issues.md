# Issues

Open work that is not a design decision (those go in `docs/adr/`). Local
until the repo is on GitHub, then these move to `gh issue`. Newest at the
bottom of each section; delete an entry when it is done, git remembers.

## Host bring-up

1. **Untested on the host:** the settings dialog, the systemd unit, the
   audio cues, and the tray's red disc (fixed 2026-08-29: the theme icon
   name was hiding the pixmap). Hotkey, capture, portal typing and
   streaming were verified on 2026-08-29.
2. **The Key proxy has not run on a real keyboard.** ADR 0006. Things
   to watch for: keys stuck down after start or quit, the Deferral
   biting Alt+click, autorepeat behaviour of forwarded keys, a second
   keyboard plugged in while running, and whether `ptw setup`'s udev rule
   takes effect without a relogin.
3. **Deferral is a constant (150 ms).** Make it a config knob if anyone
   needs a different trade-off between Alt+click and slow chord presses.
4. **Thread count on the host.** In the sandbox 2 threads beat 8 by
   1.7–1.9x, which is surprising enough to re-measure with real use.
   Default stays 2 until then.

## Correction and hold-back

5. **Multi-word names the model splits further.** "Bernal Heights" came
   out as "burn all heights" (three words); the n-gram window only covers
   the entry's two. Options: allow up to `max_words + 1` when the extra
   word is short, or match on a joined phonetic key.
6. **Thresholds are set from five recordings.** ACCEPT_BELOW 0.15 and the
   ×0.3 phonetic boost were tuned to fix "career"→"Carver" on one sample.
   Revisit with a week of real dictation; keep a list of misfires.
7. **Names still wrong on the samples:** a neighbourhood (sometimes),
   a first name, "ptw".

## Engine

8. **160 ms lookahead stays opt-in.** It costs 2–2.5x CPU and the commit
   lag gain is under half a second. Document in the settings dialog when
   a low-latency mode is worth it.
9. **sherpa-onnx as a second Engine** once its hotwords PR
   (k2-fsa/sherpa-onnx#3895) merges: model-side biasing for custom words
   would beat post-hoc Correction.
10. **Moonshine v2** only if a faster x86 build appears; 2–4x slower and
    less accurate on our samples (see `docs/research/engine-benchmark.md`).
11. **Benchmark reference transcripts** in `target/bench/` were not
    proofread; WER numbers in the benchmark report are approximate.

## Typing and hotkey

12. **Per-character portal latency.** Each keysym is a D-Bus round trip.
    If it shows in use, batch through libei or type words per call.
13. **Chord keys pressed in the wrong order leak.** Pressing z before Alt
    types a z, because only modifier presses are deferred (ADR 0006).
14. **Toggle mode has no UI.** `ptw toggle` over D-Bus works; nothing
    exposes it. See 2.

## Packaging

15. **`.deb` package** so setup is `apt install` plus `ptw setup`.
16. **Hotkey capture in the settings dialog.** Today you type the chord
    name and it is validated live.
