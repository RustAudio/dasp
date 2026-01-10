//! Tests for the `Converter` and `Interpolator` traits

use dasp_interpolate::{floor::Floor, linear::Linear, sinc::Sinc};
use dasp_ring_buffer as ring_buffer;
use dasp_signal::{self as signal, interpolate::Converter, Signal};

#[test]
fn test_floor_converter() {
    let frames: [f64; 3] = [0.0, 1.0, 2.0];
    let mut source = signal::from_iter(frames.iter().cloned());
    let interp = Floor::new(source.next());
    let mut conv = Converter::scale_playback_hz(source, interp, 0.5);

    assert_eq!(conv.next(), 0.0);
    assert_eq!(conv.next(), 0.0);
    assert_eq!(conv.next(), 1.0);
    assert_eq!(conv.next(), 1.0);
    // It may seem odd that we are emitting two values, but consider this: no matter what the next
    // value would be, Floor would always yield the same frame until we hit an interpolation_value
    // of 1.0 and had to advance the frame. We don't know what the future holds, so we should
    // continue yielding frames.
    assert_eq!(conv.next(), 2.0);
    assert_eq!(conv.next(), 2.0);
}

#[test]
fn test_linear_converter() {
    let frames: [f64; 3] = [0.0, 1.0, 2.0];
    let mut source = signal::from_iter(frames.iter().cloned());
    let a = source.next();
    let b = source.next();
    let interp = Linear::new(a, b);
    let mut conv = Converter::scale_playback_hz(source, interp, 0.5);

    assert_eq!(conv.next(), 0.0);
    assert_eq!(conv.next(), 0.5);
    assert_eq!(conv.next(), 1.0);
    assert_eq!(conv.next(), 1.5);
    assert_eq!(conv.next(), 2.0);
    // There's nothing else here to interpolate toward, but we do want to ensure that we're
    // emitting the correct number of frames.
    assert_eq!(conv.next(), 1.0);
}

#[test]
fn test_scale_playback_rate() {
    // Scale the playback rate by `0.5`
    let foo = [0.0, 1.0, 0.0, -1.0];
    let mut source = signal::from_iter(foo.iter().cloned());
    let a = source.next();
    let b = source.next();
    let interp = Linear::new(a, b);
    let frames: Vec<_> = source.scale_hz(interp, 0.5).take(8).collect();
    assert_eq!(
        &frames[..],
        &[0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5][..]
    );
}

#[test]
fn test_sinc() {
    let foo = [0f64, 1.0, 0.0, -1.0];
    let source = signal::from_iter(foo.iter().cloned());

    let frames = ring_buffer::Fixed::from(vec![0.0; 50]);
    let interp = Sinc::new(frames);
    let resampled = source.from_hz_to_hz(interp, 44100.0, 11025.0);

    assert_eq!(
        resampled.until_exhausted().find(|sample| sample.is_nan()),
        None
    );
}

#[test]
fn test_sinc_downsampling_antialiasing() {
    const SOURCE_HZ: f64 = 2000.0;
    const TARGET_HZ: f64 = 1500.0;
    const SIGNAL_FREQ: f64 = 900.0;

    let source_signal = signal::rate(SOURCE_HZ).const_hz(SIGNAL_FREQ).sine();

    let ring_buffer = ring_buffer::Fixed::from(vec![0.0; 100]);
    let sinc = Sinc::new(ring_buffer);
    let mut downsampled = source_signal.from_hz_to_hz(sinc, SOURCE_HZ, TARGET_HZ);

    for _ in 0..50 {
        downsampled.next();
    }

    let samples: Vec<f64> = downsampled.take(1500).collect();

    let rms = (samples.iter().map(|&s| s * s).sum::<f64>() / samples.len() as f64).sqrt();

    assert!(
        rms < 0.1,
        "Expected RMS < 0.1 (well-filtered), got {:.3}. Aliasing occurred.",
        rms
    );
}
