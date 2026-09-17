| engine | model | thr | files | load ms | chunk mean ms | chunk p95 ms (worst file) | chunk max ms | RTF | finalize ms mean/max | committed revisions | peak RSS MB | WER% vs Parakeet | median commit lag s (n words) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| moonshine | moon-medium-cpu0 | 0 | 5 | 1358 | 169.3 | 2067 | 5212 | 1.920 | 3 / 5 | 0 | 830 | 25.2 | 1.84 (191) |
| moonshine | moon-medium-cpu4 | 0 | 5 | 1490 | 102.3 | 1131 | 3210 | 1.179 | 2 / 3 | 0 | 833 | 25.2 | 1.84 (191) |
| moonshine | moon-medium-cpu4-chunks | 0 | 1 | 666 | 74.9 | 670 | 1602 | 0.937 | 1 / 1 | 0 | 787 | 17.6 | 2.32 (42) |
| moonshine | moon-medium-cpu4-kt | 0 | 5 | 382 | 75.3 | 743 | 2239 | 0.931 | 1 / 1 | 0 | 834 | 24.8 | 1.92 (195) |
| moonshine | moon-medium-cpu4-nodecode | 0 | 5 | 541 | 45.7 | 325 | 2764 | 0.541 | 1 / 2 | 0 | 814 | 27.0 | 1.84 (191) |
| moonshine | moon-medium-cpu8 | 0 | 5 | 1095 | 147.3 | 1719 | 4334 | 1.772 | 2 / 4 | 0 | 828 | 25.2 | 1.84 (191) |
| moonshine | moon-small-cpu0 | 0 | 5 | 467 | 106.8 | 946 | 3894 | 1.238 | 2 / 5 | 0 | 486 | 30.4 | 2.04 (146) |
| moonshine | moon-small-cpu1 | 0 | 2 | 253 | 39.8 | 366 | 874 | 0.506 | 2 / 2 | 0 | 457 | 12.9 | 1.76 (76) |
| moonshine | moon-small-cpu4 | 0 | 5 | 298 | 47.8 | 446 | 1848 | 0.562 | 1 / 2 | 0 | 472 | 30.4 | 2.00 (145) |
| moonshine | moon-small-cpu4-kt | 0 | 5 | 324 | 54.2 | 578 | 1904 | 0.627 | 1 / 2 | 0 | 482 | 32.6 | 2.08 (145) |
| moonshine | moon-small-cpu8 | 0 | 5 | 378 | 96.9 | 1055 | 3381 | 1.127 | 4 / 6 | 0 | 477 | 30.4 | 2.00 (145) |
| sherpa-onnx | sherpa-nemo160 | 2 | 5 | 4694 | 61.2 | 131 | 343 | 0.766 | 379 / 458 | 0 | 967 | 15.6 | 0.64 (216) |
| sherpa-onnx | sherpa-nemo160 | 4 | 5 | 3305 | 119.5 | 422 | 1281 | 1.478 | 1085 / 1972 | 0 | 938 | 15.6 | 0.64 (216) |
| sherpa-onnx | sherpa-nemo160 | 8 | 5 | 3242 | 178.1 | 727 | 1496 | 2.240 | 827 / 1357 | 0 | 962 | 15.6 | 0.64 (216) |
| sherpa-onnx | sherpa-nemo560 | 1 | 5 | 4639 | 48.4 | 365 | 662 | 0.589 | 308 / 410 | 0 | 945 | 15.6 | 0.80 (233) |
| sherpa-onnx | sherpa-nemo560 | 2 | 5 | 4732 | 23.9 | 181 | 473 | 0.299 | 161 / 171 | 0 | 972 | 15.6 | 0.80 (233) |
| sherpa-onnx | sherpa-nemo560 | 4 | 5 | 4681 | 34.5 | 328 | 360 | 0.428 | 227 / 233 | 0 | 933 | 15.6 | 0.80 (233) |
| sherpa-onnx | sherpa-nemo560 | 8 | 5 | 3385 | 47.1 | 466 | 717 | 0.591 | 254 / 349 | 0 | 952 | 15.6 | 0.80 (233) |
| sherpa-onnx | sherpa-nemo560-rep | 4 | 5 | 2843 | 34.6 | 350 | 398 | 0.439 | 236 / 310 | 0 | 952 | 15.6 | 0.80 (233) |
| transcribe-cpp | tcpp-R1-160ms | 2 | 5 | 807 | 48.8 | 111 | 252 | 0.608 | 117 / 179 | 0 | 1094 | 13.3 | 0.64 (221) |
| transcribe-cpp | tcpp-R1-160ms | 4 | 5 | 1075 | 57.9 | 148 | 208 | 0.723 | 154 / 189 | 0 | 1097 | 13.3 | 0.64 (221) |
| transcribe-cpp | tcpp-R1-160ms | 8 | 5 | 1064 | 87.5 | 501 | 6904 | 1.348 | 179 / 237 | 0 | 1090 | 13.3 | 0.64 (221) |
| transcribe-cpp | tcpp-R6-560ms | 1 | 5 | 707 | 30.5 | 222 | 337 | 0.383 | 230 / 276 | 0 | 1107 | 14.1 | 0.80 (229) |
| transcribe-cpp | tcpp-R6-560ms | 2 | 5 | 947 | 24.9 | 185 | 286 | 0.311 | 183 / 215 | 0 | 1108 | 14.1 | 0.80 (229) |
| transcribe-cpp | tcpp-R6-560ms | 4 | 5 | 1278 | 27.4 | 215 | 424 | 0.353 | 191 / 235 | 0 | 1107 | 14.1 | 0.80 (229) |
| transcribe-cpp | tcpp-R6-560ms | 8 | 5 | 2177 | 43.4 | 335 | 610 | 0.526 | 243 / 293 | 0 | 1108 | 14.1 | 0.80 (229) |
| transcribe-cpp | tcpp-R6-560ms-noblas | 8 | 5 | 1149 | 36.4 | 261 | 4566 | 0.480 | 1139 / 4464 | 0 | 1103 | 13.7 | 0.80 (230) |
| transcribe-cpp | tcpp-R6-560ms-rep | 4 | 5 | 1008 | 28.6 | 211 | 353 | 0.355 | 209 / 285 | 0 | 1107 | 14.1 | 0.80 (229) |

