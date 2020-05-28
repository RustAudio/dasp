//! Tests for the `Converter` and `Interpolator` traits

use dasp_interpolate::{floor::Floor, linear::Linear, sinc::Sinc};
use dasp_ring_buffer as ring_buffer;
use dasp_signal::{self as signal, interpolate::Converter, Signal};

#[test]
fn test_floor_converter() {
    let frames: [[f64; 1]; 3] = [[0.0], [1.0], [2.0]];
    let mut source = signal::from_iter(frames.iter().cloned());
    let interp = Floor::new(source.next());
    let mut conv = Converter::scale_playback_hz(source, interp, 0.5);

    assert_eq!(conv.next(), [0.0]);
    assert_eq!(conv.next(), [0.0]);
    assert_eq!(conv.next(), [1.0]);
    assert_eq!(conv.next(), [1.0]);
    // It may seem odd that we are emitting two values, but consider this: no matter what the next
    // value would be, Floor would always yield the same frame until we hit an interpolation_value
    // of 1.0 and had to advance the frame. We don't know what the future holds, so we should
    // continue yielding frames.
    assert_eq!(conv.next(), [2.0]);
    assert_eq!(conv.next(), [2.0]);
}

#[test]
fn test_linear_converter() {
    let frames: [[f64; 1]; 3] = [[0.0], [1.0], [2.0]];
    let mut source = signal::from_iter(frames.iter().cloned());
    let a = source.next();
    let b = source.next();
    let interp = Linear::new(a, b);
    let mut conv = Converter::scale_playback_hz(source, interp, 0.5);

    assert_eq!(conv.next(), [0.0]);
    assert_eq!(conv.next(), [0.5]);
    assert_eq!(conv.next(), [1.0]);
    assert_eq!(conv.next(), [1.5]);
    assert_eq!(conv.next(), [2.0]);
    // There's nothing else here to interpolate toward, but we do want to ensure that we're
    // emitting the correct number of frames.
    assert_eq!(conv.next(), [1.0]);
}

#[test]
fn test_scale_playback_rate() {
    // Scale the playback rate by `0.5`
    let foo = [[0.0], [1.0], [0.0], [-1.0]];
    let mut source = signal::from_iter(foo.iter().cloned());
    let a = source.next();
    let b = source.next();
    let interp = Linear::new(a, b);
    let frames: Vec<_> = source.scale_hz(interp, 0.5).take(8).collect();
    assert_eq!(
        &frames[..],
        &[[0.0], [0.5], [1.0], [0.5], [0.0], [-0.5], [-1.0], [-0.5]][..]
    );
}

#[test]
fn test_sinc() {
    let foo = [[0.0f64], [1.0], [0.0], [-1.0]];
    let source = signal::from_iter(foo.iter().cloned());

    let frames = ring_buffer::Fixed::from(vec![[0.0]; 50]);
    let interp = Sinc::new(frames);
    let resampled = source.from_hz_to_hz(interp, 44100.0, 11025.0);

    assert_eq!(
        resampled
            .until_exhausted()
            .find(|sample| sample[0].is_nan()),
        None
    );
}
