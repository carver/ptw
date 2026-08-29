//! Which Engines this binary was built with, and how to load one from config.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::anyhow;
use ptw_core::config::EngineConfig;
use ptw_core::engine::Engine;

pub fn models_dir() -> PathBuf {
    ptw_core::config::data_dir().join("models")
}

/// Engine kinds this build knows, for the settings dialog and `ptw doctor`.
pub fn available() -> Vec<&'static str> {
    vec!["transcribe", "scripted"]
}

/// Absolute path of the configured Model file.
pub fn model_path(config: &EngineConfig) -> PathBuf {
    let path = Path::new(&config.model);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        models_dir().join(path)
    }
}

pub fn build(config: &EngineConfig) -> anyhow::Result<Arc<dyn Engine>> {
    match config.kind.as_str() {
        "transcribe" => {
            let path = model_path(config);
            let engine = ptw_engine_transcribe::TranscribeEngine::load(
                &path,
                config.lookahead_ms,
                config.threads,
            )
            .map_err(|e| anyhow!("{e}; run `ptw setup` to download the model"))?;
            Ok(Arc::new(engine))
        }
        "scripted" => Ok(Arc::new(ptw_core::engine::ScriptedEngine::word_by_word(
            "this is the scripted engine talking",
        ))),
        other => Err(anyhow!(
            "unknown engine `{other}`; this build has: {}",
            available().join(", ")
        )),
    }
}
