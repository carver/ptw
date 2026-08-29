# ptw

Push-to-talk dictation for Linux. Read `CONTEXT.md` for the vocabulary
(Hold, Dictation, Committed text, Hold-back, Typist) and use it in code and
prose. `docs/adr/` holds the decisions that look odd without their reasons;
`docs/planning-log.md` the design interview; `docs/research/` the sources;
`docs/issues.md` the open work (local until the repo is on GitHub).

## Where things go

- `crates/ptw-core`: no desktop deps; every piece of logic with a decision
  in it lives here with tests (proptest for anything streaming).
- `crates/ptw-engine-*`: one crate per Engine runtime.
- `crates/ptw`: daemon, typists, tray, CLI, settings dialog. Thin.

## Testing without a desktop

The dev sandbox has no microphone, keyboard, display or session D-Bus.
The Engine path is fully testable here:

```sh
cargo test --workspace
PTW_TEST_MODEL=target/bench/models/nemotron-speech-streaming-en-0.6b-Q8_0.gguf \
PTW_TEST_WAV=target/bench/sample1.wav cargo test -p ptw-engine-transcribe -- --nocapture
ptw --config <cfg> transcribe --realtime target/bench/sample3.wav
```

`target/bench/` (gitignored, on the host mount) has the model, 16 kHz
copies of the user's recordings from `tests/data/`, and the benchmark
harness. Keep models and big builds there: the sandbox root is a 20 GB
overlay that has filled up once.

`ptw daemon`, `ptw doctor`, the tray and typing need the host. Hand the
user exact commands and ask for the output.

## Build deps

`cmake`, a C++ toolchain, `pkg-config`, `libasound2-dev`, `libxkbcommon-dev`.
transcribe.cpp compiles from the crate in about a minute.

## Working with the user

They commit in this tree while you work: `git status` before committing,
and read a shared file before writing it. Commit in logically isolated
pieces with the repo's git identity.
