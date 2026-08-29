# ptw (Push to Whisper)

Hold a key, talk, and the words appear in whatever app has focus while you are still talking. Linux-first, CPU-only, offline.

## Language

**Hotkey**:
The key or key combination whose hold starts a Dictation and whose release ends it.
_Avoid_: shortcut, binding, trigger

**Hold**:
The interval between Hotkey press and release. Audio is captured only during a Hold.
_Avoid_: recording, session, push

**Dictation**:
One Hold's worth of speech and the text it produces, from first audio sample to last typed character.
_Avoid_: transcription job, utterance, recording

**Flush**:
The end of a Dictation: after the Hotkey is released, audio already captured is still recognized to completion and typed before ptw goes idle.
_Avoid_: finalize, drain

**Engine**:
The speech recognizer that turns audio into text. ptw can host more than one.
_Avoid_: backend, model, recognizer

**Model**:
A weights file that an Engine loads. One Engine may support several Models.
_Avoid_: checkpoint, engine

**Lookahead**:
How much audio beyond a word the Engine waits for before committing that word. The latency-versus-accuracy knob.
_Avoid_: chunk size, right context, latency setting

**Committed text**:
Text the Engine promises never to revise. Only Committed text is ever typed.
_Avoid_: final, stable, confirmed

**Tentative text**:
The Engine's current best guess for the words after the Committed text. It may change and is never typed.
_Avoid_: partial, interim, preview, provisional

**Custom words**:
The user's list of words and phrases (names, jargon) that recognition tends to get wrong.
_Avoid_: vocabulary, dictionary, hotwords (that is one mechanism, not the concept)

**Correction**:
Replacing recognized words that are a close match for a Custom word with that Custom word.
_Avoid_: post-processing, fixup, substitution

**Hold-back**:
Committed words withheld from typing until enough following words have arrived to decide whether a Correction applies to them. One word longer than the longest Custom word, because the Engine splits words it does not know.
_Avoid_: buffer, delay window

**Key proxy**:
ptw's stand-in for the keyboard: it takes every key from the real keyboards, keeps the Hotkey for itself, and passes the rest on to the desktop.
_Avoid_: interceptor, remapper, grab (that is the mechanism)

**Deferral**:
The moment (up to 150 ms) the Key proxy waits after a Hotkey modifier press to see whether the rest of the Hotkey follows before the desktop is told about it.
_Avoid_: hold-back (that is the Correction term), debounce, delay

**Typist**:
The part of ptw that delivers text to the focused application.
_Avoid_: paster, injector, output, keyboard emulator

**Toggle mode**:
Starting and ending a Dictation with two separate taps instead of a Hold.
_Avoid_: latch, sticky