## Per-file RTF

| run | sample1 | sample2 | sample3 | sample4 | sample5 |
|---|---|---|---|---|---|
| moonshine moon-medium-cpu0 t0 | 2.723 | 2.293 | 2.354 | 1.907 | 1.310 |
| moonshine moon-medium-cpu4 t0 | 1.687 | 1.306 | 1.447 | 1.044 | 0.911 |
| moonshine moon-medium-cpu4-chunks t0 | - | - | - | 0.938 | - |
| moonshine moon-medium-cpu4-kt t0 | 0.875 | 1.103 | 1.067 | 0.801 | 0.864 |
| moonshine moon-medium-cpu4-nodecode t0 | 0.661 | 0.651 | 0.592 | 0.507 | 0.447 |
| moonshine moon-medium-cpu8 t0 | 1.511 | 2.146 | 2.399 | 1.806 | 1.354 |
| moonshine moon-small-cpu0 t0 | 1.555 | 1.443 | 1.467 | 1.318 | 0.899 |
| moonshine moon-small-cpu1 t0 | 0.464 | - | - | 0.530 | - |
| moonshine moon-small-cpu4 t0 | 0.638 | 0.654 | 0.741 | 0.529 | 0.429 |
| moonshine moon-small-cpu4-kt t0 | 0.744 | 0.812 | 0.860 | 0.542 | 0.435 |
| moonshine moon-small-cpu8 t0 | 1.036 | 1.600 | 1.687 | 1.028 | 0.709 |
| sherpa-onnx sherpa-nemo160 t2 | 0.767 | 0.776 | 0.749 | 0.763 | 0.770 |
| sherpa-onnx sherpa-nemo160 t4 | 1.252 | 1.935 | 1.660 | 1.306 | 1.320 |
| sherpa-onnx sherpa-nemo160 t8 | 2.126 | 1.615 | 2.325 | 2.823 | 2.248 |
| sherpa-onnx sherpa-nemo560 t1 | 0.656 | 0.659 | 0.646 | 0.527 | 0.539 |
| sherpa-onnx sherpa-nemo560 t2 | 0.310 | 0.280 | 0.282 | 0.319 | 0.304 |
| sherpa-onnx sherpa-nemo560 t4 | 0.474 | 0.402 | 0.420 | 0.433 | 0.429 |
| sherpa-onnx sherpa-nemo560 t8 | 0.602 | 0.505 | 0.514 | 0.725 | 0.599 |
| sherpa-onnx sherpa-nemo560-rep t4 | 0.493 | 0.393 | 0.398 | 0.394 | 0.487 |
| transcribe-cpp tcpp-R1-160ms t2 | 0.601 | 0.612 | 0.629 | 0.619 | 0.593 |
| transcribe-cpp tcpp-R1-160ms t4 | 0.739 | 0.711 | 0.673 | 0.779 | 0.719 |
| transcribe-cpp tcpp-R1-160ms t8 | 0.810 | 0.793 | 0.756 | 0.770 | 2.341 |
| transcribe-cpp tcpp-R6-560ms t1 | 0.363 | 0.375 | 0.383 | 0.404 | 0.381 |
| transcribe-cpp tcpp-R6-560ms t2 | 0.311 | 0.322 | 0.304 | 0.313 | 0.309 |
| transcribe-cpp tcpp-R6-560ms t4 | 0.300 | 0.281 | 0.377 | 0.381 | 0.378 |
| transcribe-cpp tcpp-R6-560ms t8 | 0.565 | 0.521 | 0.558 | 0.614 | 0.459 |
| transcribe-cpp tcpp-R6-560ms-noblas t8 | 0.362 | 0.385 | 0.359 | 0.627 | 0.544 |
| transcribe-cpp tcpp-R6-560ms-rep t4 | 0.352 | 0.372 | 0.370 | 0.361 | 0.337 |

## Per-file WER% vs Parakeet offline

