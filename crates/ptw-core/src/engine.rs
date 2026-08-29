//! The Engine: audio in, Committed and Tentative text out.

use std::fmt;

/// Every Engine takes 16 kHz mono f32 samples in [-1, 1].
pub const SAMPLE_RATE: u32 = 16_000;

/// The Engine's view of a Dictation so far.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Transcript {
    /// Text the Engine will not revise. Append-only across a Dictation.
    pub committed: String,
    /// The Engine's current guess for what follows. May change or vanish.
    pub tentative: String,
}

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("model not found at {0}")]
    ModelMissing(std::path::PathBuf),
    #[error("{0}")]
    Backend(String),
}

/// One Dictation's worth of recognition state.
pub trait EngineStream: Send {
    /// Feeds audio. Cheap to call often; recognition may happen here or in
    /// [`transcript`](Self::transcript).
    fn feed(&mut self, samples: &[f32]);

    /// Runs any pending recognition and returns the text so far.
    fn transcript(&mut self) -> Transcript;

    /// Flush: no more audio is coming. Returns the final text, all Committed.
    fn finish(&mut self) -> String;
}

/// A loaded Model, ready to start streams. Shared across Dictations.
pub trait Engine: Send + Sync {
    fn name(&self) -> &str;
    fn open_stream(&self) -> Result<Box<dyn EngineStream>, EngineError>;
}

impl fmt::Debug for dyn Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Engine({})", self.name())
    }
}

/// An Engine driven by a script instead of audio, for tests.
///
/// Each call to [`EngineStream::transcript`] advances one step through the
/// scripted transcripts. [`EngineStream::finish`] returns the final script
/// entry's committed text joined with its tentative text.
#[derive(Clone, Debug, Default)]
pub struct ScriptedEngine {
    pub steps: Vec<Transcript>,
}

impl ScriptedEngine {
    /// Builds a script where the committed text grows word by word and the
    /// tentative text is always the next word.
    pub fn word_by_word(text: &str) -> Self {
        let words: Vec<&str> = text.split_whitespace().collect();
        let steps = (0..=words.len())
            .map(|n| Transcript {
                committed: words[..n].join(" "),
                tentative: words.get(n).map(|w| (*w).to_string()).unwrap_or_default(),
            })
            .collect();
        Self { steps }
    }
}

impl Engine for ScriptedEngine {
    fn name(&self) -> &str {
        "scripted"
    }

    fn open_stream(&self) -> Result<Box<dyn EngineStream>, EngineError> {
        Ok(Box::new(ScriptedStream {
            steps: self.steps.clone(),
            position: 0,
            samples_fed: 0,
        }))
    }
}

pub struct ScriptedStream {
    steps: Vec<Transcript>,
    position: usize,
    pub samples_fed: usize,
}

impl EngineStream for ScriptedStream {
    fn feed(&mut self, samples: &[f32]) {
        self.samples_fed += samples.len();
    }

    fn transcript(&mut self) -> Transcript {
        let step = self
            .steps
            .get(self.position)
            .cloned()
            .unwrap_or_else(|| self.steps.last().cloned().unwrap_or_default());
        if self.position + 1 < self.steps.len() {
            self.position += 1;
        }
        step
    }

    fn finish(&mut self) -> String {
        let last = self.steps.last().cloned().unwrap_or_default();
        [last.committed.as_str(), last.tentative.as_str()]
            .iter()
            .filter(|s| !s.is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join(" ")
    }
}
