# Engine benchmarks

The harness behind `docs/research/engine-benchmark.md` and ADR 0005. Code and
result logs live here; models, prebuilt libraries, audio and build output live
in `target/bench/` (gitignored, on the host mount) so nothing large enters git.

```
bench/
  sherpa-bench/   sherpa-onnx streaming and offline runs (Rust)
  tcpp-bench/     transcribe.cpp streaming runs (Rust)
  moonshine/      Moonshine v2 runs (C) plus the fake-CPU-count shim
  run_all.sh      the matrix: main and keyterms phases
  run_extras.sh   2-thread runs, repeats, Moonshine diagnostics
  run_beam.sh     greedy vs modified_beam_search vs beam+hotwords (sherpa-onnx)
  analyze.py      JSON logs to the markdown tables in the report
  keyterms.txt    the Custom words used for biasing runs (local only, gitignored)
  results/        one JSON per run and file; offline/ holds the Parakeet reference
```

## Data

```
target/bench/
  sample1.wav .. sample5.wav   tests/data/*.flac resampled to 16 kHz mono
  models/                      sherpa-onnx int8 exports, the GGUF, Moonshine, Parakeet
  libs/                        extracted sherpa-onnx static archives, source builds
  cargo/                       one cargo target dir per harness
```

Resample with `ffmpeg -i "tests/data/ptw sample 1.flac" -ar 16000 -ac 1 target/bench/sample1.wav`.
Samples 1 to 4 are in git. Sample 5, the keyterms list and every result
that quotes it stay on the host, gitignored, so the `sample5` columns in the
tables cannot be regenerated from a clean checkout.

## Building

The harness crates are excluded from the workspace, so build them from their own
directory with a target dir under `target/bench/cargo`.

sherpa-onnx: the crate's build script downloads a prebuilt static archive from
GitHub. Behind a TLS-intercepting proxy that download fails (rustls does not
trust the injected CA), so fetch the archive with curl and point the build
script at it:

```sh
mkdir -p target/bench/libs/archives
curl -sSL -o target/bench/libs/archives/sherpa-onnx-v1.13.8-linux-x64-static-lib.tar.bz2 \
  https://github.com/k2-fsa/sherpa-onnx/releases/download/v1.13.8/sherpa-onnx-v1.13.8-linux-x64-static-lib.tar.bz2
cd bench/sherpa-bench
SHERPA_ONNX_ARCHIVE_DIR=$PWD/../../target/bench/libs/archives \
  cargo build --release --target-dir ../../target/bench/cargo/sherpa-bench-beam
```

The archive version must match the `sherpa-onnx` version in `Cargo.toml`.
`SHERPA_ONNX_LIB_DIR=<dir with lib*.a>` links an extracted or self-built copy
instead. Do not build sherpa-onnx from source with the host compiler: its
prebuilt onnxruntime comes from manylinux2014 (GCC 11) and the mix aborts at
recognizer creation. Build inside `quay.io/pypa/manylinux2014_x86_64` as
upstream's CI does, or wait for a release.

transcribe.cpp compiles from the crate (cmake, C++ toolchain, about a minute):

```sh
cd bench/tcpp-bench
cargo build --release --target-dir ../../target/bench/cargo/tcpp-bench
```

Moonshine needs the `moonshine-voice-linux-x86_64` tarball from its releases:

```sh
cd bench/moonshine
gcc -O2 bench.c -I<tarball>/include -L<tarball>/lib -lmoonshine -Wl,-rpath,<tarball>/lib -o bench
gcc -O2 -shared -fPIC nproc_shim.c -o nproc_shim.so -ldl
```

`nproc_shim.so` fakes the CPU count so onnxruntime sizes its thread pool;
`run_all.sh` sets `FAKE_NPROC` and expects a `fakesys<n>` directory per count
(a tree that mirrors `/sys/devices/system/cpu` under the same path,
trimmed to n CPUs; the shim prefixes every read of that path with it).

## Running

Each script names its binaries and models at the top. `run_all.sh main`,
`run_all.sh keyterms`, `run_extras.sh` and `run_beam.sh [threads]` write JSON
into `results/<phase>/`; `analyze.py [results dirs...]` prints the tables.