| run | sample1 | sample2 | sample3 | sample4 | sample5 |
|---|---|---|---|---|---|
| moonshine moon-medium-cpu0 t0 | 8.8 | 22.4 | 37.5 | 23.5 | 29.5 |
| moonshine moon-medium-cpu4 t0 | 8.8 | 22.4 | 37.5 | 23.5 | 29.5 |
| moonshine moon-medium-cpu4-chunks t0 | - | - | - | 17.6 | - |
| moonshine moon-medium-cpu4-kt t0 | 8.8 | 23.9 | 37.5 | 19.6 | 29.5 |
| moonshine moon-medium-cpu4-nodecode t0 | 8.8 | 26.9 | 45.0 | 21.6 | 29.5 |
| moonshine moon-medium-cpu8 t0 | 8.8 | 22.4 | 37.5 | 23.5 | 29.5 |
| moonshine moon-small-cpu0 t0 | 8.8 | 31.3 | 50.0 | 23.5 | 33.3 |
| moonshine moon-small-cpu1 t0 | 8.8 | - | - | 15.7 | - |
| moonshine moon-small-cpu4 t0 | 8.8 | 31.3 | 50.0 | 23.5 | 33.3 |
| moonshine moon-small-cpu4-kt t0 | 8.8 | 32.8 | 47.5 | 25.5 | 39.7 |
| moonshine moon-small-cpu8 t0 | 8.8 | 31.3 | 50.0 | 23.5 | 33.3 |
| sherpa-onnx sherpa-nemo160 t2 | 11.8 | 13.4 | 45.0 | 0.0 | 14.1 |
| sherpa-onnx sherpa-nemo160 t4 | 11.8 | 13.4 | 45.0 | 0.0 | 14.1 |
| sherpa-onnx sherpa-nemo160 t8 | 11.8 | 13.4 | 45.0 | 0.0 | 14.1 |
| sherpa-onnx sherpa-nemo560 t1 | 11.8 | 11.9 | 45.0 | 0.0 | 15.4 |
| sherpa-onnx sherpa-nemo560 t2 | 11.8 | 11.9 | 45.0 | 0.0 | 15.4 |
| sherpa-onnx sherpa-nemo560 t4 | 11.8 | 11.9 | 45.0 | 0.0 | 15.4 |
| sherpa-onnx sherpa-nemo560 t8 | 11.8 | 11.9 | 45.0 | 0.0 | 15.4 |
| sherpa-onnx sherpa-nemo560-rep t4 | 11.8 | 11.9 | 45.0 | 0.0 | 15.4 |
| transcribe-cpp tcpp-R1-160ms t2 | 11.8 | 11.9 | 32.5 | 0.0 | 14.1 |
| transcribe-cpp tcpp-R1-160ms t4 | 11.8 | 11.9 | 32.5 | 0.0 | 14.1 |
| transcribe-cpp tcpp-R1-160ms t8 | 11.8 | 11.9 | 32.5 | 0.0 | 14.1 |
| transcribe-cpp tcpp-R6-560ms t1 | 8.8 | 11.9 | 30.0 | 0.0 | 19.2 |
| transcribe-cpp tcpp-R6-560ms t2 | 8.8 | 11.9 | 30.0 | 0.0 | 19.2 |
| transcribe-cpp tcpp-R6-560ms t4 | 8.8 | 11.9 | 30.0 | 0.0 | 19.2 |
| transcribe-cpp tcpp-R6-560ms t8 | 8.8 | 11.9 | 30.0 | 0.0 | 19.2 |
| transcribe-cpp tcpp-R6-560ms-noblas t8 | 5.9 | 11.9 | 30.0 | 0.0 | 19.2 |
| transcribe-cpp tcpp-R6-560ms-rep t4 | 8.8 | 11.9 | 30.0 | 0.0 | 19.2 |

## Commit lag examples (word, spoken at s per Parakeet, committed at s, lag s)

