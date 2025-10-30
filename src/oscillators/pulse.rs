//! Pulse wave oscillator with modulating duty cycle.

use super::Oscillator;
use crate::{AudioSignal, Param, Signal};

pub struct PulseOscillator<const SAMPLE_RATE: u32> {
    phase: f64,
    phase_increment: f64,
    duty_cycle: Param,
}

impl<const SAMPLE_RATE: u32> PulseOscillator<SAMPLE_RATE> {
    pub fn new(frequency: f64, duty_cycle: Param) -> Self {
        let phase_increment = frequency / SAMPLE_RATE as f64;
        Self {
            phase: 0.0,
            phase_increment,
            duty_cycle,
        }
    }
}

impl<const SAMPLE_RATE: u32> AudioSignal<SAMPLE_RATE> for PulseOscillator<SAMPLE_RATE> {}

impl<const SAMPLE_RATE: u32> Signal for PulseOscillator<SAMPLE_RATE> {
    fn next_sample(&mut self) -> f64 {
        let duty = self.duty_cycle.value();
        let duty = (duty * 0.5 + 0.5).clamp(0.0, 1.0);
        let sample = if self.phase < duty { 1.0 } else { -1.0 };
        self.phase += self.phase_increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        sample
    }
}

impl<const SAMPLE_RATE: u32> Oscillator for PulseOscillator<SAMPLE_RATE> {
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
    use crate::SineOscillator;

    #[test]
    fn test_oscillator_creation() {
        let osc = PulseOscillator::<44100>::new(440.0, 0.5.into());
        assert_eq!(osc.frequency(), 440.0);
    }

    #[test]
    fn test_frequency_change() {
        let mut osc = PulseOscillator::<44100>::new(440.0, 0.5.into());
        osc.set_frequency(880.0);
        assert_eq!(osc.frequency(), 880.0);
    }

    #[test]
    fn test_sample_generation() {
        let mut osc = PulseOscillator::<44100>::new(440.0, 0.5.into());
        let sample = osc.next_sample();
        assert_eq!(sample, 1.0);
    }

    #[test]
    fn test_sample_range() {
        let mut osc = PulseOscillator::<44100>::new(440.0, 0.5.into());
        for _ in 0..44100 {
            let sample = osc.next_sample();
            assert!(sample == -1.0 || sample == 1.0);
        }
    }

    #[test]
    fn test_waveform_shape_50_percent() {
        let mut osc = PulseOscillator::<100>::new(1.0, 0.5.into());
        let s1 = osc.next_sample();
        assert_eq!(s1, 1.0);
        for _ in 0..74 {
            let sample = osc.next_sample();
            assert_eq!(sample, 1.0);
        }
        let s2 = osc.next_sample();
        assert_eq!(s2, -1.0);
    }

    #[test]
    fn test_waveform_shape_25_percent() {
        let mut osc = PulseOscillator::<100>::new(1.0, 0.25.into());
        let s1 = osc.next_sample();
        assert_eq!(s1, 1.0);
        for _ in 0..61 {
            let sample = osc.next_sample();
            assert_eq!(sample, 1.0);
        }
        let s2 = osc.next_sample();
        assert_eq!(s2, 1.0);
        let s3 = osc.next_sample();
        assert_eq!(s3, -1.0);
    }

    #[test]
    fn test_phase_wrapping() {
        let mut osc = PulseOscillator::<44100>::new(1000.0, 0.5.into());
        for _ in 0..100000 {
            osc.next_sample();
        }
        assert!(osc.phase >= 0.0 && osc.phase < 1.0);
    }

    #[test]
    fn test_reset() {
        let mut osc = PulseOscillator::<44100>::new(440.0, 0.5.into());
        for _ in 0..100 {
            osc.next_sample();
        }
        osc.reset();
        assert_eq!(osc.phase, 0.0);
    }

    #[test]
    fn test_process_buffer() {
        let mut osc = PulseOscillator::<44100>::new(440.0, 0.5.into());
        let mut buffer = vec![0.0; 128];
        osc.process(&mut buffer);
        for sample in buffer {
            assert!(sample == -1.0 || sample == 1.0);
        }
    }

    #[test]
    fn test_zero_frequency() {
        let mut osc = PulseOscillator::<44100>::new(0.0, 0.5.into());
        let sample1 = osc.next_sample();
        let sample2 = osc.next_sample();
        assert_eq!(sample1, sample2);
    }

    #[test]
    fn test_modulating_duty_cycle() {
        let lfo = SineOscillator::<100>::new(1.0);
        let mut osc = PulseOscillator::<100>::new(10.0, lfo.into());
        for _ in 0..100 {
            for _ in 0..10 {
                osc.next_sample();
            }
        }
    }

    #[test]
    fn test_duty_cycle_scaling() {
        let mut osc = PulseOscillator::<100>::new(1.0, (-1.0).into());
        let sample1 = osc.next_sample();
        assert_eq!(sample1, -1.0);

        let mut osc = PulseOscillator::<100>::new(1.0, 1.0.into());
        let sample2 = osc.next_sample();
        assert_eq!(sample2, 1.0);
    }
}
