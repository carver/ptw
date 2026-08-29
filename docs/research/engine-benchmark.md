# Streaming Engine benchmark on the ptw dev box

Measured 2026-08-29 inside the dev sandbox, which shares the host CPU (Intel Alder Lake, 16 hardware threads visible, AVX2 + AVX-VNNI, no AVX-512, 15 GB RAM, no GPU). Three CPU streaming Engines, one Model family each, fed the five `tests/data/ptw sample N.flac` recordings in 80 ms chunks. Follows on from [asr-backends.md](asr-backends.md).

## Short version

- **Nemotron Speech Streaming EN 0.6B at 560 ms Lookahead is the only configuration that keeps up with real time here with headroom, and it does so on 2 threads**: RTF 0.30 in sherpa-onnx and 0.31 in transcribe.cpp, p95 chunk cost 180 ms, Flush 160 to 220 ms, ~1 GB RSS. Both runtimes commit greedy RNN-T output directly: Committed text was never revised in any run (0 revisions over about 3,900 commit events), and neither produces Tentative text at all.
- **More threads are slower on this box.** 8 threads is 1.7 to 1.9x slower than 2 for both Nemotron runtimes; 4 is in between. Treat thread count as a setting (default 2) and re-measure on the host.
- **The 160 ms Lookahead export costs 2 to 2.5x the 560 ms one** (RTF 0.61 transcribe.cpp, 0.77 sherpa-onnx at 2 threads) and only shaves ~0.16 s off the median commit lag (0.64 s vs 0.80 s from word onset). Not worth it on this CPU.
- **Moonshine v2 via the official C API cannot run in real time here.** Medium is RTF 1.2 on 4 CPUs (0.9 to 1.9 depending on thread count), small is 0.56, and the cost barely changes between 1 and 4 CPUs. Each update re-decodes the whole in-progress line, so cost grows with line length (0.5 to 1.6 s per update on medium). Accuracy on these files is also well behind Nemotron (25 to 30% WER vs the Parakeet reference, against 13 to 16%). Keyterm biasing fixed capitalisation of listed names ("Handy", a first name, "the Mission", "Gnome shell") but not the two neighbourhoods or "ptw", and cost nothing measurable.
- transcribe.cpp's output was a little closer to the offline reference than sherpa-onnx's (13.3 to 14.1% vs 15.6% WER); both are the same Model, so the difference is the export (Q8_0 GGUF vs int8 ONNX) and the runtimes' handling of the final partial chunk.
- OpenBLAS made no measurable difference to transcribe.cpp (RTF 0.48 without vs 0.53 with, at 8 threads, inside run-to-run noise): the encoder dominates, not the decoder.

## Method

**Audio.** The five FLACs (stereo 44.1 kHz) were converted with `ffmpeg -ac 1 -ar 16000 -sample_fmt s16` to 16 kHz mono 16-bit WAV. Durations: sample 1 15.36 s, sample 2 26.02 s, sample 3 24.87 s, sample 4 26.54 s, sample 5 53.19 s; 146 s total.

**Feeding.** Every harness loads the Model once, then for each file feeds 1280-sample (80 ms) chunks in a tight loop with no sleeping, timing each chunk's feed + decode + result read with a monotonic clock. Compute time per chunk is reported as mean, p95 and max over the file; RTF is total compute divided by audio duration. After the last chunk the harness calls the Engine's end-of-input path and times it (the Flush number):

- sherpa-onnx: `accept_waveform` of 0.5 s of silence, `input_finished`, then `decode` while `is_ready`. Without the silence the online NeMo path never decodes the last partial chunk (measured Flush 0 ms and the transcript stopped mid-word), so the padding is part of the Flush cost here, as it is in sherpa-onnx's own examples.
- transcribe.cpp: `Stream::finalize`.
- Moonshine: `moonshine_stop_stream` then one `moonshine_transcribe_stream`.

**Committed vs Tentative.** After every chunk the harness reads the Engine's committed string and logs each change. A change that is not a pure append is counted as a revision. sherpa-onnx's `OnlineRecognizer` result text is treated as committed (greedy transducer output, nothing else is exposed). transcribe.cpp exposes `committed` and `tentative` directly. For Moonshine, committed is the concatenation of lines with `is_complete = 1` and tentative is the text of incomplete lines.

**Commit lag.** Each committed word is matched in order against the offline Parakeet TDT word timestamps (emission time of the first BPE token of the word, a good proxy for word onset). Lag = audio time at the end of the chunk in which the word appeared minus onset. Median over all matched words in all five files is reported, plus examples.