- moonshine moon-medium-cpu0 t0: i 0.40->2.24 (+1.84); probably 13.28->14.56 (+1.28); wait 13.84->16.24 (+2.40); i 6.88->10.64 (+3.76); not 3.92->4.48 (+0.56); be 20.48->22.40 (+1.92)
- moonshine moon-medium-cpu4 t0: i 0.40->2.24 (+1.84); probably 13.28->14.56 (+1.28); wait 13.84->16.24 (+2.40); i 6.88->10.64 (+3.76); not 3.92->4.48 (+0.56); be 20.48->22.40 (+1.92)
- moonshine moon-medium-cpu4-chunks t0: actually 0.64->7.84 (+7.20); not 3.92->7.84 (+3.92); looking 6.40->7.84 (+1.44); to 9.04->14.00 (+4.96); the 12.40->14.00 (+1.60); like 18.08->19.04 (+0.96)
- moonshine moon-medium-cpu4-kt t0: i 0.40->2.24 (+1.84); i 0.80->7.84 (+7.04); just 14.80->16.24 (+1.44); just 7.68->10.64 (+2.96); really 4.24->7.84 (+3.60); the 22.96->22.40 (+-0.56)
- moonshine moon-medium-cpu4-nodecode t0: i 0.40->2.24 (+1.84); probably 13.28->14.56 (+1.28); wait 13.84->16.24 (+2.40); guess 7.04->10.64 (+3.60); example 5.04->7.84 (+2.80); to 22.80->24.08 (+1.28)
- moonshine moon-medium-cpu8 t0: i 0.40->2.24 (+1.84); probably 13.28->14.56 (+1.28); wait 13.84->16.24 (+2.40); i 6.88->10.64 (+3.76); not 3.92->4.48 (+0.56); be 20.48->22.40 (+1.92)
- moonshine moon-small-cpu0 t0: suppose 0.96->2.24 (+1.28); little 11.68->12.32 (+0.64); this 5.92->7.84 (+1.92); you 9.04->10.64 (+1.60); example 5.04->7.84 (+2.80); that's 16.40->17.36 (+0.96)
- moonshine moon-small-cpu1 t0: suppose 0.96->2.24 (+1.28); this 8.32->12.32 (+4.00); little 11.68->12.32 (+0.64); written 3.20->4.48 (+1.28); it 7.92->14.00 (+6.08); top 12.64->14.00 (+1.36)
- moonshine moon-small-cpu4 t0: suppose 0.96->2.24 (+1.28); little 11.68->12.32 (+0.64); this 5.92->7.84 (+1.92); you 9.04->10.64 (+1.60); data 5.76->7.84 (+2.08); like 18.08->19.04 (+0.96)
- moonshine moon-small-cpu4-kt t0: suppose 0.96->2.24 (+1.28); little 11.68->12.32 (+0.64); this 5.92->7.84 (+1.92); you 9.04->10.64 (+1.60); is 3.60->4.48 (+0.88); head 13.28->14.00 (+0.72)
- moonshine moon-small-cpu8 t0: suppose 0.96->2.24 (+1.28); little 11.68->12.32 (+0.64); this 5.92->7.84 (+1.92); you 9.04->10.64 (+1.60); data 5.76->7.84 (+2.08); like 18.08->19.04 (+0.96)
- sherpa-onnx sherpa-nemo160 t2: i 0.40->2.56 (+2.16); because 4.00->4.64 (+0.64); to 18.32->18.56 (+0.24); grab 20.64->21.28 (+0.64); so 15.60->16.80 (+1.20); wake 10.96->11.68 (+0.72)
- sherpa-onnx sherpa-nemo160 t4: i 0.40->2.56 (+2.16); because 4.00->4.64 (+0.64); to 18.32->18.56 (+0.24); grab 20.64->21.28 (+0.64); so 15.60->16.80 (+1.20); wake 10.96->11.68 (+0.72)
- sherpa-onnx sherpa-nemo160 t8: i 0.40->2.56 (+2.16); because 4.00->4.64 (+0.64); to 18.32->18.56 (+0.24); grab 20.64->21.28 (+0.64); so 15.60->16.80 (+1.20); wake 10.96->11.68 (+0.72)
- sherpa-onnx sherpa-nemo560 t1: i 0.40->2.96 (+2.56); because 4.00->4.64 (+0.64); be 18.40->18.64 (+0.24); no 16.24->16.96 (+0.72); my 13.04->13.60 (+0.56); want 10.80->11.36 (+0.56)
- sherpa-onnx sherpa-nemo560 t2: i 0.40->2.96 (+2.56); because 4.00->4.64 (+0.64); be 18.40->18.64 (+0.24); no 16.24->16.96 (+0.72); my 13.04->13.60 (+0.56); want 10.80->11.36 (+0.56)
- sherpa-onnx sherpa-nemo560 t4: i 0.40->2.96 (+2.56); because 4.00->4.64 (+0.64); be 18.40->18.64 (+0.24); no 16.24->16.96 (+0.72); my 13.04->13.60 (+0.56); want 10.80->11.36 (+0.56)
- sherpa-onnx sherpa-nemo560 t8: i 0.40->2.96 (+2.56); because 4.00->4.64 (+0.64); be 18.40->18.64 (+0.24); no 16.24->16.96 (+0.72); my 13.04->13.60 (+0.56); want 10.80->11.36 (+0.56)
- sherpa-onnx sherpa-nemo560-rep t4: i 0.40->2.96 (+2.56); because 4.00->4.64 (+0.64); be 18.40->18.64 (+0.24); no 16.24->16.96 (+0.72); my 13.04->13.60 (+0.56); want 10.80->11.36 (+0.56)
- transcribe-cpp tcpp-R1-160ms t2: i 0.40->2.40 (+2.00); grill 2.72->3.84 (+1.12); it 17.60->18.24 (+0.64); no 16.24->16.80 (+0.56); of 12.88->13.44 (+0.56); don't 10.64->11.36 (+0.72)
- transcribe-cpp tcpp-R1-160ms t4: i 0.40->2.40 (+2.00); grill 2.72->3.84 (+1.12); it 17.60->18.24 (+0.64); no 16.24->16.80 (+0.56); of 12.88->13.44 (+0.56); don't 10.64->11.36 (+0.72)
- transcribe-cpp tcpp-R1-160ms t8: i 0.40->2.40 (+2.00); grill 2.72->3.84 (+1.12); it 17.60->18.24 (+0.64); no 16.24->16.80 (+0.56); of 12.88->13.44 (+0.56); don't 10.64->11.36 (+0.72)
- transcribe-cpp tcpp-R6-560ms t1: suppose 0.96->2.24 (+1.28); it 4.16->5.04 (+0.88); to 18.32->18.48 (+0.16); no 16.24->16.80 (+0.56); top 12.64->13.44 (+0.80); is 9.84->10.64 (+0.80)
- transcribe-cpp tcpp-R6-560ms t2: suppose 0.96->2.24 (+1.28); it 4.16->5.04 (+0.88); to 18.32->18.48 (+0.16); no 16.24->16.80 (+0.56); top 12.64->13.44 (+0.80); is 9.84->10.64 (+0.80)
- transcribe-cpp tcpp-R6-560ms t4: suppose 0.96->2.24 (+1.28); it 4.16->5.04 (+0.88); to 18.32->18.48 (+0.16); no 16.24->16.80 (+0.56); top 12.64->13.44 (+0.80); is 9.84->10.64 (+0.80)
- transcribe-cpp tcpp-R6-560ms t8: suppose 0.96->2.24 (+1.28); it 4.16->5.04 (+0.88); to 18.32->18.48 (+0.16); no 16.24->16.80 (+0.56); top 12.64->13.44 (+0.80); is 9.84->10.64 (+0.80)
- transcribe-cpp tcpp-R6-560ms-noblas t8: suppose 0.96->2.24 (+1.28); because 4.00->4.48 (+0.48); seem 18.08->18.48 (+0.40); has 16.08->16.80 (+0.72); the 12.40->13.44 (+1.04); else 9.52->10.08 (+0.56)
- transcribe-cpp tcpp-R6-560ms-rep t4: suppose 0.96->2.24 (+1.28); it 4.16->5.04 (+0.88); to 18.32->18.48 (+0.16); no 16.24->16.80 (+0.56); top 12.64->13.44 (+0.80); is 9.84->10.64 (+0.80)

## Revisions of committed text


## Transcripts

### sample1 (15.4 s)

