//! `ptw transcribe`: runs a WAV through the Engine like a Hold would, printing
//! text as it commits. The test path that needs no microphone.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use clap::Args as ClapArgs;
use ptw_core::config::Config;
use ptw_core::correction::CustomWords;
use ptw_core::engine::SAMPLE_RATE;
use ptw_core::resample::{Resampler, to_mono};
use ptw_core::session::{self, Input};
use ptw_core::typist::{Typist, TypistError};

use crate::engines;

#[derive(ClapArgs)]
pub struct Args {
    pub wav: PathBuf,
    /// Feed audio at the speed it was spoken instead of as fast as possible.
    #[arg(long)]
    pub realtime: bool,
}

struct PrintingTypist {
    started: Instant,
}

impl Typist for PrintingTypist {
    fn type_text(&mut self, text: &str) -> Result<(), TypistError> {
        eprint!("[{:>6.2}s] ", self.started.elapsed().as_secs_f32());
        println!("{text}");
        Ok(())
    }
}

fn read_wav(path: &Path) -> anyhow::Result<Vec<f32>> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().collect::<Result<_, _>>()?,
        hound::SampleFormat::Int => {
            let scale = (1u32 << (spec.bits_per_sample - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|s| s.map(|v| v as f32 / scale))
                .collect::<Result<_, _>>()?
        }
    };
    let mono = to_mono(&samples, usize::from(spec.channels));
    Ok(Resampler::new(spec.sample_rate).process(&mono))
}

pub fn run(config_path: &Path, args: Args) -> anyhow::Result<()> {
    let config = Config::load(config_path)?;
    let engine = engines::build(&config.engine)?;
    let words = CustomWords::new(&config.custom_words);
    let audio = read_wav(&args.wav)?;
    let seconds = audio.len() as f32 / SAMPLE_RATE as f32;
    eprintln!(
        "{}: {seconds:.1}s of audio, engine {}",
        args.wav.display(),
        engine.name()
    );

    let (tx, rx) = mpsc::channel();
    let chunk = SAMPLE_RATE as usize * 80 / 1000;
    let realtime = args.realtime;
    let feeder = thread::spawn(move || {
        for piece in audio.chunks(chunk) {
            tx.send(Input::Audio(piece.to_vec())).ok();
            if realtime {
                thread::sleep(Duration::from_millis(80));
            }
        }
        tx.send(Input::Stop).ok();
    });
    let started = Instant::now();
    let mut typist = PrintingTypist { started };
    let outcome = session::run(&*engine, &words, &rx, &mut typist)?;
    feeder.join().ok();
    let elapsed = started.elapsed().as_secs_f32();
    eprintln!("done in {elapsed:.2}s ({:.1}x realtime)", seconds / elapsed);
    eprintln!("typed: {}", outcome.typed);
    Ok(())
}