**Peak RSS** is `VmHWM` from `/proc/self/status` at the end of the process (one process per Engine configuration, five files).

**Reference transcripts** come from sherpa-onnx `OfflineRecognizer` with `sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8` (greedy, 8 threads; RTF 0.09 to 0.16). They are not ground truth: Parakeet drops a whole phrase in sample 3 ("hotkey with release detection on GNOME 46 ... interrupted GNOME 48", which every Nemotron run got), gets "mouth card" in sample 1 and a family name in sample 5, and misses punctuation-independent words elsewhere. **The WER column is therefore rough and biased against the streaming runs on sample 3.** The reference is printed under each sample below for proofreading; once corrected, `analyze.py` recomputes WER from the JSON logs.

**Noise.** The sandbox's 16 vCPUs sit on a hybrid P/E-core host that was also running the user's desktop. `taskset` is a no-op inside the sandbox (CPU% did not change when pinned to 4 CPUs), so nothing could be pinned. Two runs showed multi-second stalls (a 6.9 s chunk in transcribe.cpp R=1 at 8 threads, a 4.5 s Flush in the no-BLAS run) that are host scheduling, not the Engine. Repeating the 4-thread Nemotron runs gave RTF within 0.01 to 0.02 of the first pass, so the medians and p95s are stable; the max column is not.

## Candidates, versions, assets

| | A. sherpa-onnx | B. transcribe.cpp | C. Moonshine v2 |
|---|---|---|---|
| Runtime | `sherpa-onnx` crate 1.13.6 + `sherpa-onnx-sys` 1.13.6, linking the prebuilt `sherpa-onnx-v1.13.6-linux-x64-static-lib.tar.bz2` (bundles onnxruntime 1.27.1, static) | `transcribe-cpp` crate 0.2.2 + `transcribe-cpp-sys` 0.2.2 (vendored transcribe.cpp + ggml, built by cmake from the crate, static, `GGML_NATIVE=ON`, so AVX2/AVX-VNNI kernels) | `moonshine-voice-linux-x86_64.tar.gz` from release v0.1.5 (`libmoonshine.so`, header version 30000, bundled onnxruntime 1.23.2) |
| Model files | `sherpa-onnx-nemotron-speech-streaming-en-0.6b-560ms-int8-2026-04-25.tar.bz2` and `...-160ms-int8-2026-04-25.tar.bz2` (encoder 623 MB int8 ONNX each) | `handy-computer/nemotron-speech-streaming-en-0.6b-gguf` `nemotron-speech-streaming-en-0.6b-Q8_0.gguf` (696 MB; one file covers all Lookaheads via `att_context_right` R = 1 for 80 ms lookahead / 160 ms first-chunk delay, R = 6 for 480 ms / 560 ms) | `https://download.moonshine.ai/model/{medium,small}-streaming-en/quantized_26_08_21/` (medium 397 MB incl. the optional attention decoder, small 136 MB; URLs come from `moonshine_get_stt_dependencies`) |
| Decoding | greedy_search, `OnlineRecognizer`, 1/2/4/8 threads | `CommitPolicy::Auto`, `ParakeetStreamOptions { att_context_right }`, 1/2/4/8 threads | defaults (speculative decoding on, `decode_incomplete_lines` on); also `decode_incomplete_lines=false` and `keyterms` |
| Reference | `sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8.tar.bz2`, `OfflineRecognizer`, `model_type = nemo_transducer` | | |

Toolchain: rustc/cargo 1.98.0, gcc 15.2.0, cmake 4.2.3, ffmpeg 8.0.1, libopenblas-dev 0.3.32, Python 3.14 for analysis.

## Build notes

