//! A short example that converts an f64 sine wave to a few of the sample types available within
//! the `Sample` crate, prints their values, and then converts them back to the original f64.

use dasp::Sample;

/// An iterator that continually steps forward the phase for a signal by `0.03`.
struct Iter {
    value: f64,
}

impl Iterator for Iter {
    type Item = f64;
    fn next(&mut self) -> Option<f64> {
        self.value += 0.03;
        Some(self.value)
    }
}

fn main() {
    for phase in (Iter { value: 0.0 }).take(50) {
        // Return a sine wave for the given phase.
        fn sine_wave(phase: f64) -> f64 {
            use std::f64::consts::PI;
            (phase * PI * 2.0).sin()
        }

        let wave = sine_wave(phase);

        println!("Wave {}", wave);

        let sample_f32 = f32::from_sample(wave);
        let sample_u8 = u8::from_sample(wave);
        let sample_u16 = u16::from_sample(wave);
        let sample_u32 = u32::from_sample(wave);
        let sample_i8 = i8::from_sample(wave);
        let sample_i16 = i16::from_sample(wave);
        let sample_i32 = i32::from_sample(wave);

        println!(
            "\tFrom wave to sample -> f32: {:<10} u8: {:<10} u16: {:<10} u32: {:<10} i8: {:<10} i16: {:<10} i32: {:<10}",
            sample_f32,
            sample_u8,
            sample_u16,
            sample_u32,
            sample_i8,
            sample_i16,
            sample_i32
        );

        let wave_f32 = sample_f32.to_sample::<f64>();
        let wave_u8 = sample_u8.to_sample::<f64>();
        let wave_u16 = sample_u16.to_sample::<f64>();
        let wave_u32 = sample_u32.to_sample::<f64>();
        let wave_i8 = sample_i8.to_sample::<f64>();
        let wave_i16 = sample_i16.to_sample::<f64>();
        let wave_i32 = sample_i32.to_sample::<f64>();

        println!(
            "\tFrom sample to wave -> f32: {:<10} u8: {:<10} u16: {:<10} u32: {:<10} i8: {:<10} i16: {:<10} i32: {:<10}\n",
            wave_f32,
            wave_u8,
            wave_u16,
            wave_u32,
            wave_i8,
            wave_i16,
            wave_i32
        );

        // Check that the conversion back to wave is the same as the original wave (with some
        // headroom for resolution/rounding error).
        let headroom = 0.05;
        assert!((wave_f32 - wave).abs() < headroom, "{}", wave_f32);
        assert!((wave_u8 - wave).abs() < headroom, "{}", wave_u8);
        assert!((wave_u16 - wave).abs() < headroom, "{}", wave_u16);
        assert!((wave_u32 - wave).abs() < headroom, "{}", wave_u32);
        assert!((wave_i8 - wave).abs() < headroom, "{}", wave_i8);
        assert!((wave_i16 - wave).abs() < headroom, "{}", wave_i16);
        assert!((wave_i32 - wave).abs() < headroom, "{}", wave_i32);
    }
}
