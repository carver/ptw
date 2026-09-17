| engine | model | thr | files | load ms | chunk mean ms | chunk p95 ms (worst file) | chunk max ms | RTF | finalize ms mean/max | committed revisions | peak RSS MB | WER% vs Parakeet | median commit lag s (n words) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| moonshine | moon-medium-cpu0 | 0 | 5 | 1358 | 169.3 | 2067 | 5212 | 1.920 | 3 / 5 | 0 | 830 | 25.2 | 1.84 (191) |
| moonshine | moon-medium-cpu4 | 0 | 5 | 1490 | 102.3 | 1131 | 3210 | 1.179 | 2 / 3 | 0 | 833 | 25.2 | 1.84 (191) |
| moonshine | moon-medium-cpu8 | 0 | 5 | 1095 | 147.3 | 1719 | 4334 | 1.772 | 2 / 4 | 0 | 828 | 25.2 | 1.84 (191) |
| moonshine | moon-small-cpu0 | 0 | 5 | 467 | 106.8 | 946 | 3894 | 1.238 | 2 / 5 | 0 | 486 | 30.4 | 2.04 (146) |
| moonshine | moon-small-cpu4 | 0 | 5 | 298 | 47.8 | 446 | 1848 | 0.562 | 1 / 2 | 0 | 472 | 30.4 | 2.00 (145) |
| moonshine | moon-small-cpu8 | 0 | 5 | 378 | 96.9 | 1055 | 3381 | 1.127 | 4 / 6 | 0 | 477 | 30.4 | 2.00 (145) |
| sherpa-onnx | sherpa-nemo160 | 4 | 5 | 3305 | 119.5 | 422 | 1281 | 1.478 | 1085 / 1972 | 0 | 938 | 15.6 | 0.64 (216) |
| sherpa-onnx | sherpa-nemo160 | 8 | 5 | 3242 | 178.1 | 727 | 1496 | 2.240 | 827 / 1357 | 0 | 962 | 15.6 | 0.64 (216) |
| sherpa-onnx | sherpa-nemo560 | 4 | 5 | 4681 | 34.5 | 328 | 360 | 0.428 | 227 / 233 | 0 | 933 | 15.6 | 0.80 (233) |
| sherpa-onnx | sherpa-nemo560 | 8 | 5 | 3385 | 47.1 | 466 | 717 | 0.591 | 254 / 349 | 0 | 952 | 15.6 | 0.80 (233) |
| transcribe-cpp | tcpp-R1-160ms | 4 | 5 | 1075 | 57.9 | 148 | 208 | 0.723 | 154 / 189 | 0 | 1097 | 13.3 | 0.64 (221) |
| transcribe-cpp | tcpp-R1-160ms | 8 | 5 | 1064 | 87.5 | 501 | 6904 | 1.348 | 179 / 237 | 0 | 1090 | 13.3 | 0.64 (221) |
| transcribe-cpp | tcpp-R6-560ms | 4 | 5 | 1278 | 27.4 | 215 | 424 | 0.353 | 191 / 235 | 0 | 1107 | 14.1 | 0.80 (229) |
| transcribe-cpp | tcpp-R6-560ms | 8 | 5 | 2177 | 43.4 | 335 | 610 | 0.526 | 243 / 293 | 0 | 1108 | 14.1 | 0.80 (229) |
| transcribe-cpp | tcpp-R6-560ms-noblas | 8 | 5 | 1149 | 36.4 | 261 | 4566 | 0.480 | 1139 / 4464 | 0 | 1103 | 13.7 | 0.80 (230) |

## Per-file RTF