- **apt**: `ffmpeg cmake libopenblas-dev time pkg-config libclang-dev clang jq`. `libclang-dev`/`clang` turned out unnecessary (no bindgen; the Moonshine harness is plain C against the shipped header).
- **sherpa-onnx crate**: the `-sys` build script downloads the prebuilt archive itself and failed inside the sandbox with `tls connection init failed: invalid peer certificate: UnknownIssuer` (the proxy's CA is not in rustls's store). Workaround: `curl` the same archive and set `SHERPA_ONNX_LIB_DIR=<extracted>/lib`. After that the crate compiled in seconds and the binary is a 36 MB static executable. Nemotron works with `model_type` left unset (auto-detected from the ONNX metadata).
- **transcribe-cpp crate**: cmake build from the crate, 42 s wall / 5m17s CPU on 16 threads, no source changes. It found the system OpenBLAS on its own (`TRANSCRIBE_USE_SYSTEM_BLAS=ON` default). A second build with `TRANSCRIBE_CMAKE_ARGS=-DTRANSCRIBE_USE_SYSTEM_BLAS=OFF` took 53 s. The crate's default `metal` feature is a no-op on Linux. The `Stream` API worked as documented; one surprise is that with the Parakeet cache-aware family the `tentative` string is always empty (the RNN-T greedy path commits every token as it is emitted).
- **Moonshine**: no build; `gcc bench.c -lmoonshine` against the tarball. The library has no thread option and sizes onnxruntime's pool from the sysfs CPU topology, spinning all 16 threads at 1590% CPU. To get 4- and 8-CPU numbers I wrote an `LD_PRELOAD` shim (`nproc_shim.c`) that redirects `open`/`fopen` of `/sys/devices/system/cpu/*` and `/proc/cpuinfo` to a faked N-CPU tree and clamps `sysconf`/`sched_getaffinity`. `sysconf` alone was not enough. Model URLs are not in the docs; a 10-line C program calling `moonshine_get_stt_dependencies("en", {"model_arch": "4"})` returns them. No HF token or gate involved.
- **Disk**: models plus prebuilt libs plus two cargo target dirs came to ~4 GB. The 20 GB sandbox overlay filled once (my fault: a `cp -rL` into sysfs) and the models and target dirs were moved to `target/bench/` on the host mount.

## Results

Chunk = 80 ms of audio. p95 and max are the worst file's values; mean is over all chunks; RTF is over all 146 s. The Nemotron Engines do real work only every 7th chunk (560 ms) or every 2nd (160 ms), so mean is low and p95 is what a chunk that triggers the encoder costs. Flush = end-of-input to final text. WER is against the Parakeet offline reference (see caveat above).

| Engine | Model / Lookahead | threads | chunk mean ms | chunk p95 ms | chunk max ms | RTF | Flush ms mean / max | committed revisions | peak RSS MB | WER % vs ref | median commit lag s |
|---|---|---|---|---|---|---|---|---|---|---|---|
| sherpa-onnx | Nemotron 560 ms int8 | 1 | 48 | 365 | 662 | 0.59 | 308 / 410 | 0 | 945 | 15.6 | 0.80 |
| sherpa-onnx | Nemotron 560 ms int8 | **2** | 24 | **181** | 473 | **0.30** | 161 / 171 | 0 | 972 | 15.6 | 0.80 |
| sherpa-onnx | Nemotron 560 ms int8 | 4 | 35 | 328 | 360 | 0.43 | 227 / 233 | 0 | 933 | 15.6 | 0.80 |
| sherpa-onnx | Nemotron 560 ms int8 | 8 | 47 | 466 | 717 | 0.59 | 254 / 349 | 0 | 952 | 15.6 | 0.80 |
| sherpa-onnx | Nemotron 160 ms int8 | 2 | 61 | 131 | 343 | 0.77 | 379 / 458 | 0 | 967 | 15.6 | 0.64 |
| sherpa-onnx | Nemotron 160 ms int8 | 4 | 120 | 422 | 1281 | 1.48 | 1085 / 1972 | 0 | 938 | 15.6 | 0.64 |
| sherpa-onnx | Nemotron 160 ms int8 | 8 | 178 | 727 | 1496 | 2.24 | 827 / 1357 | 0 | 962 | 15.6 | 0.64 |
| transcribe.cpp | Nemotron Q8_0, R=6 (560 ms) | 1 | 31 | 222 | 337 | 0.38 | 230 / 276 | 0 | 1107 | 14.1 | 0.80 |
| transcribe.cpp | Nemotron Q8_0, R=6 (560 ms) | **2** | 25 | **185** | 286 | **0.31** | 183 / 215 | 0 | 1108 | 14.1 | 0.80 |
| transcribe.cpp | Nemotron Q8_0, R=6 (560 ms) | 4 | 27 | 215 | 424 | 0.35 | 191 / 235 | 0 | 1107 | 14.1 | 0.80 |
| transcribe.cpp | Nemotron Q8_0, R=6 (560 ms) | 8 | 43 | 335 | 610 | 0.53 | 243 / 293 | 0 | 1108 | 14.1 | 0.80 |
| transcribe.cpp, no OpenBLAS | Nemotron Q8_0, R=6 (560 ms) | 8 | 36 | 261 | 4566* | 0.48 | 1139* / 4464* | 0 | 1103 | 13.7 | 0.80 |
| transcribe.cpp | Nemotron Q8_0, R=1 (160 ms) | 2 | 49 | 111 | 252 | 0.61 | 117 / 179 | 0 | 1094 | 13.3 | 0.64 |
| transcribe.cpp | Nemotron Q8_0, R=1 (160 ms) | 4 | 58 | 148 | 208 | 0.72 | 154 / 189 | 0 | 1097 | 13.3 | 0.64 |
| transcribe.cpp | Nemotron Q8_0, R=1 (160 ms) | 8 | 88 | 501 | 6904* | 1.35 | 179 / 237 | 0 | 1090 | 13.3 | 0.64 |
| Moonshine | medium streaming | 4 CPUs | 102 | 1131 | 3210 | 1.18 | 2 / 3 | 0 | 833 | 25.2 | 1.84 |
| Moonshine | medium streaming | 8 CPUs | 147 | 1719 | 4334 | 1.77 | 2 / 4 | 0 | 828 | 25.2 | 1.84 |
| Moonshine | medium streaming | 16 (default) | 169 | 2067 | 5212 | 1.92 | 3 / 5 | 0 | 830 | 25.2 | 1.84 |
| Moonshine | medium, `decode_incomplete_lines=false` | 4 CPUs | 46 | 325 | 2764 | 0.54 | 1 / 2 | 0 | 814 | 27.0 | 1.84 |
| Moonshine | medium + keyterms | 4 CPUs | 75 | 743 | 2239 | 0.93 | 1 / 1 | 0 | 834 | 24.8 | 1.92 |
| Moonshine | small streaming | 1 CPU (samples 1, 4 only) | 40 | 366 | 874 | 0.51 | 2 / 2 | 0 | 457 | | 1.76 |
| Moonshine | small streaming | 4 CPUs | 48 | 446 | 1848 | 0.56 | 1 / 2 | 0 | 472 | 30.4 | 2.00 |
| Moonshine | small streaming | 8 CPUs | 97 | 1055 | 3381 | 1.13 | 4 / 6 | 0 | 477 | 30.4 | 2.00 |
| Moonshine | small streaming | 16 (default) | 107 | 946 | 3894 | 1.24 | 2 / 5 | 0 | 486 | 30.4 | 2.04 |
| Moonshine | small + keyterms | 4 CPUs | 54 | 578 | 1904 | 0.63 | 1 / 2 | 0 | 482 | 32.6 | 2.08 |
| Parakeet TDT v3 int8 (offline reference) | whole file | 8 | | | | 0.09 to 0.16 | | | | | |

