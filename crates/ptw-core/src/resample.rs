//! Converts captured audio to what the Engine wants: mono at 16 kHz.
//!
//! A windowed-sinc resampler is enough here; the input is speech from a
//! microphone and the ratio is fixed for the life of a stream. Written in
//! place of a dependency because the maths is short and the crate APIs in
//! this area churn.

use crate::engine::SAMPLE_RATE;

/// Half the number of source taps on each side of the output sample.
const HALF_TAPS: usize = 16;

/// Averages interleaved channels into one.
pub fn to_mono(interleaved: &[f32], channels: usize) -> Vec<f32> {
    match channels {
        0 | 1 => interleaved.to_vec(),
        n => interleaved
            .chunks_exact(n)
            .map(|frame| frame.iter().sum::<f32>() / n as f32)
            .collect(),
    }
}

/// Streaming resampler from `input_rate` to [`SAMPLE_RATE`], mono.
#[derive(Debug)]
pub struct Resampler {
    ratio: f64,
    /// Source samples not yet consumed, with `HALF_TAPS` of history in front.
    pending: Vec<f32>,
    /// Position of the next output sample in source samples, relative to `pending[0]`.
    position: f64,
    kernel_cutoff: f64,
}

impl Resampler {
    pub fn new(input_rate: u32) -> Self {
        let ratio = f64::from(input_rate) / f64::from(SAMPLE_RATE);
        Self {
            ratio,
            pending: vec![0.0; HALF_TAPS],
            position: HALF_TAPS as f64,
            kernel_cutoff: 1.0 / ratio.max(1.0),
        }
    }

    pub fn is_identity(&self) -> bool {
        self.ratio == 1.0
    }

    /// Feeds mono source samples and returns the output samples now available.
    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        if self.is_identity() {
            return input.to_vec();
        }
        self.pending.extend_from_slice(input);
        let mut out = Vec::with_capacity((input.len() as f64 / self.ratio) as usize + 1);
        while self.position + HALF_TAPS as f64 <= self.pending.len() as f64 {
            out.push(self.sample_at(self.position));
            self.position += self.ratio;
        }
        let keep_from = (self.position.floor() as usize).saturating_sub(HALF_TAPS);
        self.pending.drain(..keep_from);
        self.position -= keep_from as f64;
        out
    }

    fn sample_at(&self, position: f64) -> f32 {
        let center = position.floor() as usize;
        let mut acc = 0.0f64;
        let mut weight = 0.0f64;
        for i in
            (center + 1).saturating_sub(HALF_TAPS)..(center + HALF_TAPS).min(self.pending.len())
        {
            let x = i as f64 - position;
            let w = sinc(x * self.kernel_cutoff) * hann(x / HALF_TAPS as f64);
            acc += f64::from(self.pending[i]) * w;
            weight += w;
        }
        if weight.abs() < f64::EPSILON {
            0.0
        } else {
            (acc / weight) as f32
        }
    }
}

fn sinc(x: f64) -> f64 {
    if x.abs() < 1e-9 {
        1.0
    } else {
        (std::f64::consts::PI * x).sin() / (std::f64::consts::PI * x)
    }
}

fn hann(x: f64) -> f64 {
    if x.abs() >= 1.0 {
        0.0
    } else {
        0.5 * (1.0 + (std::f64::consts::PI * x).cos())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(rate: u32, hz: f64, seconds: f64) -> Vec<f32> {
        (0..(f64::from(rate) * seconds) as usize)
            .map(|i| (2.0 * std::f64::consts::PI * hz * i as f64 / f64::from(rate)).sin() as f32)
            .collect()
    }

    fn dominant_hz(samples: &[f32], rate: u32) -> f64 {
        let crossings = samples
            .windows(2)
            .filter(|w| w[0] < 0.0 && w[1] >= 0.0)
            .count();
        crossings as f64 / (samples.len() as f64 / f64::from(rate))
    }

    #[test]
    fn mono_mixdown_averages_channels() {
        assert_eq!(to_mono(&[1.0, 0.0, 0.5, 0.5], 2), vec![0.5, 0.5]);
        assert_eq!(to_mono(&[1.0, 2.0], 1), vec![1.0, 2.0]);
    }

    #[test]
    fn keeps_the_pitch_and_length_from_48k_and_44k() {
        for rate in [48_000, 44_100, 32_000] {
            let mut r = Resampler::new(rate);
            let input = sine(rate, 440.0, 1.0);
            let mut out = Vec::new();
            for chunk in input.chunks(480) {
                out.extend(r.process(chunk));
            }
            let expected_len = SAMPLE_RATE as usize;
            assert!(
                (out.len() as i64 - expected_len as i64).abs() < 64,
                "rate {rate}: got {} samples",
                out.len()
            );
            let hz = dominant_hz(&out[100..], SAMPLE_RATE);
            assert!((hz - 440.0).abs() < 5.0, "rate {rate}: {hz} Hz");
            let peak = out[100..].iter().fold(0.0f32, |m, s| m.max(s.abs()));
            assert!((peak - 1.0).abs() < 0.05, "rate {rate}: peak {peak}");
        }
    }

    #[test]
    fn identity_rate_passes_through() {
        let mut r = Resampler::new(SAMPLE_RATE);
        assert!(r.is_identity());
        assert_eq!(r.process(&[0.1, 0.2]), vec![0.1, 0.2]);
    }

    #[test]
    fn chunking_does_not_change_the_output() {
        let input = sine(48_000, 300.0, 0.5);
        let mut whole = Resampler::new(48_000);
        let expected = whole.process(&input);
        let mut pieces = Resampler::new(48_000);
        let mut got = Vec::new();
        for chunk in input.chunks(37) {
            got.extend(pieces.process(chunk));
        }
        assert_eq!(got.len(), expected.len());
        for (a, b) in got.iter().zip(&expected) {
            assert!((a - b).abs() < 1e-5);
        }
    }
}
