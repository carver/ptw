//! Streaming + offline benchmark harness for the sherpa-onnx crate.
//!
//! sherpa-bench stream <model_dir> <threads> <label> <out_dir> [--beam N] [--hotwords a/b/c] [--score S] <wav>...
//! sherpa-bench offline <model_dir> <threads> <label> <out_dir> <wav>...
//!
//! --beam N switches to modified_beam_search with N active paths (0 = greedy).
//! --hotwords biases each stream toward the '/'-separated phrases (needs --beam).
//! --score is the per-token hotword boost (default 2.0).
//!
//! Writes one JSON per wav into out_dir: <label>_t<threads>_<wavstem>.json

use serde_json::{json, Value};
use sherpa_onnx::*;
use std::time::Instant;

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
            let kept = common_word_prefix(&self.committed, new);
            let old_words = self.committed.split_whitespace().count();
            self.revisions.push(json!({"chunk": chunk, "t": t, "old": self.committed, "new": new,
                "kept_words": kept, "reverted_words": old_words - kept}));
            self.log.push(json!({"chunk": chunk, "t": t, "added": new, "revised": true}));
        }
        self.committed = new.to_string();
    }
}

fn common_word_prefix(a: &str, b: &str) -> usize {
    a.split_whitespace().zip(b.split_whitespace()).take_while(|(x, y)| x == y).count()
}

/// sherpa's hotword tokenizer wants a sentencepiece vocab; the release ships
/// only tokens.txt. Equal scores make the encoder greedy-longest-match, which
/// reproduces the model's tokenization (same trick as upstream's #3723).
fn bpe_vocab_path(model_dir: &str) -> String {
    let path = format!("{model_dir}/bpe.vocab");
    if !std::path::Path::new(&path).exists() {
        let tokens = std::fs::read_to_string(format!("{model_dir}/tokens.txt")).expect("tokens.txt");
        let vocab: String = tokens
            .lines()
            .filter_map(|l| l.split_whitespace().next())
            .map(|tok| format!("{tok}\t-1.0\n"))
            .collect();
        std::fs::write(&path, vocab).expect("write bpe.vocab");
    }
    path
}

#[derive(Default)]
struct Decode {
    beam: i32,
    hotwords: Option<String>,
    score: f32,
}

fn stats(ms: &[f64]) -> (f64, f64, f64, f64) {
    let mut v = ms.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    let mean = v.iter().sum::<f64>() / n as f64;
    let p95 = v[((n as f64) * 0.95) as usize - if n > 20 { 0 } else { 1 }.min(n - 1)];
    let max = v[n - 1];
    let total = v.iter().sum::<f64>();
    (mean, p95, max, total)
}

fn stem(path: &str) -> String {
    std::path::Path::new(path).file_stem().unwrap().to_string_lossy().to_string()
}

fn run_stream(model_dir: &str, threads: i32, label: &str, out_dir: &str, decode: &Decode, wavs: &[String]) {
    let t0 = Instant::now();
    let beam = decode.beam > 0;
    let cfg = OnlineRecognizerConfig {
        model_config: OnlineModelConfig {
            transducer: OnlineTransducerModelConfig {
                encoder: Some(format!("{model_dir}/encoder.int8.onnx")),
                decoder: Some(format!("{model_dir}/decoder.int8.onnx")),
                joiner: Some(format!("{model_dir}/joiner.int8.onnx")),
            },
            tokens: Some(format!("{model_dir}/tokens.txt")),
            num_threads: threads,
            provider: Some("cpu".into()),
            modeling_unit: beam.then(|| "bpe".into()),
            bpe_vocab: beam.then(|| bpe_vocab_path(model_dir)),
            ..Default::default()
        },
        decoding_method: Some(if beam { "modified_beam_search" } else { "greedy_search" }.into()),
        max_active_paths: if beam { decode.beam } else { 4 },
        hotwords_score: decode.score,
        ..Default::default()
    };
    let rec = OnlineRecognizer::create(&cfg).expect("create online recognizer");
    let load_ms = t0.elapsed().as_secs_f64() * 1000.0;
    eprintln!("loaded {label} in {load_ms:.0} ms");

    for wav in wavs {
        let pcm = read_wav(wav);
        let audio_s = pcm.len() as f64 / 16000.0;
        let stream = match &decode.hotwords {
            Some(hw) => rec.create_stream_with_hotwords(hw),
            None => rec.create_stream(),
        };
        let mut tracker = CommitTracker::new();
        let mut chunk_ms = vec![];
        let mut n_decodes = 0usize;
        for (i, chunk) in pcm.chunks(CHUNK).enumerate() {
            let t = Instant::now();
            stream.accept_waveform(16000, chunk);
            while rec.is_ready(&stream) {
                rec.decode(&stream);
                n_decodes += 1;
            }
            let text = rec.get_result(&stream).map(|r| r.text).unwrap_or_default();
            chunk_ms.push(t.elapsed().as_secs_f64() * 1000.0);
            tracker.observe(i, (i + 1) as f64 * CHUNK as f64 / 16000.0, &text);
        }
        let n_chunks = chunk_ms.len();
        let t = Instant::now();
        // sherpa's online NeMo path only decodes whole chunks; pad 0.5 s of
        // silence so the last partial chunk gets flushed (the docs do the same).
        stream.accept_waveform(16000, &vec![0.0f32; 8000]);
        stream.input_finished();
        while rec.is_ready(&stream) {
            rec.decode(&stream);
            n_decodes += 1;
        }
        let res = rec.get_result(&stream).expect("result");
        let finalize_ms = t.elapsed().as_secs_f64() * 1000.0;
        tracker.observe(n_chunks, audio_s, &res.text);
        let (mean, p95, max, total) = stats(&chunk_ms);
        let out = json!({
            "engine": "sherpa-onnx", "model": label, "threads": threads, "file": stem(wav),
            "beam": decode.beam, "hotwords": decode.hotwords, "hotwords_score": decode.score,
            "audio_s": audio_s, "load_ms": load_ms, "n_chunks": n_chunks, "n_decodes": n_decodes,
            "chunk_mean_ms": mean, "chunk_p95_ms": p95, "chunk_max_ms": max,
            "total_compute_s": total / 1000.0, "rtf": total / 1000.0 / audio_s,
            "finalize_ms": finalize_ms,
            "commit_log": tracker.log, "revisions": tracker.revisions,
            "final_text": res.text.trim(), "peak_rss_mb": peak_rss_mb(),
            "token_timestamps": res.timestamps, "tokens": res.tokens,
        });
        let path = format!("{out_dir}/{label}_t{threads}_{}.json", stem(wav));
        std::fs::write(&path, serde_json::to_string_pretty(&out).unwrap()).unwrap();
        eprintln!("{}: rtf {:.3} p95 {:.1} ms max {:.1} ms fin {:.0} ms | {}", stem(wav), total / 1000.0 / audio_s, p95, max, finalize_ms, res.text.trim());
    }
}