- **Parakeet TDT 0.6b v3 int8 offline (reference, proofread me)**: I suppose I should take out the mouth card before I start talking here. This isn't the best time to capture my voice because I'm a little tired, so my voice is probably creaky.
- moonshine moon-medium-cpu0 t0: I suppose I should. Take out The mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little... I'm tired, so my voice is probably crooked.
- moonshine moon-medium-cpu4 t0: I suppose I should. Take out The mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little... I'm tired, so my voice is probably crooked.
- moonshine moon-medium-cpu4-kt t0: I suppose I should. Take out the mouth guard before I start talking here This isn't the best time to capture my voice because I'm a little... I'm tired, so my voice is probably crooked.
- moonshine moon-medium-cpu4-nodecode t0: I suppose I should. Take out The mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little... I'm tired, so my voice is probably crooked.
- moonshine moon-medium-cpu8 t0: I suppose I should. Take out The mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little... I'm tired, so my voice is probably crooked.
- moonshine moon-small-cpu0 t0: Hey, suppose I should. Take out. The mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little... Tired so my voice is probably crooked
- moonshine moon-small-cpu1 t0: Hey, suppose I should. Take out. The mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little... Tired so my voice is probably crooked
- moonshine moon-small-cpu4 t0: Hey, suppose I should. Take out. The mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little... Tired so my voice is probably crooked
- moonshine moon-small-cpu4-kt t0: Hey, suppose I should. Take out. The mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little... Tired so my voice is probably crooked
- moonshine moon-small-cpu8 t0: Hey, suppose I should. Take out. The mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little... Tired so my voice is probably crooked
- sherpa-onnx sherpa-nemo160 t2: Hey, supposedly I should take out the mouth card before I start talking here. This isn't the best time to capture my voice cause I'm a little tired so my voice is probably cirky
- sherpa-onnx sherpa-nemo160 t4: Hey, supposedly I should take out the mouth card before I start talking here. This isn't the best time to capture my voice cause I'm a little tired so my voice is probably cirky
- sherpa-onnx sherpa-nemo160 t8: Hey, supposedly I should take out the mouth card before I start talking here. This isn't the best time to capture my voice cause I'm a little tired so my voice is probably cirky
- sherpa-onnx sherpa-nemo560 t1: Hey, supposedly I should take out the mouth card before I start talking here. This isn't the best time to capture my voice cause I'm a little tired so my voice is probably cooky
- sherpa-onnx sherpa-nemo560 t2: Hey, supposedly I should take out the mouth card before I start talking here. This isn't the best time to capture my voice cause I'm a little tired so my voice is probably cooky
- sherpa-onnx sherpa-nemo560 t4: Hey, supposedly I should take out the mouth card before I start talking here. This isn't the best time to capture my voice cause I'm a little tired so my voice is probably cooky
- sherpa-onnx sherpa-nemo560 t8: Hey, supposedly I should take out the mouth card before I start talking here. This isn't the best time to capture my voice cause I'm a little tired so my voice is probably cooky
- sherpa-onnx sherpa-nemo560-rep t4: Hey, supposedly I should take out the mouth card before I start talking here. This isn't the best time to capture my voice cause I'm a little tired so my voice is probably cooky
- transcribe-cpp tcpp-R1-160ms t2: Hey, supposed I should take out the mouth card before I start talking here This isn't the best time to capture my voice because I'm a little tired and so my voice is probably cricky.
- transcribe-cpp tcpp-R1-160ms t4: Hey, supposed I should take out the mouth card before I start talking here This isn't the best time to capture my voice because I'm a little tired and so my voice is probably cricky.
- transcribe-cpp tcpp-R1-160ms t8: Hey, supposed I should take out the mouth card before I start talking here This isn't the best time to capture my voice because I'm a little tired and so my voice is probably cricky.
- transcribe-cpp tcpp-R6-560ms t1: Hey, suppose I should take out the mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little tired and so my voice is probably creaky.
- transcribe-cpp tcpp-R6-560ms t2: Hey, suppose I should take out the mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little tired and so my voice is probably creaky.
- transcribe-cpp tcpp-R6-560ms t4: Hey, suppose I should take out the mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little tired and so my voice is probably creaky.
- transcribe-cpp tcpp-R6-560ms t8: Hey, suppose I should take out the mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little tired and so my voice is probably creaky.
- transcribe-cpp tcpp-R6-560ms-noblas t8: Hey, suppose I should take out the mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little tired, so my voice is probably creaky.
- transcribe-cpp tcpp-R6-560ms-rep t4: Hey, suppose I should take out the mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little tired and so my voice is probably creaky.

### sample2 (26.0 s)