\* host stall during that run, not reproduced on repeat. Model load times: sherpa-onnx Nemotron 3.4 to 4.7 s, transcribe.cpp 0.7 to 2.2 s, Moonshine medium 1.1 to 1.5 s, small 0.3 to 0.5 s.

Per-file RTF, the configurations that matter:

| run | sample 1 | sample 2 | sample 3 | sample 4 | sample 5 |
|---|---|---|---|---|---|
| sherpa-onnx Nemotron 560 ms, 2 thr | 0.31 | 0.28 | 0.28 | 0.32 | 0.30 |
| sherpa-onnx Nemotron 560 ms, 4 thr (run 1 / run 2) | 0.47 / 0.49 | 0.40 / 0.39 | 0.42 / 0.40 | 0.43 / 0.39 | 0.43 / 0.49 |
| sherpa-onnx Nemotron 160 ms, 2 thr | 0.77 | 0.78 | 0.75 | 0.76 | 0.77 |
| transcribe.cpp R=6, 2 thr | 0.31 | 0.32 | 0.30 | 0.31 | 0.31 |
| transcribe.cpp R=6, 4 thr (run 1 / run 2) | 0.30 / 0.35 | 0.28 / 0.37 | 0.38 / 0.37 | 0.38 / 0.36 | 0.38 / 0.34 |
| transcribe.cpp R=1, 2 thr | 0.60 | 0.61 | 0.63 | 0.62 | 0.59 |
| Moonshine medium, 4 CPUs | 1.69 | 1.31 | 1.45 | 1.04 | 0.91 |
| Moonshine small, 4 CPUs | 0.64 | 0.65 | 0.74 | 0.53 | 0.43 |

### Committed text behaviour

