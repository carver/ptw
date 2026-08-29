---
status: accepted
---

# First Engine: Nemotron Speech Streaming EN 0.6B on transcribe.cpp

Measured on the user's five recordings on this CPU (`docs/research/engine-benchmark.md`): Nemotron at the 560 ms Lookahead runs at 0.3x real time on 2 threads in either runtime, commits a word about 0.3 to 0.5 s after it ends, never revises Committed text, and uses about 1 GB. Moonshine v2 was 2 to 4x slower, slower than real time at medium, committed 1.8 to 2 s late and was far less accurate on this voice; its keyterm biasing only fixed casing of names it already heard. So Nemotron is the Model. Between the runtimes, transcribe.cpp (the `transcribe-cpp` crate, what Handy ships) loads in about a second, covers every Lookahead from one 696 MB GGUF, and had marginally closer transcripts; sherpa-onnx needs a 4 s load and one 623 MB encoder per Lookahead but has a hotwords pull request in flight. We start with transcribe.cpp behind the `Engine` trait and add sherpa-onnx when hotwords for this Model merge.

## Consequences

- Nemotron produces no Tentative text at all under greedy decoding; Committed words arrive every 560 ms. There is nothing to preview even if an overlay became possible.
- The 160 ms Lookahead costs 2 to 2.5x the compute for 0.16 s less lag and no accuracy gain here. It stays a setting, not the default.
- Thread count is a setting, default 2. On the host (i7-1260P, 4 P + 8 E cores) throughput rises with threads, 5.6x real time at 2 to 7.4x at 8, so 8 saves 24 ms per 560 ms chunk against the 560 ms Lookahead, for four times the cores busy while dictating. 2 is the cheapest count that is comfortably real time. 0 lets transcribe.cpp take all 16 hardware threads and lands at 3.9x, slower than 2. The sandbox's "8 is slower than 2" was its CPU quota, not the chip.
- Building needs cmake and a C++ toolchain; OpenBLAS is optional and made no measurable difference.
- Custom words stay a Correction layer for now.