- **Parakeet TDT 0.6b v3 int8 offline (reference, proofread me)**: I suppose I should have said grill of docs because it seems important to write the results of this planning session down for future sessions to have background. Plant write down these findings when it resolves. Wait, should I just be using Handy? I wasn't using it because it doesn't seem to be streaming. If it streams the results maybe it's findings directly instead of building PT.
- moonshine moon-medium-cpu0 t0: I suppose I should have said "Grill with Docs" because it seems important to write the results of this planning session down. for future sessions to head back. Plan right down these bindings when it resolves. Wait should I just be using handy I love using it because it doesn't seem to be streaming If it's true as a result, maybe it's fine if it's directly instead of building PT
- moonshine moon-medium-cpu4 t0: I suppose I should have said "Grill with Docs" because it seems important to write the results of this planning session down. for future sessions to head back. Plan right down these bindings when it resolves. Wait should I just be using handy I love using it because it doesn't seem to be streaming If it's true as a result, maybe it's fine if it's directly instead of building PT
- moonshine moon-medium-cpu4-kt t0: I suppose I should have said grill with docks because it seems important to write the results of this planning session down. for future sessions to head back. Plan right down these bindings when it resolves. Wait should I just be using Handy I love using it because it doesn't seem to be streaming If it's true as a result, maybe it's fine if it's directly instead of building PT
- moonshine moon-medium-cpu4-nodecode t0: I suppose I should have said "Grill with Docs" because it seems important to write the results of this planning session down. for future sessions to head back. Plan right down these bindings when it resolves. Wait should I just be using handy I love using it because it doesn't seem to be streaming. If it's true as a result, maybe it's fine if you try to play instead of building PT.
- moonshine moon-medium-cpu8 t0: I suppose I should have said "Grill with Docs" because it seems important to write the results of this planning session down. for future sessions to head back. Plan right down these bindings when it resolves. Wait should I just be using handy I love using it because it doesn't seem to be streaming If it's true as a result, maybe it's fine if it's directly instead of building PT
- moonshine moon-small-cpu0 t0: I suppose I should have said grill of dogs, because it seems important to write the results of this planning session down. For the future sessions that back. Planetary felony's findings were in the applesauce. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming. If it's true as a result, maybe it's fine if it's directly instead of building PT
- moonshine moon-small-cpu4 t0: I suppose I should have said grill of dogs, because it seems important to write the results of this planning session down. For the future sessions that back. Planetary felony's findings were in the applesauce. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming. If it's true as a result, maybe it's fine if it's directly instead of building PT
- moonshine moon-small-cpu4-kt t0: I suppose I should have said grill-of-thoughts because it seems important to write the results of this planning session down. For the future sessions that back. Planetary felony's findings were in the applesauce. Wait, should I just be using Handy? I wasn't using it because it doesn't seem to be streaming. If it's true as a result, maybe it's fine if it's directly instead of building ppt
- moonshine moon-small-cpu8 t0: I suppose I should have said grill of dogs, because it seems important to write the results of this planning session down. For the future sessions that back. Planetary felony's findings were in the applesauce. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming. If it's true as a result, maybe it's fine if it's directly instead of building PT
- sherpa-onnx sherpa-nemo160 t2: I suppose I should have said grill with dogs because it seems important to write the results of this planning session down for future sessions to have background plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming but if it streams the results maybe it's fine to use directly instead of building PTW
- sherpa-onnx sherpa-nemo160 t4: I suppose I should have said grill with dogs because it seems important to write the results of this planning session down for future sessions to have background plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming but if it streams the results maybe it's fine to use directly instead of building PTW
- sherpa-onnx sherpa-nemo160 t8: I suppose I should have said grill with dogs because it seems important to write the results of this planning session down for future sessions to have background plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming but if it streams the results maybe it's fine to use directly instead of building PTW
- sherpa-onnx sherpa-nemo560 t1: I suppose I should have said grill with docs because it seems important to write the results of this planning session down for future sessions to have background plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams the results maybe it's fine to use directly instead of building PTW
- sherpa-onnx sherpa-nemo560 t2: I suppose I should have said grill with docs because it seems important to write the results of this planning session down for future sessions to have background plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams the results maybe it's fine to use directly instead of building PTW
- sherpa-onnx sherpa-nemo560 t4: I suppose I should have said grill with docs because it seems important to write the results of this planning session down for future sessions to have background plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams the results maybe it's fine to use directly instead of building PTW
- sherpa-onnx sherpa-nemo560 t8: I suppose I should have said grill with docs because it seems important to write the results of this planning session down for future sessions to have background plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams the results maybe it's fine to use directly instead of building PTW
- sherpa-onnx sherpa-nemo560-rep t4: I suppose I should have said grill with docs because it seems important to write the results of this planning session down for future sessions to have background plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams the results maybe it's fine to use directly instead of building PTW
- transcribe-cpp tcpp-R1-160ms t2: I suppose I should have said grill with docks because it seems important to write the results of this planning session down for future sessions to have background plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams the results, maybe it's finding you directly instead of building PTW.
- transcribe-cpp tcpp-R1-160ms t4: I suppose I should have said grill with docks because it seems important to write the results of this planning session down for future sessions to have background plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams the results, maybe it's finding you directly instead of building PTW.
- transcribe-cpp tcpp-R1-160ms t8: I suppose I should have said grill with docks because it seems important to write the results of this planning session down for future sessions to have background plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams the results, maybe it's finding you directly instead of building PTW.
- transcribe-cpp tcpp-R6-560ms t1: I suppose I should have said grill with docs because it seems important to write the results of this planning session down for future sessions to have background. Plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams or results, maybe it's finding it directly instead of building PTW.
- transcribe-cpp tcpp-R6-560ms t2: I suppose I should have said grill with docs because it seems important to write the results of this planning session down for future sessions to have background. Plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams or results, maybe it's finding it directly instead of building PTW.
- transcribe-cpp tcpp-R6-560ms t4: I suppose I should have said grill with docs because it seems important to write the results of this planning session down for future sessions to have background. Plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams or results, maybe it's finding it directly instead of building PTW.
- transcribe-cpp tcpp-R6-560ms t8: I suppose I should have said grill with docs because it seems important to write the results of this planning session down for future sessions to have background. Plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams or results, maybe it's finding it directly instead of building PTW.
- transcribe-cpp tcpp-R6-560ms-noblas t8: I suppose I should have said grill with docs because it seems important to write the results of this planning session down for future sessions to have background. Plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams or results, maybe it's finding it directly instead of building PTW.
- transcribe-cpp tcpp-R6-560ms-rep t4: I suppose I should have said grill with docs because it seems important to write the results of this planning session down for future sessions to have background. Plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams or results, maybe it's finding it directly instead of building PTW.