- **Nothing revised Committed text.** Across all 30 configurations and 5 files, every change to the committed string was an append. For sherpa-onnx and transcribe.cpp that follows from greedy RNN-T decoding with a cache-aware encoder: a token, once emitted, is never reconsidered. For Moonshine, lines flagged `is_complete` never changed afterwards either.
- **Neither Nemotron runtime produces Tentative text.** sherpa-onnx exposes only the hypothesis so far, which is all committed. transcribe.cpp's `StreamText::tentative` stayed empty for the whole run (`tentative_changed` fired 0 times) even with `CommitPolicy::Auto`; the committed string simply grows by a few tokens every encoder step. So with Nemotron, "text streams out while speaking" means committed words arriving every 560 ms (or 160 ms), and there is no preview of the next words to show. If ptw wants a greyed-out preview, it will have to come from somewhere else (or from a different Model).
- **Moonshine is the opposite.** It keeps the current line tentative and rewrites it wholesale on every update (22 to 61 tentative rewrites per file, often changing earlier words: "Take a" became "Take out", "Hey it's supposed to" became ""), and only commits when its VAD closes the line. That is why its median commit lag is 1.8 to 2.0 s from word onset and individual words waited up to 3.8 s. It also splits lines at every pause, which shows up as stray capitals and periods in the transcripts ("I suppose I should. Take out The mouth guard").

### Flush (end of audio to final text)

Nemotron 560 ms: 160 to 230 ms at 2 threads in both runtimes (the cost of one more encoder step over the padded tail). Nemotron 160 ms: 120 ms transcribe.cpp, 380 ms sherpa-onnx. Moonshine: 1 to 5 ms, because `stop_stream` only has to close the current line; the decode work already happened during feeding.

### Commit lag examples (word, onset per Parakeet -> committed at, lag)

- sherpa-onnx Nemotron 560 ms: "because" 4.00 -> 4.64 (+0.64), "be" 18.40 -> 18.64 (+0.24), "no" 16.24 -> 16.96 (+0.72), "my" 13.04 -> 13.60 (+0.56), "want" 10.80 -> 11.36 (+0.56). Median 0.80 s over 233 words.
- transcribe.cpp R=6: "it" 4.16 -> 5.04 (+0.88), "to" 18.32 -> 18.48 (+0.16), "no" 16.24 -> 16.80 (+0.56), "top" 12.64 -> 13.44 (+0.80). Median 0.80 s over 229 words.
- transcribe.cpp R=1 / sherpa-onnx 160 ms: median 0.64 s over ~220 words; e.g. "to" 18.32 -> 18.56 (+0.24), "of" 12.88 -> 13.44 (+0.56).
- Moonshine medium: "not" 3.92 -> 4.48 (+0.56) but "wait" 13.84 -> 16.24 (+2.40), "i" 6.88 -> 10.64 (+3.76). Median 1.84 s (191 words); small 2.00 s.

The first word of each file lags 2 to 2.5 s in every Engine: that is the model warming up on leading silence plus the chunk quantisation, not steady-state behaviour.

Lag is measured from word *onset*, so it includes the word's own duration (~0.3 s) before the Lookahead even starts. Steady-state, Nemotron commits a word roughly 0.25 to 0.5 s after it ends at 560 ms Lookahead, which matches the Model card's design.

### Threads

Both Nemotron runtimes peak at 2 threads on this box. 1 thread is 1.2x (transcribe.cpp) to 2x (sherpa-onnx) slower than 2; 4 threads is 1.15 to 1.4x slower; 8 threads is 1.7 to 1.9x slower, and its p95 doubles. Moonshine (onnxruntime with spinning enabled) gets slower from 4 CPUs up and is no faster on 4 than on 1. The likely cause is the sandbox's vCPUs mapping onto a mix of P- and E-cores plus a busy host: every thread-pool barrier waits for the slowest thread. This needs re-measuring on the host with pinning before choosing a default, but the safe choice today is 2.

### Why Moonshine is slow here

The per-chunk trace (sample 4, medium, 4 CPUs) shows work landing every 7th chunk (a 560 ms cadence, despite the documented 200 ms throttle), costing 520 to 1350 ms per update while a line is open and growing with the line's length, then dropping back to ~250 ms after the VAD closes the line. That is consistent with re-decoding the whole current line on each update (speculative re-decode of the previous hypothesis, per the header comment). Turning `decode_incomplete_lines` off halves the cost (RTF 0.54) but then no text at all appears until the line closes, which defeats the purpose. Turning speculative decoding off made it slower (RTF 1.5). The paper's 29%-of-audio number was measured on an M3 with an unstated pipeline; the x86 tarball's onnxruntime 1.23.2 may lack the int8 kernels the M3 build uses. I did not profile further.

### Keyterm biasing (Moonshine)