| run | sample1 | sample2 | sample3 | sample4 | sample5 |
|---|---|---|---|---|---|
| moonshine moon-medium-cpu0 t0 | 2.723 | 2.293 | 2.354 | 1.907 | 1.310 |
| moonshine moon-medium-cpu4 t0 | 1.687 | 1.306 | 1.447 | 1.044 | 0.911 |
| moonshine moon-medium-cpu8 t0 | 1.511 | 2.146 | 2.399 | 1.806 | 1.354 |
| moonshine moon-small-cpu0 t0 | 1.555 | 1.443 | 1.467 | 1.318 | 0.899 |
| moonshine moon-small-cpu4 t0 | 0.638 | 0.654 | 0.741 | 0.529 | 0.429 |
| moonshine moon-small-cpu8 t0 | 1.036 | 1.600 | 1.687 | 1.028 | 0.709 |
| sherpa-onnx sherpa-nemo160 t4 | 1.252 | 1.935 | 1.660 | 1.306 | 1.320 |
| sherpa-onnx sherpa-nemo160 t8 | 2.126 | 1.615 | 2.325 | 2.823 | 2.248 |
| sherpa-onnx sherpa-nemo560 t4 | 0.474 | 0.402 | 0.420 | 0.433 | 0.429 |
| sherpa-onnx sherpa-nemo560 t8 | 0.602 | 0.505 | 0.514 | 0.725 | 0.599 |
| transcribe-cpp tcpp-R1-160ms t4 | 0.739 | 0.711 | 0.673 | 0.779 | 0.719 |
| transcribe-cpp tcpp-R1-160ms t8 | 0.810 | 0.793 | 0.756 | 0.770 | 2.341 |
| transcribe-cpp tcpp-R6-560ms t4 | 0.300 | 0.281 | 0.377 | 0.381 | 0.378 |
| transcribe-cpp tcpp-R6-560ms t8 | 0.565 | 0.521 | 0.558 | 0.614 | 0.459 |
| transcribe-cpp tcpp-R6-560ms-noblas t8 | 0.362 | 0.385 | 0.359 | 0.627 | 0.544 |

## Per-file WER% vs Parakeet offline

| run | sample1 | sample2 | sample3 | sample4 | sample5 |
|---|---|---|---|---|---|
| moonshine moon-medium-cpu0 t0 | 8.8 | 22.4 | 37.5 | 23.5 | 29.5 |
| moonshine moon-medium-cpu4 t0 | 8.8 | 22.4 | 37.5 | 23.5 | 29.5 |
| moonshine moon-medium-cpu8 t0 | 8.8 | 22.4 | 37.5 | 23.5 | 29.5 |
| moonshine moon-small-cpu0 t0 | 8.8 | 31.3 | 50.0 | 23.5 | 33.3 |
| moonshine moon-small-cpu4 t0 | 8.8 | 31.3 | 50.0 | 23.5 | 33.3 |
| moonshine moon-small-cpu8 t0 | 8.8 | 31.3 | 50.0 | 23.5 | 33.3 |
| sherpa-onnx sherpa-nemo160 t4 | 11.8 | 13.4 | 45.0 | 0.0 | 14.1 |
| sherpa-onnx sherpa-nemo160 t8 | 11.8 | 13.4 | 45.0 | 0.0 | 14.1 |
| sherpa-onnx sherpa-nemo560 t4 | 11.8 | 11.9 | 45.0 | 0.0 | 15.4 |
| sherpa-onnx sherpa-nemo560 t8 | 11.8 | 11.9 | 45.0 | 0.0 | 15.4 |
| transcribe-cpp tcpp-R1-160ms t4 | 11.8 | 11.9 | 32.5 | 0.0 | 14.1 |
| transcribe-cpp tcpp-R1-160ms t8 | 11.8 | 11.9 | 32.5 | 0.0 | 14.1 |
| transcribe-cpp tcpp-R6-560ms t4 | 8.8 | 11.9 | 30.0 | 0.0 | 19.2 |
| transcribe-cpp tcpp-R6-560ms t8 | 8.8 | 11.9 | 30.0 | 0.0 | 19.2 |
| transcribe-cpp tcpp-R6-560ms-noblas t8 | 5.9 | 11.9 | 30.0 | 0.0 | 19.2 |

## Commit lag examples (word, spoken at s per Parakeet, committed at s, lag s)

