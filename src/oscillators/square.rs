//! Square wave oscillator implementation.

use super::Oscillator;
use crate::{AudioSignal, Signal};

pub struct SquareOscillator<const SAMPLE_RATE: u32> {
    phase: f64,
    phase_increment: f64,
}

impl<const SAMPLE_RATE: u32> SquareOscillator<SAMPLE_RATE> {
    pub fn new(frequency: f64) -> Self {
        let phase_increment = frequency / SAMPLE_RATE as f64;
        Self {
            phase: 0.0,
            phase_increment,
        }
    }
}

impl<const SAMPLE_RATE: u32> Signal for SquareOscillator<SAMPLE_RATE> {
    fn next_sample(&mut self) -> f64 {
        let sample = if self.phase < 0.5 { 1.0 } else { -1.0 };
        self.phase += self.phase_increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        sample
    }
}

impl<const SAMPLE_RATE: u32> AudioSignal<SAMPLE_RATE> for SquareOscillator<SAMPLE_RATE> {}

impl<const SAMPLE_RATE: u32> Oscillator for SquareOscillator<SAMPLE_RATE> {
    fn set_frequency(&mut self, frequency: f64) {
        self.phase_increment = frequency / SAMPLE_RATE as f64;
    }

    fn frequency(&self) -> f64 {
        self.phase_increment * SAMPLE_RATE as f64
    }

    fn reset(&mut self) {
        self.phase = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oscillator_creation() {
        let osc = SquareOscillator::<44100>::new(440.0);
        assert_eq!(osc.frequency(), 440.0);
    }

    #[test]
    fn test_frequency_change() {
        let mut osc = SquareOscillator::<44100>::new(440.0);
        osc.set_frequency(880.0);
        assert_eq!(osc.frequency(), 880.0);
    }

    #[test]
    fn test_reset() {
        let mut osc = SquareOscillator::<44100>::new(440.0);
        for _ in 0..100 {
            osc.next_sample();
        }
        osc.reset();
        let sample = osc.next_sample();
        assert_eq!(sample, 1.0);
    }

    #[test]
    fn test_sample_generation() {
        let mut osc = SquareOscillator::<44100>::new(440.0);
        let sample = osc.next_sample();
        assert!(sample == 1.0 || sample == -1.0);
    }

    #[test]
    fn test_waveform_shape_50_percent() {
        let mut osc = SquareOscillator::<44100>::new(1.0);
        osc.reset();
        let mut high_count = 0;
        let mut low_count = 0;
        for _ in 0..44100 {
            let sample = osc.next_sample();
            if sample == 1.0 {
                high_count += 1;
            } else {
                low_count += 1;
            }
        }
        assert!((high_count as f64 / 44100.0 - 0.5).abs() < 0.01);
        assert!((low_count as f64 / 44100.0 - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_symmetric_duty_cycle() {
        let mut osc = SquareOscillator::<44100>::new(100.0);
        osc.reset();
        let mut high_count: u32 = 0;
        let mut low_count: u32 = 0;
        let samples_per_period = 44100 / 100;
        for _ in 0..samples_per_period {
            let sample = osc.next_sample();
            if sample == 1.0 {
                high_count += 1;
            } else {
                low_count += 1;
            }
        }
        assert!(high_count.abs_diff(low_count) <= 1);
    }

    #[test]
    fn test_phase_wrapping() {
        let mut osc = SquareOscillator::<44100>::new(44100.0);
        osc.next_sample();
        osc.next_sample();
        let sample = osc.next_sample();
        assert!(!sample.is_nan());
    }

    #[test]
    fn test_zero_frequency() {
        let mut osc = SquareOscillator::<44100>::new(0.0);
        let sample1 = osc.next_sample();
        let sample2 = osc.next_sample();
        assert_eq!(sample1, sample2);
    }

    #[test]
    fn test_sample_range() {
        let mut osc = SquareOscillator::<44100>::new(440.0);
        for _ in 0..1000 {
            let sample = osc.next_sample();
            assert!(sample == 1.0 || sample == -1.0);
        }
    }

    #[test]
    fn test_process_buffer() {
        let mut osc = SquareOscillator::<44100>::new(440.0);
        let mut buffer = [0.0; 128];
        osc.process(&mut buffer);
        for &sample in buffer.iter() {
            assert!(sample == 1.0 || sample == -1.0);
        }
    }
}