Keyterms passed: `<two first names>,<two neighbourhoods>,Handy,ptw,GNOME,GNOME Shell,Linux,the Mission,grill-me,Wayland` (one family name got three spellings across engines; I did not guess one for the list). Effects, medium: "handy" -> "Handy", "the mission" -> "the Mission", "domeshell's own grab api" -> "Gnome shell's own grab api", "0.46" -> "No. 46". Still wrong with keyterms: both neighbourhoods, "PT" (ptw), "Grill with Docs". WER moved from 25.2 to 24.8% (medium) and 30.4 to 32.6% (small: it produced "grill-of-thoughts" and "ppt"). Cost: none visible (RTF 0.93 with vs 1.18 without is noise in the right direction). So the biasing does what the docs say for words the tokenizer can already spell, and no more; it does not rescue names the acoustic model never gets close to.

## Transcripts

One section per sample. The first row is the offline Parakeet reference and is the one to proofread; the rest are the Engines at their best configuration (2 threads for Nemotron, 4 CPUs for Moonshine). Output is verbatim, including the Engines' own punctuation and casing. Note that transcribe.cpp R=6 dropped a clause at the end of sample 5 ("talking to this computer that will hopefully transcribe"), the only deletion of that size in a Nemotron run.

### ptw sample 1.flac (15.4 s)

| engine | transcript |
|---|---|
| **Parakeet TDT 0.6b v3 int8, offline (reference: proofread this one)** | I suppose I should take out the mouth card before I start talking here. This isn't the best time to capture my voice because I'm a little tired, so my voice is probably creaky. |
| Nemotron 560 ms, sherpa-onnx, 2 thr | Hey, supposedly I should take out the mouth card before I start talking here. This isn't the best time to capture my voice cause I'm a little tired so my voice is probably cooky |
| Nemotron 160 ms, sherpa-onnx, 2 thr | Hey, supposedly I should take out the mouth card before I start talking here. This isn't the best time to capture my voice cause I'm a little tired so my voice is probably cirky |
| Nemotron R=6 (560 ms), transcribe.cpp, 2 thr | Hey, suppose I should take out the mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little tired and so my voice is probably creaky. |
| Nemotron R=1 (160 ms), transcribe.cpp, 2 thr | Hey, supposed I should take out the mouth card before I start talking here This isn't the best time to capture my voice because I'm a little tired and so my voice is probably cricky. |
| Moonshine medium, 4 CPUs | I suppose I should. Take out The mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little... I'm tired, so my voice is probably crooked. |
| Moonshine medium + keyterms | I suppose I should. Take out the mouth guard before I start talking here This isn't the best time to capture my voice because I'm a little... I'm tired, so my voice is probably crooked. |
| Moonshine small, 4 CPUs | Hey, suppose I should. Take out. The mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little... Tired so my voice is probably crooked |
| Moonshine small + keyterms | Hey, suppose I should. Take out. The mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little... Tired so my voice is probably crooked |

### ptw sample 2.flac (26.0 s)

| engine | transcript |
|---|---|
| **Parakeet TDT 0.6b v3 int8, offline (reference: proofread this one)** | I suppose I should have said grill of docs because it seems important to write the results of this planning session down for future sessions to have background. Plant write down these findings when it resolves. Wait, should I just be using Handy? I wasn't using it because it doesn't seem to be streaming. If it streams the results maybe it's findings directly instead of building PT. |
| Nemotron 560 ms, sherpa-onnx, 2 thr | I suppose I should have said grill with docs because it seems important to write the results of this planning session down for future sessions to have background plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams the results maybe it's fine to use directly instead of building PTW |
| Nemotron 160 ms, sherpa-onnx, 2 thr | I suppose I should have said grill with dogs because it seems important to write the results of this planning session down for future sessions to have background plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming but if it streams the results maybe it's fine to use directly instead of building PTW |
| Nemotron R=6 (560 ms), transcribe.cpp, 2 thr | I suppose I should have said grill with docs because it seems important to write the results of this planning session down for future sessions to have background. Plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams or results, maybe it's finding it directly instead of building PTW. |
| Nemotron R=1 (160 ms), transcribe.cpp, 2 thr | I suppose I should have said grill with docks because it seems important to write the results of this planning session down for future sessions to have background plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams the results, maybe it's finding you directly instead of building PTW. |
| Moonshine medium, 4 CPUs | I suppose I should have said "Grill with Docs" because it seems important to write the results of this planning session down. for future sessions to head back. Plan right down these bindings when it resolves. Wait should I just be using handy I love using it because it doesn't seem to be streaming If it's true as a result, maybe it's fine if it's directly instead of building PT |
| Moonshine medium + keyterms | I suppose I should have said grill with docks because it seems important to write the results of this planning session down. for future sessions to head back. Plan right down these bindings when it resolves. Wait should I just be using Handy I love using it because it doesn't seem to be streaming If it's true as a result, maybe it's fine if it's directly instead of building PT |
| Moonshine small, 4 CPUs | I suppose I should have said grill of dogs, because it seems important to write the results of this planning session down. For the future sessions that back. Planetary felony's findings were in the applesauce. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming. If it's true as a result, maybe it's fine if it's directly instead of building PT |
| Moonshine small + keyterms | I suppose I should have said grill-of-thoughts because it seems important to write the results of this planning session down. For the future sessions that back. Planetary felony's findings were in the applesauce. Wait, should I just be using Handy? I wasn't using it because it doesn't seem to be streaming. If it's true as a result, maybe it's fine if it's directly instead of building ppt |

