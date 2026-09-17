//! Streaming benchmark harness for the transcribe-cpp crate.
//!
//! tcpp-bench <model.gguf> <threads> <att_context_right> <label> <out_dir> <wav>...

use serde_json::{json, Value};
use std::time::Instant;
use transcribe_cpp::{
    CommitPolicy, Model, ParakeetStreamOptions, RunOptions, SessionOptions, StreamExtension,
    StreamOptions,
};

const CHUNK: usize = 1280; // 80 ms at 16 kHz

fn peak_rss_mb() -> f64 {
    let s = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    for line in s.lines() {
        if let Some(rest) = line.strip_prefix("VmHWM:") {
            let kb: f64 = rest.trim().trim_end_matches("kB").trim().parse().unwrap_or(0.0);
            return kb / 1024.0;
        }
    }
    0.0
}

fn read_wav(path: &str) -> Vec<f32> {
    let mut r = hound::WavReader::open(path).expect("open wav");
    assert_eq!(r.spec().sample_rate, 16000);
    assert_eq!(r.spec().channels, 1);
    r.samples::<i16>().map(|s| s.unwrap() as f32 / 32768.0).collect()
}

struct CommitTracker {
    committed: String,
    log: Vec<Value>,
    revisions: Vec<Value>,
}

impl CommitTracker {
    fn new() -> Self {
        Self { committed: String::new(), log: vec![], revisions: vec![] }
    }
    fn observe(&mut self, chunk: usize, t: f64, new: &str) {
        if new == self.committed {
            return;
        }
        if new.starts_with(&self.committed) {
            let added = new[self.committed.len()..].to_string();
            self.log.push(json!({"chunk": chunk, "t": t, "added": added}));
        } else {
            self.revisions.push(json!({"chunk": chunk, "t": t, "old": self.committed, "new": new}));
            self.log.push(json!({"chunk": chunk, "t": t, "added": new, "revised": true}));
        }
        self.committed = new.to_string();
    }
}

fn stats(ms: &[f64]) -> (f64, f64, f64, f64) {
    let mut v = ms.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    let mean = v.iter().sum::<f64>() / n as f64;
    let p95 = v[(((n as f64) * 0.95) as usize).min(n - 1)];
    let max = v[n - 1];
    let total = v.iter().sum::<f64>();
    (mean, p95, max, total)
}

fn stem(path: &str) -> String {
    std::path::Path::new(path).file_stem().unwrap().to_string_lossy().to_string()
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 7 {
        eprintln!("usage: tcpp-bench <model.gguf> <threads> <att_right> <label> <out_dir> <wav>...");
        std::process::exit(2);
    }
    let threads: i32 = a[2].parse().unwrap();
    let att_right: i32 = a[3].parse().unwrap();
    let label = &a[4];
    let out_dir = &a[5];
    let wavs = &a[6..];

    let t0 = Instant::now();
    let model = Model::load(&a[1]).expect("load model");
    assert!(model.capabilities().supports_streaming, "model does not support streaming");
    let mut session = model
        .session_with(&SessionOptions { n_threads: threads, ..Default::default() })
        .expect("session");
    let load_ms = t0.elapsed().as_secs_f64() * 1000.0;
    eprintln!("loaded {} ({} / {} / backend {}) in {load_ms:.0} ms", label, model.arch(), model.variant(), model.backend());

    let sopts = StreamOptions {
        commit_policy: CommitPolicy::Auto,
        family: Some(StreamExtension::ParakeetStream(ParakeetStreamOptions {
            att_context_right: Some(att_right),
        })),
        ..Default::default()
    };

    for wav in wavs {
        let pcm = read_wav(wav);
        let audio_s = pcm.len() as f64 / 16000.0;
        let mut stream = session.stream(&RunOptions::default(), &sopts).expect("stream");
        let mut tracker = CommitTracker::new();
        let mut tentative_log: Vec<Value> = vec![];
        let mut chunk_ms = vec![];
        let mut n_tentative_changes = 0usize;
        for (i, chunk) in pcm.chunks(CHUNK).enumerate() {
            let t = Instant::now();
            let upd = stream.feed(chunk).expect("feed");
            let text = stream.text();
            chunk_ms.push(t.elapsed().as_secs_f64() * 1000.0);
            let at = (i + 1) as f64 * CHUNK as f64 / 16000.0;
            tracker.observe(i, at, &text.committed);
            if upd.tentative_changed {
                n_tentative_changes += 1;
                if tentative_log.len() < 400 {
                    tentative_log.push(json!({"chunk": i, "t": at, "tentative": text.tentative}));
                }
            }
        }
        let n_chunks = chunk_ms.len();
        let t = Instant::now();
        let upd = stream.finalize().expect("finalize");
        let text = stream.text();
        let finalize_ms = t.elapsed().as_secs_f64() * 1000.0;
        assert!(upd.is_final);
        tracker.observe(n_chunks, audio_s, &text.committed);
        let (mean, p95, max, total) = stats(&chunk_ms);
        let out = json!({
            "engine": "transcribe-cpp", "model": label, "threads": threads, "att_right": att_right,
            "file": stem(wav), "audio_s": audio_s, "load_ms": load_ms, "n_chunks": n_chunks,
            "chunk_mean_ms": mean, "chunk_p95_ms": p95, "chunk_max_ms": max,
            "total_compute_s": total / 1000.0, "rtf": total / 1000.0 / audio_s,
            "finalize_ms": finalize_ms, "n_tentative_changes": n_tentative_changes,
            "commit_log": tracker.log, "revisions": tracker.revisions, "tentative_log": tentative_log,
            "final_text": text.full.trim(), "final_committed": text.committed.trim(),
            "peak_rss_mb": peak_rss_mb(),
        });
        let path = format!("{out_dir}/{label}_t{threads}_{}.json", stem(wav));
        std::fs::write(&path, serde_json::to_string_pretty(&out).unwrap()).unwrap();
        eprintln!("{}: rtf {:.3} p95 {:.1} ms max {:.1} ms fin {:.0} ms | {}", stem(wav), total / 1000.0 / audio_s, p95, max, finalize_ms, text.full.trim());
    }
}
