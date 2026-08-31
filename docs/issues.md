# Issues

Open work that is not a design decision (those go in `docs/adr/`). Local
until the repo is on GitHub, then these move to `gh issue`. Numbers are
permanent: a new entry takes the Next ID below (bump it) and goes at the
bottom of its section. Delete an entry when it is done, git remembers;
leave the gap, never renumber.

Next ID: 15

## Host bring-up

1. **The Key proxy has only done a plain Hold on a real keyboard.**
   ADR 0006 (everything else on the first host pass is verified as of
   2026-08-29, audio cues and autorepeat included). Still to check: a
   second keyboard plugged in while running, and keys stuck down after
   start or quit. Alt+click can only lose its Alt inside the 150 ms
   Deferral, so it is a thing to notice in use, not to test.
2. **Deferral is a constant (150 ms).** Make it a config knob if anyone
   needs a different trade-off between Alt+click and slow chord presses.
## Correction and hold-back

3. **Thresholds are set from five recordings.** ACCEPT_BELOW 0.15 and the
   ×0.25 phonetic boost were tuned to fix "career"→"Carver" and
   "clawed"→"Claude", each on one sample. Revisit with a week of real
   dictation; keep a list of misfires. Known so far: "cloud", "clod" and
   "clout" all become "Claude" (see 14).
4. **Names still wrong on the samples:** a neighbourhood (sometimes),
   a first name, "ptw".
14. **Metaphone cannot tell "clawed" from "cloud"; a pronouncing
    dictionary can.** Double Metaphone drops vowels, so both are KLT and
    the accept threshold has to pick which false result to live with
    (today: "cloud"→"Claude"). CMUdict keeps them apart: "clawed" and
    "claude" are both `K L AO D`, "cloud" is `K L AW D`. Layer it over
    the scorer: both keys in the dictionary and phonemes equal (stress
    ignored) is a homophone, accept; both in but unequal, drop the
    phonetic boost and let plain Levenshtein decide (rejects "cloud" at
    0.33); either out — names usually are — falls back to Metaphone as
    today. Multi-word runs concatenate per-word phonemes when every word
    is in. `cmudict-fast` on crates.io; a few MB of data, no desktop
    deps, testable in the sandbox. Smaller than and independent of
    model-side biasing (6), which is still the better endpoint.

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

## Packaging

12. **`.deb` package** so setup is `apt install` plus `ptw setup`. It
    could also ship the udev rule and a login hook, so joining `input`
    stops being a documented relogin step (`ptw doctor`'s `service` line
    and the daemon both explain it today).
13. **Hotkey capture in the settings dialog.** Today you type the chord
    name and it is validated live.
