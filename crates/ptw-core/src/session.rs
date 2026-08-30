//! Runs one Dictation: audio chunks in, typed text out, until told to stop.
//!
//! The desktop crate owns the threads that produce audio and key events;
//! this loop only needs channels, so it is tested with a scripted Engine.

use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use tracing::{debug, warn};

use crate::correction::CustomWords;
use crate::dictation::Dictation;
use crate::engine::{Engine, EngineError, EngineStream};
use crate::typist::{Typist, TypistError};

/// What the audio thread sends: samples at [`crate::engine::SAMPLE_RATE`],
/// or a control message.
#[derive(Debug)]
pub enum Input {
    Audio(Vec<f32>),
    /// A modifier key went down (`true`) or the last one came up. While
    /// one is down, text waits: the compositor would turn it into
    /// shortcuts (Alt+Space opens GNOME's window menu).
    ModifiersHeld(bool),
    /// The Hotkey was released: Flush and type the rest.
    Stop,
    /// The Dictation was aborted: type nothing more.
    Abort,
}

/// How often to ask the Engine for text while audio keeps coming.
const DECODE_TICK: Duration = Duration::from_millis(100);

/// How long after Stop to wait for the modifiers to come up before the
/// remaining text is dropped rather than typed as shortcuts.
const RELEASE_GRACE: Duration = Duration::from_secs(5);

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    /// Everything typed, including the trailing space.
    pub typed: String,
    /// Text that was ready but never typed because modifiers stayed down.
    pub dropped: String,
    pub aborted: bool,
}

/// Holds text back while a modifier key is down.
#[derive(Default)]
struct Gate {
    modifiers_held: bool,
    pending: String,
}

impl Gate {
    fn offer(&mut self, text: &str, typist: &mut dyn Typist, outcome: &mut Outcome) {
        self.pending.push_str(text);
        if !self.modifiers_held {
            self.flush(typist, outcome);
        }
    }

    fn set_modifiers_held(&mut self, held: bool, typist: &mut dyn Typist, outcome: &mut Outcome) {
        self.modifiers_held = held;
        if !held {
            self.flush(typist, outcome);
        }
    }

    fn flush(&mut self, typist: &mut dyn Typist, outcome: &mut Outcome) {
        if self.pending.is_empty() {
            return;
        }
        let text = std::mem::take(&mut self.pending);
        match type_with_reconnect(typist, &text) {
            Ok(()) => {
                debug!(text, "typed");
                outcome.typed.push_str(&text);
            }
            Err(e) => warn!(error = %e, text, "typing failed; text dropped"),
        }
    }
}

/// A typing failure usually means the route to the desktop died while ptw
/// was idle (suspend kills the portal session), so reconnect and retype
/// the chunk once before giving it up.
fn type_with_reconnect(typist: &mut dyn Typist, text: &str) -> Result<(), TypistError> {
    let Err(first) = typist.type_text(text) else {
        return Ok(());
    };
    warn!(error = %first, "typing failed; reconnecting the Typist");
    if let Err(e) = typist.reconnect() {
        warn!(error = %e, "Typist reconnect failed");
        return Err(first);
    }
    typist.type_text(text)
}

/// Feeds audio from `input` into a fresh stream of `engine` and types what
/// the Engine commits until [`Input::Stop`] or [`Input::Abort`] arrives.
///
/// Returns when the Dictation is over or the sender hangs up (treated as Stop).
pub fn run(
    engine: &dyn Engine,
    words: &CustomWords,
    input: &Receiver<Input>,
    typist: &mut dyn Typist,
) -> Result<Outcome, EngineError> {
    let mut stream = engine.open_stream()?;
    let mut dictation = Dictation::new(words.clone());
    let mut outcome = Outcome::default();
    let mut gate = Gate::default();
    let mut next_tick = Instant::now() + DECODE_TICK;
    loop {
        let timeout = next_tick.saturating_duration_since(Instant::now());
        match input.recv_timeout(timeout) {
            Ok(Input::Audio(samples)) => stream.feed(&samples),
            Ok(Input::ModifiersHeld(held)) => {
                gate.set_modifiers_held(held, typist, &mut outcome);
            }
            Ok(Input::Stop) | Err(RecvTimeoutError::Disconnected) => break,
            Ok(Input::Abort) => {
                outcome.aborted = true;
                return Ok(outcome);
            }
            Err(RecvTimeoutError::Timeout) => {}
        }
        if Instant::now() >= next_tick {
            emit(
                &mut *stream,
                &mut dictation,
                typist,
                &mut gate,
                &mut outcome,
            );
            next_tick = Instant::now() + DECODE_TICK;
        }
    }
    let final_text = stream.finish();
    debug!(final_text, "flush");
    gate.offer(&dictation.finish(&final_text), typist, &mut outcome);
    wait_for_modifiers(input, &mut gate, typist, &mut outcome);
    Ok(outcome)
}