- moonshine moon-medium-cpu0 t0: i 0.40->2.24 (+1.84); probably 13.28->14.56 (+1.28); wait 13.84->16.24 (+2.40); i 6.88->10.64 (+3.76); not 3.92->4.48 (+0.56); be 20.48->22.40 (+1.92)
- moonshine moon-medium-cpu4 t0: i 0.40->2.24 (+1.84); probably 13.28->14.56 (+1.28); wait 13.84->16.24 (+2.40); i 6.88->10.64 (+3.76); not 3.92->4.48 (+0.56); be 20.48->22.40 (+1.92)
- moonshine moon-medium-cpu8 t0: i 0.40->2.24 (+1.84); probably 13.28->14.56 (+1.28); wait 13.84->16.24 (+2.40); i 6.88->10.64 (+3.76); not 3.92->4.48 (+0.56); be 20.48->22.40 (+1.92)
- moonshine moon-small-cpu0 t0: suppose 0.96->2.24 (+1.28); little 11.68->12.32 (+0.64); this 5.92->7.84 (+1.92); you 9.04->10.64 (+1.60); example 5.04->7.84 (+2.80); that's 16.40->17.36 (+0.96)
- moonshine moon-small-cpu4 t0: suppose 0.96->2.24 (+1.28); little 11.68->12.32 (+0.64); this 5.92->7.84 (+1.92); you 9.04->10.64 (+1.60); data 5.76->7.84 (+2.08); like 18.08->19.04 (+0.96)
- moonshine moon-small-cpu8 t0: suppose 0.96->2.24 (+1.28); little 11.68->12.32 (+0.64); this 5.92->7.84 (+1.92); you 9.04->10.64 (+1.60); data 5.76->7.84 (+2.08); like 18.08->19.04 (+0.96)
- sherpa-onnx sherpa-nemo160 t4: i 0.40->2.56 (+2.16); because 4.00->4.64 (+0.64); to 18.32->18.56 (+0.24); grab 20.64->21.28 (+0.64); so 15.60->16.80 (+1.20); wake 10.96->11.68 (+0.72)
- sherpa-onnx sherpa-nemo160 t8: i 0.40->2.56 (+2.16); because 4.00->4.64 (+0.64); to 18.32->18.56 (+0.24); grab 20.64->21.28 (+0.64); so 15.60->16.80 (+1.20); wake 10.96->11.68 (+0.72)
- sherpa-onnx sherpa-nemo560 t4: i 0.40->2.96 (+2.56); because 4.00->4.64 (+0.64); be 18.40->18.64 (+0.24); no 16.24->16.96 (+0.72); my 13.04->13.60 (+0.56); want 10.80->11.36 (+0.56)
- sherpa-onnx sherpa-nemo560 t8: i 0.40->2.96 (+2.56); because 4.00->4.64 (+0.64); be 18.40->18.64 (+0.24); no 16.24->16.96 (+0.72); my 13.04->13.60 (+0.56); want 10.80->11.36 (+0.56)
- transcribe-cpp tcpp-R1-160ms t4: i 0.40->2.40 (+2.00); grill 2.72->3.84 (+1.12); it 17.60->18.24 (+0.64); no 16.24->16.80 (+0.56); of 12.88->13.44 (+0.56); don't 10.64->11.36 (+0.72)
- transcribe-cpp tcpp-R1-160ms t8: i 0.40->2.40 (+2.00); grill 2.72->3.84 (+1.12); it 17.60->18.24 (+0.64); no 16.24->16.80 (+0.56); of 12.88->13.44 (+0.56); don't 10.64->11.36 (+0.72)
- transcribe-cpp tcpp-R6-560ms t4: suppose 0.96->2.24 (+1.28); it 4.16->5.04 (+0.88); to 18.32->18.48 (+0.16); no 16.24->16.80 (+0.56); top 12.64->13.44 (+0.80); is 9.84->10.64 (+0.80)
- transcribe-cpp tcpp-R6-560ms t8: suppose 0.96->2.24 (+1.28); it 4.16->5.04 (+0.88); to 18.32->18.48 (+0.16); no 16.24->16.80 (+0.56); top 12.64->13.44 (+0.80); is 9.84->10.64 (+0.80)
- transcribe-cpp tcpp-R6-560ms-noblas t8: suppose 0.96->2.24 (+1.28); because 4.00->4.48 (+0.48); seem 18.08->18.48 (+0.40); has 16.08->16.80 (+0.72); the 12.40->13.44 (+1.04); else 9.52->10.08 (+0.56)

## Revisions of committed text


## Transcripts

### sample1 (15.4 s)

