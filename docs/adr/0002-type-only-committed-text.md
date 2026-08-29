---
status: accepted
---

# Type only Committed text, never retract

Streaming recognizers revise recent words. We could type Tentative text and fix it with backspaces, but backspace means different things in terminals, editors with auto-pairing and completion, and chat boxes, and a wrong count corrupts the user's document. So the Typist only ever appends Committed text. Custom-word Correction, which needs to see a few following words, is handled by a Hold-back rather than by retracting. The cost is that typed text lags the voice by the Engine's Lookahead plus the Hold-back; with a cache-aware transducer that is a few hundred milliseconds, not seconds.

## Considered options

- Type Tentative text and backspace-correct: snappiest, rejected for the corruption risk above.
- Type only on release (Handy's approach): safe, but not streaming.
- Show Tentative text in an overlay: not possible on GNOME without a shell extension (no layer-shell, no always-on-top), so not a v1 option.