fn emit(
    stream: &mut dyn EngineStream,
    dictation: &mut Dictation,
    typist: &mut dyn Typist,
    gate: &mut Gate,
    outcome: &mut Outcome,
) {
    let transcript = stream.transcript();
    let text = dictation.update(&transcript.committed);
    gate.offer(&text, typist, outcome);
}

/// After Stop, the chord's own modifier is usually still on its way up.
fn wait_for_modifiers(
    input: &Receiver<Input>,
    gate: &mut Gate,
    typist: &mut dyn Typist,
    outcome: &mut Outcome,
) {
    let deadline = Instant::now() + RELEASE_GRACE;
    while gate.modifiers_held && !gate.pending.is_empty() {
        let timeout = deadline.saturating_duration_since(Instant::now());
        match input.recv_timeout(timeout) {
            Ok(Input::ModifiersHeld(held)) => gate.set_modifiers_held(held, typist, outcome),
            Ok(Input::Abort) => break,
            Ok(Input::Audio(_) | Input::Stop) => {}
            Err(RecvTimeoutError::Timeout | RecvTimeoutError::Disconnected) => break,
        }
    }
    if !gate.pending.is_empty() {
        outcome.dropped = std::mem::take(&mut gate.pending);
        warn!(
            text = outcome.dropped,
            "modifier keys still held; text dropped rather than typed as shortcuts"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::ScriptedEngine;
    use crate::typist::RecordingTypist;
    use std::sync::mpsc;

    #[test]
    fn types_committed_words_as_they_arrive_then_flushes() {
        let engine = ScriptedEngine::word_by_word("hello there Kaitlin");
        let words = CustomWords::new(["Caitlyn"]);
        let (tx, rx) = mpsc::channel();
        let mut typist = RecordingTypist::default();
        let feeder = std::thread::spawn(move || {
            for _ in 0..4 {
                tx.send(Input::Audio(vec![0.0; 1280])).unwrap();
                std::thread::sleep(Duration::from_millis(120));
            }
            tx.send(Input::Stop).unwrap();
        });
        let outcome = run(&engine, &words, &rx, &mut typist).unwrap();
        feeder.join().unwrap();
        assert_eq!(typist.text(), "hello there Caitlyn ");
        assert_eq!(
            outcome,
            Outcome {
                typed: "hello there Caitlyn ".into(),
                dropped: String::new(),
                aborted: false
            }
        );
        assert!(
            typist.chunks.len() >= 2,
            "streamed in pieces, got {:?}",
            typist.chunks
        );
    }

    #[test]
    fn text_waits_until_the_modifiers_come_up() {
        let engine = ScriptedEngine::word_by_word("hold to talk");
        let (tx, rx) = mpsc::channel();
        let mut typist = RecordingTypist::default();
        let feeder = std::thread::spawn(move || {
            tx.send(Input::ModifiersHeld(true)).unwrap();
            for _ in 0..4 {
                tx.send(Input::Audio(vec![0.0; 1280])).unwrap();
                std::thread::sleep(Duration::from_millis(120));
            }
            tx.send(Input::Stop).unwrap();
            std::thread::sleep(Duration::from_millis(100));
            tx.send(Input::ModifiersHeld(false)).unwrap();
            std::thread::sleep(Duration::from_millis(100));
        });
        let outcome = run(&engine, &CustomWords::default(), &rx, &mut typist).unwrap();
        feeder.join().unwrap();
        assert_eq!(typist.chunks, vec!["hold to talk ".to_string()]);
        assert_eq!(outcome.typed, "hold to talk ");
        assert_eq!(outcome.dropped, "");
    }

    #[test]
    fn text_is_dropped_when_the_modifiers_never_come_up() {
        let engine = ScriptedEngine::word_by_word("one two");
        let (tx, rx) = mpsc::channel();
        let mut typist = RecordingTypist::default();
        tx.send(Input::ModifiersHeld(true)).unwrap();
        tx.send(Input::Audio(vec![0.0; 1280])).unwrap();
        tx.send(Input::Stop).unwrap();
        drop(tx);
        let outcome = run(&engine, &CustomWords::default(), &rx, &mut typist).unwrap();
        assert_eq!(typist.text(), "");
        assert_eq!(outcome.typed, "");
        assert_eq!(outcome.dropped, "one two ");
    }

    /// Fails every keystroke until reconnected, like a portal session
    /// that suspend killed.
    #[derive(Default)]
    struct DeadSessionTypist {
        broken: bool,
        reconnects: usize,
        inner: RecordingTypist,
    }

    impl Typist for DeadSessionTypist {
        fn type_text(&mut self, text: &str) -> Result<(), TypistError> {
            if self.broken {
                return Err(TypistError::Backend("Invalid session".into()));
            }
            self.inner.type_text(text)
        }

        fn reconnect(&mut self) -> Result<(), TypistError> {
            self.reconnects += 1;
            self.broken = false;
            Ok(())
        }
    }

    #[test]
    fn reconnects_and_retypes_when_typing_fails() {
        let engine = ScriptedEngine::word_by_word("back from suspend");
        let (tx, rx) = mpsc::channel();
        let mut typist = DeadSessionTypist {
            broken: true,
            ..Default::default()
        };
        tx.send(Input::Audio(vec![0.0; 1280])).unwrap();
        tx.send(Input::Stop).unwrap();
        let outcome = run(&engine, &CustomWords::default(), &rx, &mut typist).unwrap();
        assert_eq!(typist.inner.text(), "back from suspend ");
        assert_eq!(typist.reconnects, 1);
        assert_eq!(outcome.typed, "back from suspend ");
    }

    /// A Typist without a reconnect path (the trait default) keeps
    /// today's behavior: warn and drop.
    struct BrokenTypist;

    impl Typist for BrokenTypist {
        fn type_text(&mut self, _text: &str) -> Result<(), TypistError> {
            Err(TypistError::Backend("no route to desktop".into()))
        }
    }

    #[test]
    fn drops_text_when_reconnect_is_not_supported() {
        let engine = ScriptedEngine::word_by_word("lost words");
        let (tx, rx) = mpsc::channel();
        tx.send(Input::Audio(vec![0.0; 1280])).unwrap();
        tx.send(Input::Stop).unwrap();
        let outcome = run(&engine, &CustomWords::default(), &rx, &mut BrokenTypist).unwrap();
        assert_eq!(outcome.typed, "");
    }

    #[test]
    fn abort_types_nothing_more() {
        let engine = ScriptedEngine::word_by_word("one two three");
        let (tx, rx) = mpsc::channel();
        let mut typist = RecordingTypist::default();
        tx.send(Input::Audio(vec![0.0; 1280])).unwrap();
        tx.send(Input::Abort).unwrap();
        let outcome = run(&engine, &CustomWords::default(), &rx, &mut typist).unwrap();
        assert!(outcome.aborted);
        assert_eq!(typist.text(), "");
    }

    #[test]
    fn a_hung_up_sender_counts_as_stop() {
        let engine = ScriptedEngine::word_by_word("done");
        let (tx, rx) = mpsc::channel();
        drop(tx);
        let mut typist = RecordingTypist::default();
        let outcome = run(&engine, &CustomWords::default(), &rx, &mut typist).unwrap();
        assert_eq!(outcome.typed, "done ");
    }
}