- **Parakeet TDT 0.6b v3 int8 offline (reference, proofread me)**: I suppose I should take out the mouth card before I start talking here. This isn't the best time to capture my voice because I'm a little tired, so my voice is probably creaky.
- moonshine moon-medium-cpu0 t0: I suppose I should. Take out The mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little... I'm tired, so my voice is probably crooked.
- moonshine moon-medium-cpu4 t0: I suppose I should. Take out The mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little... I'm tired, so my voice is probably crooked.
- moonshine moon-medium-cpu8 t0: I suppose I should. Take out The mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little... I'm tired, so my voice is probably crooked.
- moonshine moon-small-cpu0 t0: Hey, suppose I should. Take out. The mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little... Tired so my voice is probably crooked
- moonshine moon-small-cpu4 t0: Hey, suppose I should. Take out. The mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little... Tired so my voice is probably crooked
- moonshine moon-small-cpu8 t0: Hey, suppose I should. Take out. The mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little... Tired so my voice is probably crooked
- sherpa-onnx sherpa-nemo160 t4: Hey, supposedly I should take out the mouth card before I start talking here. This isn't the best time to capture my voice cause I'm a little tired so my voice is probably cirky
- sherpa-onnx sherpa-nemo160 t8: Hey, supposedly I should take out the mouth card before I start talking here. This isn't the best time to capture my voice cause I'm a little tired so my voice is probably cirky
- sherpa-onnx sherpa-nemo560 t4: Hey, supposedly I should take out the mouth card before I start talking here. This isn't the best time to capture my voice cause I'm a little tired so my voice is probably cooky
- sherpa-onnx sherpa-nemo560 t8: Hey, supposedly I should take out the mouth card before I start talking here. This isn't the best time to capture my voice cause I'm a little tired so my voice is probably cooky
- transcribe-cpp tcpp-R1-160ms t4: Hey, supposed I should take out the mouth card before I start talking here This isn't the best time to capture my voice because I'm a little tired and so my voice is probably cricky.
- transcribe-cpp tcpp-R1-160ms t8: Hey, supposed I should take out the mouth card before I start talking here This isn't the best time to capture my voice because I'm a little tired and so my voice is probably cricky.
- transcribe-cpp tcpp-R6-560ms t4: Hey, suppose I should take out the mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little tired and so my voice is probably creaky.
- transcribe-cpp tcpp-R6-560ms t8: Hey, suppose I should take out the mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little tired and so my voice is probably creaky.
- transcribe-cpp tcpp-R6-560ms-noblas t8: Hey, suppose I should take out the mouth guard before I start talking here. This isn't the best time to capture my voice because I'm a little tired, so my voice is probably creaky.

### sample2 (26.0 s)

- **Parakeet TDT 0.6b v3 int8 offline (reference, proofread me)**: I suppose I should have said grill of docs because it seems important to write the results of this planning session down for future sessions to have background. Plant write down these findings when it resolves. Wait, should I just be using Handy? I wasn't using it because it doesn't seem to be streaming. If it streams the results maybe it's findings directly instead of building PT.
- moonshine moon-medium-cpu0 t0: I suppose I should have said "Grill with Docs" because it seems important to write the results of this planning session down. for future sessions to head back. Plan right down these bindings when it resolves. Wait should I just be using handy I love using it because it doesn't seem to be streaming If it's true as a result, maybe it's fine if it's directly instead of building PT
- moonshine moon-medium-cpu4 t0: I suppose I should have said "Grill with Docs" because it seems important to write the results of this planning session down. for future sessions to head back. Plan right down these bindings when it resolves. Wait should I just be using handy I love using it because it doesn't seem to be streaming If it's true as a result, maybe it's fine if it's directly instead of building PT
- moonshine moon-medium-cpu8 t0: I suppose I should have said "Grill with Docs" because it seems important to write the results of this planning session down. for future sessions to head back. Plan right down these bindings when it resolves. Wait should I just be using handy I love using it because it doesn't seem to be streaming If it's true as a result, maybe it's fine if it's directly instead of building PT
- moonshine moon-small-cpu0 t0: I suppose I should have said grill of dogs, because it seems important to write the results of this planning session down. For the future sessions that back. Planetary felony's findings were in the applesauce. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming. If it's true as a result, maybe it's fine if it's directly instead of building PT
- moonshine moon-small-cpu4 t0: I suppose I should have said grill of dogs, because it seems important to write the results of this planning session down. For the future sessions that back. Planetary felony's findings were in the applesauce. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming. If it's true as a result, maybe it's fine if it's directly instead of building PT
- moonshine moon-small-cpu8 t0: I suppose I should have said grill of dogs, because it seems important to write the results of this planning session down. For the future sessions that back. Planetary felony's findings were in the applesauce. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming. If it's true as a result, maybe it's fine if it's directly instead of building PT
- sherpa-onnx sherpa-nemo160 t4: I suppose I should have said grill with dogs because it seems important to write the results of this planning session down for future sessions to have background plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming but if it streams the results maybe it's fine to use directly instead of building PTW
- sherpa-onnx sherpa-nemo160 t8: I suppose I should have said grill with dogs because it seems important to write the results of this planning session down for future sessions to have background plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming but if it streams the results maybe it's fine to use directly instead of building PTW
- sherpa-onnx sherpa-nemo560 t4: I suppose I should have said grill with docs because it seems important to write the results of this planning session down for future sessions to have background plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams the results maybe it's fine to use directly instead of building PTW
- sherpa-onnx sherpa-nemo560 t8: I suppose I should have said grill with docs because it seems important to write the results of this planning session down for future sessions to have background plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams the results maybe it's fine to use directly instead of building PTW
- transcribe-cpp tcpp-R1-160ms t4: I suppose I should have said grill with docks because it seems important to write the results of this planning session down for future sessions to have background plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams the results, maybe it's finding you directly instead of building PTW.
- transcribe-cpp tcpp-R1-160ms t8: I suppose I should have said grill with docks because it seems important to write the results of this planning session down for future sessions to have background plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams the results, maybe it's finding you directly instead of building PTW.
- transcribe-cpp tcpp-R6-560ms t4: I suppose I should have said grill with docs because it seems important to write the results of this planning session down for future sessions to have background. Plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams or results, maybe it's finding it directly instead of building PTW.
- transcribe-cpp tcpp-R6-560ms t8: I suppose I should have said grill with docs because it seems important to write the results of this planning session down for future sessions to have background. Plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams or results, maybe it's finding it directly instead of building PTW.
- transcribe-cpp tcpp-R6-560ms-noblas t8: I suppose I should have said grill with docs because it seems important to write the results of this planning session down for future sessions to have background. Plan to write down these findings when it resolves. Wait, should I just be using handy? I wasn't using it because it doesn't seem to be streaming, but if it streams or results, maybe it's finding it directly instead of building PTW.

