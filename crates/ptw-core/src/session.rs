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
use crate::typist::Typist;

/// What the audio thread sends: samples at [`crate::engine::SAMPLE_RATE`],
/// or a control message.
#[derive(Debug)]
pub enum Input {
    Audio(Vec<f32>),
    /// The Hotkey was released: Flush and type the rest.
    Stop,
    /// The Dictation was aborted: type nothing more.
    Abort,
}

/// How often to ask the Engine for text while audio keeps coming.
const DECODE_TICK: Duration = Duration::from_millis(100);

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    /// Everything typed, including the trailing space.
    pub typed: String,
    pub aborted: bool,
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
    let mut next_tick = Instant::now() + DECODE_TICK;
    loop {
        let timeout = next_tick.saturating_duration_since(Instant::now());
        match input.recv_timeout(timeout) {
            Ok(Input::Audio(samples)) => stream.feed(&samples),
            Ok(Input::Stop) | Err(RecvTimeoutError::Disconnected) => break,
            Ok(Input::Abort) => {
                outcome.aborted = true;
                return Ok(outcome);
            }
            Err(RecvTimeoutError::Timeout) => {}
        }
        if Instant::now() >= next_tick {
            emit(&mut *stream, &mut dictation, typist, &mut outcome);
            next_tick = Instant::now() + DECODE_TICK;
        }
    }
    let final_text = stream.finish();
    debug!(final_text, "flush");
    type_text(typist, &dictation.finish(&final_text), &mut outcome);
    Ok(outcome)
}

fn emit(
    stream: &mut dyn EngineStream,
    dictation: &mut Dictation,
    typist: &mut dyn Typist,
    outcome: &mut Outcome,
) {
    let transcript = stream.transcript();
    let text = dictation.update(&transcript.committed);
    type_text(typist, &text, outcome);
}

fn type_text(typist: &mut dyn Typist, text: &str, outcome: &mut Outcome) {
    if text.is_empty() {
        return;
    }
    match typist.type_text(text) {
        Ok(()) => outcome.typed.push_str(text),
        Err(e) => warn!(error = %e, text, "typing failed; text dropped"),
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
