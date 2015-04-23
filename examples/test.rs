
extern crate sample;

use sample::Sample;

// NOTE: temporary replacement for unstable `std::iter::iterate`
struct Iter {
    value: f64,
}
impl Iterator for Iter {
    type Item = f64;
    fn next(&mut self) -> Option<f64> { self.value += 0.03; Some(self.value) }
}


fn main() {

    for phase in (Iter { value: 0.0 }).take(50) {
        let wave = sine_wave(phase);

        println!("Wave {}", wave);

        let sample_f32: f32 = Sample::from_wave(wave);
        let sample_u8 : u8  = Sample::from_wave(wave);
        let sample_u16: u16 = Sample::from_wave(wave);
        let sample_u32: u32 = Sample::from_wave(wave);
        let sample_i8 : i8  = Sample::from_wave(wave);
        let sample_i16: i16 = Sample::from_wave(wave);
        let sample_i32: i32 = Sample::from_wave(wave);

        println!("\tFrom wave to sample -> f32: {:<10} u8: {:<10} u16: {:<10} u32: {:<10} i8: {:<10} i16: {:<10} i32: {:<10}",
                 sample_f32, sample_u8, sample_u16, sample_u32, sample_i8, sample_i16, sample_i32);


        let wave_f32 = sample_f32.to_wave();
        let wave_u8  = sample_u8.to_wave();
        let wave_u16 = sample_u16.to_wave();
        let wave_u32 = sample_u32.to_wave();
        let wave_i8  = sample_i8.to_wave();
        let wave_i16 = sample_i16.to_wave();
        let wave_i32 = sample_i32.to_wave();

        println!("\tFrom sample to wave -> f32: {:<10} u8: {:<10} u16: {:<10} u32: {:<10} i8: {:<10} i16: {:<10} i32: {:<10}\n",
                 wave_f32, wave_u8, wave_u16, wave_u32, wave_i8, wave_i16, wave_i32);

        // Check that the conversion back to wave is the same as the original wave (with some
        // headroom for resolution/rounding error).
        let headroom = 0.05;
        assert!((wave_f32 - wave).abs() < headroom, "{}", wave_f32);
        assert!((wave_u8  - wave).abs() < headroom, "{}", wave_u8);
        assert!((wave_u16 - wave).abs() < headroom, "{}", wave_u16);
        assert!((wave_u32 - wave).abs() < headroom, "{}", wave_u32);
        assert!((wave_i8  - wave).abs() < headroom, "{}", wave_i8);
        assert!((wave_i16 - wave).abs() < headroom, "{}", wave_i16);
        assert!((wave_i32 - wave).abs() < headroom, "{}", wave_i32);
    }

}

/// Return a sine wave for the given phase.
fn sine_wave<S: Sample>(phase: f64) -> S {
    use std::f64::consts::PI;
    Sample::from_wave((phase * PI * 2.0).sin() as f32)
}