### sample3 (24.9 s)

- **Parakeet TDT 0.6b v3 int8 offline (reference, proofread me)**: So what is it that we want here? I guess I could just read back something that you wrote to me. Gnome 46, the global shortcuts portal has no back end. Gnome Shell's own grab API is closed three parties.
- moonshine moon-medium-cpu0 t0: So? What is it that we... Here Alright, I guess I could just read back something that you wrote to me. Hotkey with release detection. 0.46 the global shortcuts portal has no backend. And domeshell's own grab api is close to 3.0
- moonshine moon-medium-cpu4 t0: So? What is it that we... Here Alright, I guess I could just read back something that you wrote to me. Hotkey with release detection. 0.46 the global shortcuts portal has no backend. And domeshell's own grab api is close to 3.0
- moonshine moon-medium-cpu8 t0: So? What is it that we... Here Alright, I guess I could just read back something that you wrote to me. Hotkey with release detection. 0.46 the global shortcuts portal has no backend. And domeshell's own grab api is close to 3.0
- moonshine moon-small-cpu0 t0: So What is it that we... Here. I guess I could just read back something that you wrote to me. A lot of key with release detection. Now in 46, the global shortcuts portal has no backend. They were out to 1148. Nome shells own grab api is closed throughput.
- moonshine moon-small-cpu4 t0: So What is it that we... Here. I guess I could just read back something that you wrote to me. A lot of key with release detection. On 946, the global shortcuts portal has no backend. They were acting at 948. Nome shells own grab api is closed throughput.
- moonshine moon-small-cpu8 t0: So What is it that we... Here. I guess I could just read back something that you wrote to me. A lot of key with release detection. On 946, the global shortcuts portal has no backend. They were acting at 948. Nome shells own grab api is closed throughput.
- sherpa-onnx sherpa-nemo160 t4: So what is it that we want here? I guess I could just read back something that you wrote to me. Hotkey with release detection on GNOME forty six, the global shortcuts portal has no backend interrupting GNOME forty eight and GNOME shells on Grab API is close to third parties.
- sherpa-onnx sherpa-nemo160 t8: So what is it that we want here? I guess I could just read back something that you wrote to me. Hotkey with release detection on GNOME forty six, the global shortcuts portal has no backend interrupting GNOME forty eight and GNOME shells on Grab API is close to third parties.
- sherpa-onnx sherpa-nemo560 t4: So what is it that we want here? I guess I could just read back something that you wrote to me hotkey with release detection on GNOME forty six, the global shortcuts portal has no backend interrupting GNOME forty eight and GNOME shells on Grab API is close to third parties.
- sherpa-onnx sherpa-nemo560 t8: So what is it that we want here? I guess I could just read back something that you wrote to me hotkey with release detection on GNOME forty six, the global shortcuts portal has no backend interrupting GNOME forty eight and GNOME shells on Grab API is close to third parties.
- transcribe-cpp tcpp-R1-160ms t4: So what is it that we want here? I guess I could just read back something that you wrote to me. Hotkey with release detection on GNOME 46, the global shortcuts portal has no back end. It erupted GNOME 48. And GNOME shell's own grab API is close to third parties.
- transcribe-cpp tcpp-R1-160ms t8: So what is it that we want here? I guess I could just read back something that you wrote to me. Hotkey with release detection on GNOME 46, the global shortcuts portal has no back end. It erupted GNOME 48. And GNOME shell's own grab API is close to third parties.
- transcribe-cpp tcpp-R6-560ms t4: So what is it that we want here? I guess I could just read back something that you wrote to me hotkey with release detection on GNOME 46 the global shortcuts portal has no back end interrupted GNOME 48 and GNOME shell's own grab API is close to third parties
- transcribe-cpp tcpp-R6-560ms t8: So what is it that we want here? I guess I could just read back something that you wrote to me hotkey with release detection on GNOME 46 the global shortcuts portal has no back end interrupted GNOME 48 and GNOME shell's own grab API is close to third parties
- transcribe-cpp tcpp-R6-560ms-noblas t8: So what is it that we want here? I guess I could just read back something that you wrote to me hotkey with release detection on GNOME 46 the global shortcuts portal has no back end interrupted GNOME 48 and GNOME shell's own grab API is close to third parties