### ptw sample 3.flac (24.9 s)

| engine | transcript |
|---|---|
| **Parakeet TDT 0.6b v3 int8, offline (reference: proofread this one)** | So what is it that we want here? I guess I could just read back something that you wrote to me. Gnome 46, the global shortcuts portal has no back end. Gnome Shell's own grab API is closed three parties. |
| Nemotron 560 ms, sherpa-onnx, 2 thr | So what is it that we want here? I guess I could just read back something that you wrote to me hotkey with release detection on GNOME forty six, the global shortcuts portal has no backend interrupting GNOME forty eight and GNOME shells on Grab API is close to third parties. |
| Nemotron 160 ms, sherpa-onnx, 2 thr | So what is it that we want here? I guess I could just read back something that you wrote to me. Hotkey with release detection on GNOME forty six, the global shortcuts portal has no backend interrupting GNOME forty eight and GNOME shells on Grab API is close to third parties. |
| Nemotron R=6 (560 ms), transcribe.cpp, 2 thr | So what is it that we want here? I guess I could just read back something that you wrote to me hotkey with release detection on GNOME 46 the global shortcuts portal has no back end interrupted GNOME 48 and GNOME shell's own grab API is close to third parties |
| Nemotron R=1 (160 ms), transcribe.cpp, 2 thr | So what is it that we want here? I guess I could just read back something that you wrote to me. Hotkey with release detection on GNOME 46, the global shortcuts portal has no back end. It erupted GNOME 48. And GNOME shell's own grab API is close to third parties. |
| Moonshine medium, 4 CPUs | So? What is it that we... Here Alright, I guess I could just read back something that you wrote to me. Hotkey with release detection. 0.46 the global shortcuts portal has no backend. And domeshell's own grab api is close to 3.0 |
| Moonshine medium + keyterms | So? What is it that we... Here Alright, I guess I could just read back something that you wrote to me. Hotkey with release detection. No. 46 the global shortcuts portal has no back end. They're acting No. 48. Gnome shell's own grab api is close to 3.0 |
| Moonshine small, 4 CPUs | So What is it that we... Here. I guess I could just read back something that you wrote to me. A lot of key with release detection. On 946, the global shortcuts portal has no backend. They were acting at 948. Nome shells own grab api is closed throughput. |
| Moonshine small + keyterms | So What is it that we... Here. I guess I could just read back something that you wrote to me. A lot of key with release detection. No. 46 the global shortcuts portal has no backend. They were out to be known 48. Gnome Shell's own Grab api is closed throughput. |

### ptw sample 4.flac (26.5 s)

| engine | transcript |
|---|---|
| **Parakeet TDT 0.6b v3 int8, offline (reference: proofread this one)** | Actually reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer. |
| Nemotron 560 ms, sherpa-onnx, 2 thr | Actually reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer |
| Nemotron 160 ms, sherpa-onnx, 2 thr | Actually reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer |
| Nemotron R=6 (560 ms), transcribe.cpp, 2 thr | Actually, reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer. |
| Nemotron R=1 (160 ms), transcribe.cpp, 2 thr | Actually, reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer. |
| Moonshine medium, 4 CPUs | Actually reading stuff that's already written is not Really good example data that we're looking for here It would be best to just use one of the words come to mind off the At the top of my head. So that's I like what That would actually be the same as an dictating to the VP. |
| Moonshine medium + keyterms | Actually reading stuff that's already written is not Really the example data that we're looking for here It would be best to just use one of the words come to mind off the the top of my head. So that's I like what That would actually be the same as an dictating to the VP. |
| Moonshine small, 4 CPUs | Actually reading stuff that's already written is not. Another example data that we're looking for here. It would be best to just use one of the words come to mind often. It's at the top of my head. So that's I'm like, what? That would actually be same as in Dictating to the computer. |
| Moonshine small + keyterms | Actually reading stuff that's already written is not. Another example data that we're looking for here. It would be best to just use one of the words come to mind often. It's at the top of my head. So that's I'm like, what? That would actually be the same as in Dictating to the computer. |

