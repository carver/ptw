# Issues

Open work that is not a design decision (those go in `docs/adr/`). Local
until the repo is on GitHub, then these move to `gh issue`. Numbers are
permanent: a new entry takes the Next ID below (bump it) and goes at the
bottom of its section. Delete an entry when it is done, git remembers;
leave the gap, never renumber.

Next ID: 16

## Host bring-up

1. **The Key proxy has only done a plain Hold on a real keyboard.**
   ADR 0006 (everything else on the first host pass is verified as of
   2026-08-29, audio cues and autorepeat included; a chord held across
   a restart waited 5.5 s for the grab and left z alive, 2026-09-17).
   Still to check: a second keyboard plugged in while running. Quit
   looks safe by reading mutter and libinput:
   removing the proxy device releases its keys, and a release with no
   press is ignored. Alt+click can only lose its Alt inside the
   Deferral, so it is a thing to notice in use, not to test.

## Correction and hold-back

3. **Thresholds are set from five recordings.** ACCEPT_BELOW 0.15 and the
   ×0.3 phonetic boost were tuned to fix "career"→"Carver" on one
   sample; CMUdict homophones ("clawed"→"Claude") no longer lean on the
   boost. Revisit with a week of real dictation; keep a list of
   misfires. Words the dictionary lacks ("clod") still ride the
   Metaphone knife edge at exactly 0.15.
4. **Names still wrong on the samples:** a neighbourhood (sometimes),
   a first name, "ptw".

## Engine

5. **160 ms lookahead stays opt-in.** It costs 2–2.5x CPU and the commit
   lag gain is under half a second. Document in the settings dialog when
   a low-latency mode is worth it.
6. **sherpa-onnx as a second Engine** once its hotwords PR
   (k2-fsa/sherpa-onnx#3895) merges: model-side biasing for custom words
   would beat post-hoc Correction.
7. **Moonshine v2** only if a faster x86 build appears; 2–4x slower and
   less accurate on our samples (see `docs/research/engine-benchmark.md`).
8. **Benchmark reference transcripts** in `target/bench/` were not
   proofread; WER numbers in the benchmark report are approximate.

## Typing and hotkey

9. **Per-character portal latency.** Each keysym is a D-Bus round trip.
   If it shows in use, batch through libei or type words per call.
10. **Chord keys pressed in the wrong order leak.** Pressing z before Alt
    types a z, because only modifier presses are deferred (ADR 0006).
11. **Toggle mode has no UI.** `ptw toggle` over D-Bus works; nothing
    exposes it.
15. **`ptw doctor` cannot see a key the compositor holds down.** The dead
    z took a wizard to place: the kernel showed nothing down, only a
    throwaway window's `wl_keyboard.enter` (under `WAYLAND_DEBUG=1`)
    showed the compositor holding one key. A doctor line that opens such
    a window and reports the count, with the stop-ptw-then-press-once
    cure, turns the next one into a one-liner.

## Packaging

12. **`.deb` package** so setup is `apt install` plus `ptw setup`. It
    could also ship the udev rule and a login hook, so joining `input`
    stops being a documented relogin step (`ptw doctor`'s `service` line
    and the daemon both explain it today).
13. **Hotkey capture in the settings dialog.** Today you type the chord
    name and it is validated live.