### sample4 (26.5 s)

- **Parakeet TDT 0.6b v3 int8 offline (reference, proofread me)**: Actually reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer.
- moonshine moon-medium-cpu0 t0: Actually reading stuff that's already written is not Really good example data that we're looking for here It would be best to just use one of the words come to mind off the At the top of my head. So that's I like what That would actually be the same as an dictating to the VP.
- moonshine moon-medium-cpu4 t0: Actually reading stuff that's already written is not Really good example data that we're looking for here It would be best to just use one of the words come to mind off the At the top of my head. So that's I like what That would actually be the same as an dictating to the VP.
- moonshine moon-medium-cpu8 t0: Actually reading stuff that's already written is not Really good example data that we're looking for here It would be best to just use one of the words come to mind off the At the top of my head. So that's I like what That would actually be the same as an dictating to the VP.
- moonshine moon-small-cpu0 t0: Actually reading stuff that's already written is not. Another example data that we're looking for here. It would be best to just use one of the words come to mind often. It's at the top of my head. So that's I'm like, what? That would actually be same as in Dictating to the computer.
- moonshine moon-small-cpu4 t0: Actually reading stuff that's already written is not. Another example data that we're looking for here. It would be best to just use one of the words come to mind often. It's at the top of my head. So that's I'm like, what? That would actually be same as in Dictating to the computer.
- moonshine moon-small-cpu8 t0: Actually reading stuff that's already written is not. Another example data that we're looking for here. It would be best to just use one of the words come to mind often. It's at the top of my head. So that's I'm like, what? That would actually be same as in Dictating to the computer.
- sherpa-onnx sherpa-nemo160 t4: Actually reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer
- sherpa-onnx sherpa-nemo160 t8: Actually reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer
- sherpa-onnx sherpa-nemo560 t4: Actually reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer
- sherpa-onnx sherpa-nemo560 t8: Actually reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer
- transcribe-cpp tcpp-R1-160ms t4: Actually, reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer.
- transcribe-cpp tcpp-R1-160ms t8: Actually, reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer.
- transcribe-cpp tcpp-R6-560ms t4: Actually, reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer.
- transcribe-cpp tcpp-R6-560ms t8: Actually, reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer.
- transcribe-cpp tcpp-R6-560ms-noblas t8: Actually, reading stuff that's already written is not really the example data that we're looking for here. It would be best to just use whatever words come to mind off the top of my head. So that's more like what I would actually be saying as I'm dictating to the computer.

### sample5 (53.2 s)

Transcripts withheld: this recording is not in the repository.

