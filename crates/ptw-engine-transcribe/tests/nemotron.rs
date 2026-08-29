//! Runs a real recording through the Engine. Needs the Model and a WAV:
//! set `PTW_TEST_MODEL` and `PTW_TEST_WAV` (16 kHz mono), otherwise skipped.

use std::time::Instant;

use ptw_core::engine::{Engine, SAMPLE_RATE};
use ptw_engine_transcribe::TranscribeEngine;

fn env_path(var: &str) -> Option<std::path::PathBuf> {
    std::env::var_os(var)
        .map(Into::into)
        .filter(|p: &std::path::PathBuf| p.is_file())
}

#[test]
fn committed_text_only_grows_and_the_flush_is_quick() {
    let (Some(model), Some(wav)) = (env_path("PTW_TEST_MODEL"), env_path("PTW_TEST_WAV")) else {
        eprintln!("skipped: set PTW_TEST_MODEL and PTW_TEST_WAV");
        return;
    };
    let mut reader = hound::WavReader::open(&wav).unwrap();
    assert_eq!(reader.spec().sample_rate, SAMPLE_RATE);
    assert_eq!(reader.spec().channels, 1);
    let pcm: Vec<f32> = reader
        .samples::<i16>()
        .map(|s| f32::from(s.unwrap()) / 32768.0)
        .collect();

    let engine = TranscribeEngine::load(&model, 560, 0).unwrap();
    let mut stream = engine.open_stream().unwrap();
    let mut previous = String::new();
    let mut slowest = 0.0f32;
    for chunk in pcm.chunks(SAMPLE_RATE as usize * 80 / 1000) {
        let started = Instant::now();
        stream.feed(chunk);
        let transcript = stream.transcript();
        slowest = slowest.max(started.elapsed().as_secs_f32());
        assert!(
            transcript.committed.starts_with(&previous),
            "committed text was revised:\n{previous}\n{}",
            transcript.committed
        );
        previous = transcript.committed;
    }
    let started = Instant::now();
    let final_text = stream.finish();
    let flush = started.elapsed().as_secs_f32();
    eprintln!("slowest chunk {slowest:.3}s, flush {flush:.3}s\n{final_text}");
    assert!(final_text.starts_with(&previous));
    assert!(final_text.split_whitespace().count() > 5, "{final_text:?}");
    assert!(flush < 2.0, "flush took {flush}s");

    let mut second = engine.open_stream().unwrap();
    second.feed(&pcm[..SAMPLE_RATE as usize]);
    assert!(
        second.transcript().committed.is_empty()
            || second.transcript().committed.len() < final_text.len()
    );
}
