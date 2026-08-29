//! The transcribe.cpp Engine (ADR 0005): one GGUF file, cache-aware
//! streaming, Committed text that only ever grows.

use std::path::{Path, PathBuf};

use ptw_core::engine::{Engine, EngineError, EngineStream, Transcript};
use tracing::{debug, info};
use transcribe_cpp::{
    CommitPolicy, Model, ParakeetStreamOptions, RunOptions, Session, SessionOptions, Stream,
    StreamExtension, StreamOptions,
};

/// The Model this Engine is tuned for, as `ptw setup` downloads it.
pub const NEMOTRON_FILE: &str = "nemotron-speech-streaming-en-0.6b-Q8_0.gguf";
pub const NEMOTRON_URL: &str = "https://huggingface.co/handy-computer/nemotron-speech-streaming-en-0.6b-gguf/resolve/main/nemotron-speech-streaming-en-0.6b-Q8_0.gguf";
pub const NEMOTRON_SHA256: &str =
    "90d8c89714cd31efc88be62a40c6b2bea57e0cc2063af1ffe2c28f1a228ca110";

/// Threads that measured fastest on the dev box (ADR 0005).
pub const DEFAULT_THREADS: usize = 2;

/// Nemotron's encoder looks `att_context_right` frames ahead; each frame is
/// 80 ms. The Model card's Lookahead settings map to these.
fn att_context_right(lookahead_ms: u32) -> i32 {
    match lookahead_ms {
        0..=80 => 0,
        81..=160 => 1,
        161..=560 => 6,
        _ => 13,
    }
}

pub struct TranscribeEngine {
    model: Model,
    name: String,
    session_options: SessionOptions,
    stream_options: StreamOptions,
}

impl TranscribeEngine {
    /// Loads the GGUF. `threads` of 0 means [`DEFAULT_THREADS`].
    pub fn load(path: &Path, lookahead_ms: u32, threads: usize) -> Result<Self, EngineError> {
        if !path.is_file() {
            return Err(EngineError::ModelMissing(path.to_path_buf()));
        }
        let model = Model::load(path).map_err(|e| EngineError::Backend(e.to_string()))?;
        if !model.capabilities().supports_streaming {
            return Err(EngineError::Backend(format!(
                "{} does not support streaming",
                path.display()
            )));
        }
        let threads = if threads == 0 {
            DEFAULT_THREADS
        } else {
            threads
        };
        let right = att_context_right(lookahead_ms);
        info!(
            arch = model.arch(),
            variant = model.variant(),
            backend = model.backend(),
            threads,
            att_context_right = right,
            "model loaded"
        );
        Ok(Self {
            name: format!(
                "transcribe.cpp/{}",
                path.file_stem()
                    .map(|s| s.to_string_lossy())
                    .unwrap_or_default()
            ),
            model,
            session_options: SessionOptions {
                n_threads: threads as i32,
                ..Default::default()
            },
            stream_options: StreamOptions {
                commit_policy: CommitPolicy::Auto,
                family: Some(StreamExtension::ParakeetStream(ParakeetStreamOptions {
                    att_context_right: Some(right),
                })),
                ..Default::default()
            },
        })
    }
}

/// A `Stream` borrows its `Session` mutably for its whole life, so the
/// Session lives in a leaked Box the stream can borrow for `'static`; Drop
/// takes the stream down first and then reclaims the Box.
pub struct TranscribeStream {
    stream: Option<Stream<'static>>,
    session: *mut Session,
    finished: bool,
}

// SAFETY: Session is Send and nothing but this struct holds the pointer.
unsafe impl Send for TranscribeStream {}

impl TranscribeStream {
    fn open(
        session: Session,
        run: &RunOptions,
        options: &StreamOptions,
    ) -> Result<Self, EngineError> {
        let session = Box::into_raw(Box::new(session));
        // SAFETY: the pointer came from Box::into_raw just above and is only
        // ever dereferenced here and in Drop, after the stream is gone.
        let stream = unsafe { &mut *session }.stream(run, options);
        match stream {
            Ok(stream) => Ok(Self {
                stream: Some(stream),
                session,
                finished: false,
            }),
            Err(e) => {
                // SAFETY: no stream borrows the session; reclaim the Box.
                drop(unsafe { Box::from_raw(session) });
                Err(EngineError::Backend(e.to_string()))
            }
        }
    }

    fn stream(&mut self) -> &mut Stream<'static> {
        self.stream.as_mut().expect("stream lives until drop")
    }
}

impl Drop for TranscribeStream {
    fn drop(&mut self) {
        drop(self.stream.take());
        // SAFETY: the only borrow of the session was the stream, dropped above.
        drop(unsafe { Box::from_raw(self.session) });
    }
}

impl Engine for TranscribeEngine {
    fn name(&self) -> &str {
        &self.name
    }

    fn open_stream(&self) -> Result<Box<dyn EngineStream>, EngineError> {
        let session = self
            .model
            .session_with(&self.session_options)
            .map_err(|e| EngineError::Backend(e.to_string()))?;
        Ok(Box::new(TranscribeStream::open(
            session,
            &RunOptions::default(),
            &self.stream_options,
        )?))
    }
}

impl EngineStream for TranscribeStream {
    fn feed(&mut self, samples: &[f32]) {
        if self.finished || samples.is_empty() {
            return;
        }
        if let Err(e) = self.stream().feed(samples) {
            debug!(error = %e, "feed failed");
        }
    }

    fn transcript(&mut self) -> Transcript {
        let text = self.stream().text();
        Transcript {
            committed: text.committed,
            tentative: text.tentative,
        }
    }

    fn finish(&mut self) -> String {
        if !self.finished {
            self.finished = true;
            if let Err(e) = self.stream().finalize() {
                debug!(error = %e, "finalize failed");
            }
        }
        self.stream().text().committed
    }
}

/// Where `ptw setup` puts the default Model.
pub fn default_model_path(models_dir: &Path) -> PathBuf {
    models_dir.join(NEMOTRON_FILE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookahead_maps_to_the_model_cards_settings() {
        assert_eq!(att_context_right(80), 0);
        assert_eq!(att_context_right(160), 1);
        assert_eq!(att_context_right(560), 6);
        assert_eq!(att_context_right(1120), 13);
    }
}