fn run_offline(model_dir: &str, threads: i32, label: &str, out_dir: &str, wavs: &[String]) {
    let t0 = Instant::now();
    let cfg = OfflineRecognizerConfig {
        model_config: OfflineModelConfig {
            transducer: OfflineTransducerModelConfig {
                encoder: Some(format!("{model_dir}/encoder.int8.onnx")),
                decoder: Some(format!("{model_dir}/decoder.int8.onnx")),
                joiner: Some(format!("{model_dir}/joiner.int8.onnx")),
            },
            tokens: Some(format!("{model_dir}/tokens.txt")),
            num_threads: threads,
            provider: Some("cpu".into()),
            model_type: Some("nemo_transducer".into()),
            ..Default::default()
        },
        decoding_method: Some("greedy_search".into()),
        ..Default::default()
    };
    let rec = OfflineRecognizer::create(&cfg).expect("create offline recognizer");
    let load_ms = t0.elapsed().as_secs_f64() * 1000.0;
    for wav in wavs {
        let pcm = read_wav(wav);
        let audio_s = pcm.len() as f64 / 16000.0;
        let t = Instant::now();
        let stream = rec.create_stream();
        stream.accept_waveform(16000, &pcm);
        rec.decode(&stream);
        let res = stream.get_result().expect("result");
        let total = t.elapsed().as_secs_f64();
        // Merge BPE tokens into words with the timestamp of the first token.
        let mut words: Vec<Value> = vec![];
        if let Some(ts) = &res.timestamps {
            for (tok, t) in res.tokens.iter().zip(ts) {
                let starts_word = tok.starts_with('▁') || tok.starts_with(' ') || words.is_empty();
                let clean = tok.trim_start_matches('▁').trim_start_matches(' ').to_string();
                if starts_word {
                    words.push(json!({"w": clean, "t": t}));
                } else if let Some(last) = words.last_mut() {
                    let w = format!("{}{}", last["w"].as_str().unwrap(), clean);
                    last["w"] = json!(w);
                }
            }
        }
        let out = json!({
            "engine": "sherpa-onnx-offline", "model": label, "threads": threads, "file": stem(wav),
            "audio_s": audio_s, "load_ms": load_ms, "total_compute_s": total, "rtf": total / audio_s,
            "final_text": res.text.trim(), "peak_rss_mb": peak_rss_mb(), "words": words,
        });
        let path = format!("{out_dir}/{label}_t{threads}_{}.json", stem(wav));
        std::fs::write(&path, serde_json::to_string_pretty(&out).unwrap()).unwrap();
        eprintln!("{}: rtf {:.3} | {}", stem(wav), total / audio_s, res.text.trim());
    }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 7 {
        eprintln!("usage: sherpa-bench stream|offline <model_dir> <threads> <label> <out_dir> <wav>...");
        std::process::exit(2);
    }
    let threads: i32 = a[3].parse().unwrap();
    let mut decode = Decode { score: 2.0, ..Default::default() };
    let mut rest = &a[6..];
    while let [flag, value, tail @ ..] = rest {
        match flag.as_str() {
            "--beam" => decode.beam = value.parse().expect("--beam N"),
            "--hotwords" => decode.hotwords = Some(value.clone()),
            "--score" => decode.score = value.parse().expect("--score S"),
            _ => break,
        }
        rest = tail;
    }
    if decode.hotwords.is_some() && decode.beam == 0 {
        eprintln!("--hotwords needs --beam");
        std::process::exit(2);
    }
    match a[1].as_str() {
        "stream" => run_stream(&a[2], threads, &a[4], &a[5], &decode, rest),
        "offline" => run_offline(&a[2], threads, &a[4], &a[5], rest),
        _ => panic!("mode"),
    }
}