### ptw sample 5.flac (53.2 s)

Transcripts withheld: this recording is not in the repository.

## Recommendation

1. **Ship Nemotron Speech Streaming EN 0.6B at the 560 ms Lookahead as the first Engine, on 2 threads.** Either runtime clears the bar: RTF 0.30, p95 chunk 180 ms, Flush ~200 ms, 1 GB RSS, append-only Committed text. Pick by integration cost, not speed:
   - **sherpa-onnx** is the cheaper build (prebuilt static archive, no cmake) and has the hotwords PR in flight; its downside here is the 4 s Model load, the 623 MB int8 encoder per Lookahead (two packages if both Lookaheads are offered), and needing the harness to pad silence to Flush the last chunk.
   - **transcribe.cpp** was a one-minute cmake build from the crate, loads in ~1 s, covers all four Lookaheads from one 696 MB GGUF via `att_context_right`, and its transcripts were marginally closer to the reference. It has no biasing path at all.
   - Given the two are within noise of each other on speed, start with transcribe.cpp for the single-file Model story and the cleaner Flush, and keep the sherpa-onnx harness alive so the hotwords PR can be evaluated the day it merges.
2. **Do not offer the 160 ms Lookahead as a default.** It costs 2 to 2.5x the compute for 0.16 s less commit lag and no accuracy gain on these files. It may make sense as an opt-in on a quieter, pinned host.
3. **Drop Moonshine v2 as a second Engine for now.** On this CPU it is 2 to 4x slower than Nemotron, slower than real time at medium, its lines are committed 1.8 to 2 s after the words, its accuracy on the user's voice is far behind, and keyterms only fix casing of names it already hears. Revisit if a faster x86 build appears or if the GGUF port in transcribe.cpp (no biasing) turns out to be much faster; the C harness and the CPU-count shim are ready for that.
4. **Make thread count a user-visible setting, default 2**, and measure on the real host with `taskset` before trusting any scaling assumption from this sandbox.
5. **Plan for no Tentative text with Nemotron.** The greedy RNN-T path gives committed words every 560 ms and nothing in between. If a live preview matters, it needs to come from a second, cheaper pass, or from a Model that exposes it.
6. Custom vocabulary stays a post-hoc correction layer (Handy-style) until sherpa-onnx's hotwords PR lands; none of the three Engines fixed the two neighbourhoods or "ptw" out of the box.

## What I could not verify

- Whether the "2 threads beats 8" result survives outside the sandbox. It is consistent across both runtimes and repeat runs, but the host was busy and pinning was unavailable.
- Why Moonshine costs 4 to 8x what its paper reports; I did not profile onnxruntime. The `decode_incomplete_lines=false` and CPU-count experiments bound the problem but do not explain it.
- Published Nemotron CPU numbers (transcribe.cpp: 7 to 8x real time on a Ryzen 4750U; sherpa-onnx docs: RTF 0.16) are about 2x better than the 0.30 measured here. Sandbox overhead and the busy host are the likely gap; I did not test on bare metal.
- WER against a corrected reference. The Parakeet reference has visible errors; the per-file JSON logs and `analyze.py` allow recomputation once the references under each sample are fixed.
- sherpa-onnx's `modified_beam_search` + hotwords for Nemotron (PR #3895) was not tested; it is not in 1.13.6.

## Reproducing

Harness sources, run scripts, keyterm list and all per-run JSON logs are in `target/bench/harness/` (gitignored, on the host mount); Models and prebuilt libraries in `target/bench/models` and `target/bench/libs`; cargo target dirs in `target/bench/cargo`. The original working copy is in the session scratchpad under `bench/`. `run_all.sh main` runs the matrix, `run_all.sh keyterms` the Moonshine biasing pass, `run_extras.sh` the 2-thread, repeat and Moonshine diagnostic runs, and `analyze.py <results dirs...>` regenerates every table above from the JSON. Build the two Rust harnesses with `SHERPA_ONNX_LIB_DIR` pointing at the extracted sherpa archive and `CARGO_TARGET_DIR` under `target/bench/cargo`; compile the Moonshine harness with `gcc -O2 bench.c -I<tarball>/include -L<tarball>/lib -lmoonshine -Wl,-rpath,<tarball>/lib`.
