//! Microphone capture and audio cues, on one thread that owns the streams
//! (cpal streams cannot move between threads).

use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;

use anyhow::{Context, anyhow};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream};
use ptw_core::resample::{Resampler, to_mono};
use ptw_core::session::Input;
use tracing::{info, warn};

#[derive(Clone, Copy, Debug)]
pub enum Cue {
    Start,
    Stop,
    Abort,
}

enum Command {
    StartCapture { device: String, sink: Sender<Input> },
    StopCapture,
    Play(Cue),
}

/// Handle to the audio thread.
#[derive(Clone)]
pub struct Audio {
    tx: Sender<Command>,
}

impl Audio {
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel();
        thread::Builder::new()
            .name("ptw-audio".into())
            .spawn(move || {
                let mut capture: Option<Stream> = None;
                for command in rx {
                    match command {
                        Command::StartCapture { device, sink } => match open_capture(&device, sink)
                        {
                            Ok(stream) => drop(capture.replace(stream)),
                            Err(e) => warn!(error = %e, "cannot open microphone"),
                        },
                        Command::StopCapture => drop(capture.take()),
                        Command::Play(cue) => {
                            thread::Builder::new()
                                .name("ptw-cue".into())
                                .spawn(move || {
                                    if let Err(e) = play(cue) {
                                        warn!(error = %e, "cannot play cue");
                                    }
                                })
                                .ok();
                        }
                    }
                }
            })
            .expect("spawn audio thread");
        Self { tx }
    }

    /// Opens the microphone and sends 16 kHz mono chunks to `sink` until
    /// [`stop_capture`](Self::stop_capture).
    pub fn start_capture(&self, device: &str, sink: Sender<Input>) {
        self.tx
            .send(Command::StartCapture {
                device: device.to_string(),
                sink,
            })
            .ok();
    }

    pub fn stop_capture(&self) {
        self.tx.send(Command::StopCapture).ok();
    }

    pub fn play(&self, cue: Cue) {
        self.tx.send(Command::Play(cue)).ok();
    }
}

fn device_name(device: &cpal::Device) -> String {
    device
        .description()
        .map(|d| d.name().to_string())
        .unwrap_or_else(|_| "?".to_string())
}

/// Input device names as cpal reports them.
pub fn list_inputs() -> Vec<String> {
    let host = cpal::default_host();
    host.input_devices()
        .map(|devices| devices.map(|d| device_name(&d)).collect())
        .unwrap_or_default()
}

fn input_device(name: &str) -> anyhow::Result<cpal::Device> {
    let host = cpal::default_host();
    if name.is_empty() {
        return host
            .default_input_device()
            .ok_or_else(|| anyhow!("no default input device"));
    }
    host.input_devices()?
        .find(|d| device_name(d) == name)
        .ok_or_else(|| anyhow!("input device `{name}` not found"))
}

fn open_capture(name: &str, sink: Sender<Input>) -> anyhow::Result<Stream> {
    let device = input_device(name)?;
    let supported = device
        .default_input_config()
        .context("no default input config")?;
    let channels = usize::from(supported.channels());
    let rate = supported.sample_rate();
    let sample_format = supported.sample_format();
    info!(
        device = device_name(&device),
        rate,
        channels,
        ?sample_format,
        "capturing"
    );
    let config = supported.config();
    let mut resampler = Resampler::new(rate);
    let err = |e| warn!(error = %e, "capture stream error");
    let stream = match sample_format {
        SampleFormat::F32 => device.build_input_stream(
            config,
            move |data: &[f32], _| {
                let mono = to_mono(data, channels);
                sink.send(Input::Audio(resampler.process(&mono))).ok();
            },
            err,
            None,
        )?,
        SampleFormat::I16 => device.build_input_stream(
            config,
            move |data: &[i16], _| {
                let floats: Vec<f32> = data.iter().map(|s| f32::from(*s) / 32768.0).collect();
                let mono = to_mono(&floats, channels);
                sink.send(Input::Audio(resampler.process(&mono))).ok();
            },
            err,
            None,
        )?,
        other => return Err(anyhow!("unsupported sample format {other:?}")),
    };
    stream.play()?;
    Ok(stream)
}

/// A short tone: rising for Start, falling for Stop, low buzz for Abort.
fn play(cue: Cue) -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow!("no default output device"))?;
    let supported = device.default_output_config()?;
    let rate = supported.sample_rate();
    let channels = usize::from(supported.channels());
    let (from_hz, to_hz, millis) = match cue {
        Cue::Start => (660.0, 880.0, 90),
        Cue::Stop => (880.0, 660.0, 90),
        Cue::Abort => (220.0, 220.0, 120),
    };
    let total = rate as usize * millis / 1000;
    let mut position = 0usize;
    let stream = device.build_output_stream(
        supported.config(),
        move |data: &mut [f32], _| {
            for frame in data.chunks_mut(channels) {
                let sample = if position < total {
                    let t = position as f32 / total as f32;
                    let hz = from_hz + (to_hz - from_hz) * t;
                    let envelope = (std::f32::consts::PI * t).sin();
                    0.15 * envelope
                        * (2.0 * std::f32::consts::PI * hz * position as f32 / rate as f32).sin()
                } else {
                    0.0
                };
                position += 1;
                frame.fill(sample);
            }
        },
        |e| warn!(error = %e, "cue stream error"),
        None,
    )?;
    stream.play()?;
    thread::sleep(Duration::from_millis(millis as u64 + 40));
    Ok(())
}
