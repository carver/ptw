#!/bin/bash
# Greedy vs modified_beam_search vs beam+hotwords on the 560 ms Nemotron export,
# sherpa-onnx built at the #3895 merge commit. 2 threads = the shipped default.
# usage: run_beam.sh [threads]
set -u
B=$(cd "$(dirname "$0")" && pwd); TB=$B/../target/bench
cd "$B"
T=${1:-2}
WAVS="$TB/sample1.wav $TB/sample2.wav $TB/sample3.wav $TB/sample4.wav $TB/sample5.wav"
OUT=results/beam; mkdir -p $OUT
NEMO560=$TB/models/sherpa-onnx-nemotron-speech-streaming-en-0.6b-560ms-int8-2026-04-25
SHERPA=$TB/cargo/sherpa-bench-beam/release/sherpa-bench
HW=$(tr ',' '/' < keyterms.txt)
log() { echo "[$(date +%H:%M:%S)] $*" >&2; }
log "greedy t=$T";      $SHERPA stream $NEMO560 $T beam0-560ms $OUT $WAVS 2>&1 | grep -E '^(loaded|sample)'
log "beam4 t=$T";       $SHERPA stream $NEMO560 $T beam4-560ms $OUT --beam 4 $WAVS 2>&1 | grep -E '^(loaded|sample)'
log "beam4+hw t=$T";    $SHERPA stream $NEMO560 $T beam4-hw-560ms $OUT --beam 4 --hotwords "$HW" $WAVS 2>&1 | grep -E '^(loaded|sample)'
log done