### sample3 (24.9 s)

- **Parakeet TDT 0.6b v3 int8 offline (reference, proofread me)**: So what is it that we want here? I guess I could just read back something that you wrote to me. Gnome 46, the global shortcuts portal has no back end. Gnome Shell's own grab API is closed three parties.
- moonshine moon-medium-cpu0 t0: So? What is it that we... Here Alright, I guess I could just read back something that you wrote to me. Hotkey with release detection. 0.46 the global shortcuts portal has no backend. And domeshell's own grab api is close to 3.0
- moonshine moon-medium-cpu4 t0: So? What is it that we... Here Alright, I guess I could just read back something that you wrote to me. Hotkey with release detection. 0.46 the global shortcuts portal has no backend. And domeshell's own grab api is close to 3.0
- moonshine moon-medium-cpu4-kt t0: So? What is it that we... Here Alright, I guess I could just read back something that you wrote to me. Hotkey with release detection. No. 46 the global shortcuts portal has no back end. They're acting No. 48. Gnome shell's own grab api is close to 3.0
- moonshine moon-medium-cpu4-nodecode t0: So, um? What is it that we... Here Alright, I guess I could just read back something that you wrote to me. Hotkey with release detection. 946, the global shortcuts portal, has no backend. They're acting 948. And domeshell's own grab api is close to 3.0
- moonshine moon-medium-cpu8 t0: So? What is it that we... Here Alright, I guess I could just read back something that you wrote to me. Hotkey with release detection. 0.46 the global shortcuts portal has no backend. And domeshell's own grab api is close to 3.0
- moonshine moon-small-cpu0 t0: So What is it that we... Here. I guess I could just read back something that you wrote to me. A lot of key with release detection. Now in 46, the global shortcuts portal has no backend. They were out to 1148. Nome shells own grab api is closed throughput.
- moonshine moon-small-cpu4 t0: So What is it that we... Here. I guess I could just read back something that you wrote to me. A lot of key with release detection. On 946, the global shortcuts portal has no backend. They were acting at 948. Nome shells own grab api is closed throughput.
- moonshine moon-small-cpu4-kt t0: So What is it that we... Here. I guess I could just read back something that you wrote to me. A lot of key with release detection. No. 46 the global shortcuts portal has no backend. They were out to be known 48. Gnome Shell's own Grab api is closed throughput.
- moonshine moon-small-cpu8 t0: So What is it that we... Here. I guess I could just read back something that you wrote to me. A lot of key with release detection. On 946, the global shortcuts portal has no backend. They were acting at 948. Nome shells own grab api is closed throughput.
- sherpa-onnx sherpa-nemo160 t2: So what is it that we want here? I guess I could just read back something that you wrote to me. Hotkey with release detection on GNOME forty six, the global shortcuts portal has no backend interrupting GNOME forty eight and GNOME shells on Grab API is close to third parties.
- sherpa-onnx sherpa-nemo160 t4: So what is it that we want here? I guess I could just read back something that you wrote to me. Hotkey with release detection on GNOME forty six, the global shortcuts portal has no backend interrupting GNOME forty eight and GNOME shells on Grab API is close to third parties.
- sherpa-onnx sherpa-nemo160 t8: So what is it that we want here? I guess I could just read back something that you wrote to me. Hotkey with release detection on GNOME forty six, the global shortcuts portal has no backend interrupting GNOME forty eight and GNOME shells on Grab API is close to third parties.
- sherpa-onnx sherpa-nemo560 t1: So what is it that we want here? I guess I could just read back something that you wrote to me hotkey with release detection on GNOME forty six, the global shortcuts portal has no backend interrupting GNOME forty eight and GNOME shells on Grab API is close to third parties.
- sherpa-onnx sherpa-nemo560 t2: So what is it that we want here? I guess I could just read back something that you wrote to me hotkey with release detection on GNOME forty six, the global shortcuts portal has no backend interrupting GNOME forty eight and GNOME shells on Grab API is close to third parties.
- sherpa-onnx sherpa-nemo560 t4: So what is it that we want here? I guess I could just read back something that you wrote to me hotkey with release detection on GNOME forty six, the global shortcuts portal has no backend interrupting GNOME forty eight and GNOME shells on Grab API is close to third parties.
- sherpa-onnx sherpa-nemo560 t8: So what is it that we want here? I guess I could just read back something that you wrote to me hotkey with release detection on GNOME forty six, the global shortcuts portal has no backend interrupting GNOME forty eight and GNOME shells on Grab API is close to third parties.
- sherpa-onnx sherpa-nemo560-rep t4: So what is it that we want here? I guess I could just read back something that you wrote to me hotkey with release detection on GNOME forty six, the global shortcuts portal has no backend interrupting GNOME forty eight and GNOME shells on Grab API is close to third parties.
- transcribe-cpp tcpp-R1-160ms t2: So what is it that we want here? I guess I could just read back something that you wrote to me. Hotkey with release detection on GNOME 46, the global shortcuts portal has no back end. It erupted GNOME 48. And GNOME shell's own grab API is close to third parties.
- transcribe-cpp tcpp-R1-160ms t4: So what is it that we want here? I guess I could just read back something that you wrote to me. Hotkey with release detection on GNOME 46, the global shortcuts portal has no back end. It erupted GNOME 48. And GNOME shell's own grab API is close to third parties.
- transcribe-cpp tcpp-R1-160ms t8: So what is it that we want here? I guess I could just read back something that you wrote to me. Hotkey with release detection on GNOME 46, the global shortcuts portal has no back end. It erupted GNOME 48. And GNOME shell's own grab API is close to third parties.
- transcribe-cpp tcpp-R6-560ms t1: So what is it that we want here? I guess I could just read back something that you wrote to me hotkey with release detection on GNOME 46 the global shortcuts portal has no back end interrupted GNOME 48 and GNOME shell's own grab API is close to third parties
- transcribe-cpp tcpp-R6-560ms t2: So what is it that we want here? I guess I could just read back something that you wrote to me hotkey with release detection on GNOME 46 the global shortcuts portal has no back end interrupted GNOME 48 and GNOME shell's own grab API is close to third parties
- transcribe-cpp tcpp-R6-560ms t4: So what is it that we want here? I guess I could just read back something that you wrote to me hotkey with release detection on GNOME 46 the global shortcuts portal has no back end interrupted GNOME 48 and GNOME shell's own grab API is close to third parties
- transcribe-cpp tcpp-R6-560ms t8: So what is it that we want here? I guess I could just read back something that you wrote to me hotkey with release detection on GNOME 46 the global shortcuts portal has no back end interrupted GNOME 48 and GNOME shell's own grab API is close to third parties
- transcribe-cpp tcpp-R6-560ms-noblas t8: So what is it that we want here? I guess I could just read back something that you wrote to me hotkey with release detection on GNOME 46 the global shortcuts portal has no back end interrupted GNOME 48 and GNOME shell's own grab API is close to third parties
- transcribe-cpp tcpp-R6-560ms-rep t4: So what is it that we want here? I guess I could just read back something that you wrote to me hotkey with release detection on GNOME 46 the global shortcuts portal has no back end interrupted GNOME 48 and GNOME shell's own grab API is close to third parties

