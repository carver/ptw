#!/bin/bash
# Extras: 2-thread runs, repeats for variance, Moonshine diagnostics.
set -u
B=$(cd "$(dirname "$0")" && pwd); TB=$B/../target/bench; cd "$B"
WAVS="$TB/sample1.wav $TB/sample2.wav $TB/sample3.wav $TB/sample4.wav $TB/sample5.wav"
OUT=results/extras; mkdir -p $OUT
NEMO560=$TB/models/sherpa-onnx-nemotron-speech-streaming-en-0.6b-560ms-int8-2026-04-25
GGUF=$TB/models/nemotron-speech-streaming-en-0.6b-Q8_0.gguf
SHERPA=$TB/cargo/sherpa-bench/release/sherpa-bench; TCPP=$TB/cargo/tcpp-bench/release/tcpp-bench; MOON=./moonshine/bench
moon() { local n=$1; shift; FAKE_NPROC=$n FAKE_SYS=$B/moonshine/fakesys$n LD_PRELOAD=$B/moonshine/nproc_shim.so "$MOON" "$@"; }
log() { echo "[$(date +%H:%M:%S)] $*" >&2; }
log "sherpa nemo560 t=2"; $SHERPA stream $NEMO560 2 sherpa-nemo560 $OUT $WAVS 2>&1 | grep -E '^(loaded|sample)'
log "tcpp R6 t=2"; $TCPP $GGUF 2 6 tcpp-R6-560ms $OUT $WAVS 2>&1 | grep -E '^(loaded|sample)'
log "repeat sherpa nemo560 t=4"; $SHERPA stream $NEMO560 4 sherpa-nemo560-rep $OUT $WAVS 2>&1 | grep -E '^(loaded|sample)'
log "repeat tcpp R6 t=4"; $TCPP $GGUF 4 6 tcpp-R6-560ms-rep $OUT $WAVS 2>&1 | grep -E '^(loaded|sample)'
log "moonshine medium cpus=4 decode_incomplete_lines=false"; MOON_OPTS="decode_incomplete_lines=false" moon 4 $TB/models/medium-streaming-en 5 moon-medium-cpu4-nodecode $OUT - $WAVS 2>&1 | grep -E '^(loaded|sample)'
log "moonshine small cpus=1"; moon 1 $TB/models/small-streaming-en 4 moon-small-cpu1 $OUT - $TB/sample1.wav $TB/sample4.wav 2>&1 | grep -E '^(loaded|sample)'
log "moonshine medium cpus=4 per-chunk dump"; moon 4 $TB/models/medium-streaming-en 5 moon-medium-cpu4-chunks $OUT - $TB/sample4.wav 2>&1 | grep -E '^(loaded|sample)'
log done
