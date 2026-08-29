//! Which Engines this binary was built with, and how to load one from config.

use std::sync::Arc;

use anyhow::anyhow;
use ptw_core::config::EngineConfig;
use ptw_core::engine::Engine;

/// Engine kinds this build knows, for error messages and `ptw doctor`.
pub fn available() -> Vec<&'static str> {
    vec!["scripted"]
}

pub fn build(config: &EngineConfig) -> anyhow::Result<Arc<dyn Engine>> {
    match config.kind.as_str() {
        "scripted" => Ok(Arc::new(ptw_core::engine::ScriptedEngine::word_by_word(
            "this is the scripted engine talking",
        ))),
        other => Err(anyhow!(
            "unknown engine `{other}`; this build has: {}",
            available().join(", ")
        )),
    }
}