### sample4 (26.5 s)

- **Parakeet TDT 0.6b v3 int8 offline (reference, proofread me)**: Actually reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer.
- moonshine moon-medium-cpu0 t0: Actually reading stuff that's already written is not Really good example data that we're looking for here It would be best to just use one of the words come to mind off the At the top of my head. So that's I like what That would actually be the same as an dictating to the VP.
- moonshine moon-medium-cpu4 t0: Actually reading stuff that's already written is not Really good example data that we're looking for here It would be best to just use one of the words come to mind off the At the top of my head. So that's I like what That would actually be the same as an dictating to the VP.
- moonshine moon-medium-cpu4-chunks t0: Actually reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use one of the words coming to mind off the top of my head. So that's I'm like what That would actually be the same as an Dictating to the computer
- moonshine moon-medium-cpu4-kt t0: Actually reading stuff that's already written is not Really the example data that we're looking for here It would be best to just use one of the words come to mind off the the top of my head. So that's I like what That would actually be the same as an dictating to the VP.
- moonshine moon-medium-cpu4-nodecode t0: Actually reading stuff that's already written is not Really good example data that we're looking for here. It would be best to just use one of the words come to mind often. At the top of my head. So that's I like what That would actually be same as an dictating to the VP.
- moonshine moon-medium-cpu8 t0: Actually reading stuff that's already written is not Really good example data that we're looking for here It would be best to just use one of the words come to mind off the At the top of my head. So that's I like what That would actually be the same as an dictating to the VP.
- moonshine moon-small-cpu0 t0: Actually reading stuff that's already written is not. Another example data that we're looking for here. It would be best to just use one of the words come to mind often. It's at the top of my head. So that's I'm like, what? That would actually be same as in Dictating to the computer.
- moonshine moon-small-cpu1 t0: Actually reading stuff that's already written is not. Really the example data that we're looking for here It would be best to just use one of the words coming to mind off the top of my head. So that's I'm like, what? That would actually be same as in Dictating to the computer.
- moonshine moon-small-cpu4 t0: Actually reading stuff that's already written is not. Another example data that we're looking for here. It would be best to just use one of the words come to mind often. It's at the top of my head. So that's I'm like, what? That would actually be same as in Dictating to the computer.
- moonshine moon-small-cpu4-kt t0: Actually reading stuff that's already written is not. Another example data that we're looking for here. It would be best to just use one of the words come to mind often. It's at the top of my head. So that's I'm like, what? That would actually be the same as in Dictating to the computer.
- moonshine moon-small-cpu8 t0: Actually reading stuff that's already written is not. Another example data that we're looking for here. It would be best to just use one of the words come to mind often. It's at the top of my head. So that's I'm like, what? That would actually be same as in Dictating to the computer.
- sherpa-onnx sherpa-nemo160 t2: Actually reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer
- sherpa-onnx sherpa-nemo160 t4: Actually reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer
- sherpa-onnx sherpa-nemo160 t8: Actually reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer
- sherpa-onnx sherpa-nemo560 t1: Actually reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer
- sherpa-onnx sherpa-nemo560 t2: Actually reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer
- sherpa-onnx sherpa-nemo560 t4: Actually reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer
- sherpa-onnx sherpa-nemo560 t8: Actually reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer
- sherpa-onnx sherpa-nemo560-rep t4: Actually reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer
- transcribe-cpp tcpp-R1-160ms t2: Actually, reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer.
- transcribe-cpp tcpp-R1-160ms t4: Actually, reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer.
- transcribe-cpp tcpp-R1-160ms t8: Actually, reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer.
- transcribe-cpp tcpp-R6-560ms t1: Actually, reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer.
- transcribe-cpp tcpp-R6-560ms t2: Actually, reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer.
- transcribe-cpp tcpp-R6-560ms t4: Actually, reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer.
- transcribe-cpp tcpp-R6-560ms t8: Actually, reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer.
- transcribe-cpp tcpp-R6-560ms-noblas t8: Actually, reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer.
- transcribe-cpp tcpp-R6-560ms-rep t4: Actually, reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer.

### sample5 (53.2 s)

Transcripts withheld: this recording is not in the repository.

