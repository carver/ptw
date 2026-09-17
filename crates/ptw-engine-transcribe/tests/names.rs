//! Runs a recording through the Engine and the Correction layer and checks
//! that named Custom words come out spelled right. The recording and the
//! names are private, so they live in a gitignored TOML file:
//!
//! ```toml
//! wav = "target/bench/sample5.wav"      # 16 kHz mono, relative to the repo root
//! custom_words = ["Caitlyn", "Bernal Heights"]
//! expect = ["Caitlyn"]                  # must appear in the typed text
//! not_yet = ["Bernal Heights"]          # reported, not asserted
//! ```
//!
//! Set `PTW_TEST_MODEL` and `PTW_TEST_NAMES` to run it, otherwise skipped.

use std::path::{Path, PathBuf};

use ptw_core::correction::CustomWords;
use ptw_core::dictation::Dictation;
use ptw_core::engine::{Engine, SAMPLE_RATE};
use ptw_engine_transcribe::TranscribeEngine;
use serde::Deserialize;

#[derive(Deserialize)]
struct Spec {
    wav: PathBuf,
    custom_words: Vec<String>,
    expect: Vec<String>,
    #[serde(default)]
    not_yet: Vec<String>,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn from_env(var: &str) -> Option<PathBuf> {
    let path = repo_root().join(std::env::var_os(var)?);
    path.is_file().then_some(path)
}

fn read_wav(path: &Path) -> Vec<f32> {
    let mut reader = hound::WavReader::open(path).unwrap();
    assert_eq!(reader.spec().sample_rate, SAMPLE_RATE, "{}", path.display());
    assert_eq!(reader.spec().channels, 1, "{}", path.display());
    reader
        .samples::<i16>()
        .map(|s| f32::from(s.unwrap()) / 32768.0)
        .collect()
}

/// What the Typist would have received: every Committed word after the
/// Hold-back and Correction, exactly as the daemon's session does it.
fn dictate(engine: &dyn Engine, words: &CustomWords, pcm: &[f32]) -> String {
    let mut stream = engine.open_stream().unwrap();
    let mut dictation = Dictation::new(words.clone());
    let mut typed = String::new();
    for chunk in pcm.chunks(SAMPLE_RATE as usize * 80 / 1000) {
        stream.feed(chunk);
        typed.push_str(&dictation.update(&stream.transcript().committed));
    }
    typed.push_str(&dictation.finish(&stream.finish()));
    typed
}

#[test]
fn custom_names_are_typed_as_listed() {
    let (Some(model), Some(spec_path)) = (from_env("PTW_TEST_MODEL"), from_env("PTW_TEST_NAMES"))
    else {
        eprintln!("skipped: set PTW_TEST_MODEL and PTW_TEST_NAMES");
        return;
    };
    let spec: Spec = toml::from_str(&std::fs::read_to_string(&spec_path).unwrap()).unwrap();
    let pcm = read_wav(&repo_root().join(&spec.wav));
    let engine = TranscribeEngine::load(&model, 560, 0).unwrap();

    let typed = dictate(&engine, &CustomWords::new(&spec.custom_words), &pcm);
    eprintln!("{typed}");

    let missing: Vec<&str> = spec
        .expect
        .iter()
        .map(String::as_str)
        .filter(|name| !typed.contains(name))
        .collect();
    let now_right: Vec<&str> = spec
        .not_yet
        .iter()
        .map(String::as_str)
        .filter(|name| typed.contains(name))
        .collect();
    if !now_right.is_empty() {
        eprintln!("now right, move to expect: {now_right:?}");
    }
    assert!(missing.is_empty(), "not typed as listed: {missing:?}");
}
