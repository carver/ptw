#!/bin/bash
# Full benchmark matrix. Runs sequentially so runs don't contend for cores.
# usage: run_all.sh <phase>   phase = main | keyterms
set -u
B=$(cd "$(dirname "$0")" && pwd); TB=$B/../target/bench
cd "$B"
WAVS="$TB/sample1.wav $TB/sample2.wav $TB/sample3.wav $TB/sample4.wav $TB/sample5.wav"
OUT=results/main; mkdir -p $OUT results/keyterms
NEMO560=$TB/models/sherpa-onnx-nemotron-speech-streaming-en-0.6b-560ms-int8-2026-04-25
NEMO160=$TB/models/sherpa-onnx-nemotron-speech-streaming-en-0.6b-160ms-int8-2026-04-25
GGUF=$TB/models/nemotron-speech-streaming-en-0.6b-Q8_0.gguf
SHERPA=$TB/cargo/sherpa-bench/release/sherpa-bench
TCPP=$TB/cargo/tcpp-bench/release/tcpp-bench
TCPP_NOBLAS=$TB/cargo/tcpp-bench-noblas/release/tcpp-bench
MOON=./moonshine/bench
# Moonshine has no thread option; the LD_PRELOAD shim fakes the CPU count so
# onnxruntime sizes its pool. moon <n> runs with n fake CPUs (0 = unshimmed, 16).
moon() { local n=$1; shift; if [ "$n" = 0 ]; then "$MOON" "$@"; else FAKE_NPROC=$n FAKE_SYS=$B/moonshine/fakesys$n LD_PRELOAD=$B/moonshine/nproc_shim.so "$MOON" "$@"; fi; }
phase=${1:-main}
log() { echo "[$(date +%H:%M:%S)] $*" >&2; }

if [ "$phase" = main ]; then
  for t in 4 8; do
    log "sherpa nemo560 t=$t"; $SHERPA stream $NEMO560 $t sherpa-nemo560 $OUT $WAVS 2>&1 | grep -E '^(loaded|sample)'
    log "sherpa nemo160 t=$t"; $SHERPA stream $NEMO160 $t sherpa-nemo160 $OUT $WAVS 2>&1 | grep -E '^(loaded|sample)'
    log "tcpp R6 t=$t";  $TCPP $GGUF $t 6 tcpp-R6-560ms $OUT $WAVS 2>&1 | grep -E '^(loaded|sample)'
    log "tcpp R1 t=$t";  $TCPP $GGUF $t 1 tcpp-R1-160ms $OUT $WAVS 2>&1 | grep -E '^(loaded|sample)'
  done
  if [ -x $TCPP_NOBLAS ]; then
    log "tcpp R6 t=8 no-BLAS"; $TCPP_NOBLAS $GGUF 8 6 tcpp-R6-560ms-noblas $OUT $WAVS 2>&1 | grep -E '^(loaded|sample)'
  fi
  for n in 4 8 0; do
    log "moonshine medium cpus=$n"; moon $n $TB/models/medium-streaming-en 5 moon-medium-cpu$n $OUT - $WAVS 2>&1 | grep -E '^(loaded|sample)'
    log "moonshine small cpus=$n";  moon $n $TB/models/small-streaming-en 4 moon-small-cpu$n $OUT - $WAVS 2>&1 | grep -E '^(loaded|sample)'
  done
elif [ "$phase" = keyterms ]; then
  KT=$(cat keyterms.txt)
  log "moonshine medium keyterms"; moon 4 $TB/models/medium-streaming-en 5 moon-medium-cpu4-kt results/keyterms "$KT" $WAVS 2>&1 | grep -E '^(loaded|sample)'
  log "moonshine small keyterms";  moon 4 $TB/models/small-streaming-en 4 moon-small-cpu4-kt results/keyterms "$KT" $WAVS 2>&1 | grep -E '^(loaded|sample)'
fi
log done
