//! The Typist: delivers text to the focused application.

#[derive(Debug, thiserror::Error)]
pub enum TypistError {
    #[error("{0}")]
    Backend(String),
}

pub trait Typist: Send {
    /// Types `text` after whatever was typed before. Never deletes.
    fn type_text(&mut self, text: &str) -> Result<(), TypistError>;

    /// Re-establishes the route to the desktop after a typing failure
    /// (suspend kills the portal session, for one). Backends with
    /// nothing to re-establish keep this default.
    fn reconnect(&mut self) -> Result<(), TypistError> {
        Err(TypistError::Backend("this Typist cannot reconnect".into()))
    }
}

/// Collects typed text in memory, for tests and `ptw doctor`.
#[derive(Debug, Default)]
pub struct RecordingTypist {
    pub chunks: Vec<String>,
}

impl RecordingTypist {
    pub fn text(&self) -> String {
        self.chunks.concat()
    }
}

impl Typist for RecordingTypist {
    fn type_text(&mut self, text: &str) -> Result<(), TypistError> {
        self.chunks.push(text.to_string());
        Ok(())
    }
}
