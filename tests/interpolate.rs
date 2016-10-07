//! Tests for the `Converter` and `Interpolator` traits

extern crate sample;

use sample::interpolate::{Converter, Floor, Linear};

#[test]
fn test_floor_converter() {
    let frames: [[f64; 1]; 3] = [[0.0], [1.0], [2.0]];
    let mut source = frames.iter().cloned();
    let interp = Floor::from_source(&mut source).unwrap();
    let mut conv = Converter::scale_playback_hz(source, interp, 0.5);

    assert_eq!(conv.next(), Some([0.0]));
    assert_eq!(conv.next(), Some([0.0]));
    assert_eq!(conv.next(), Some([1.0]));
    assert_eq!(conv.next(), Some([1.0]));
    // It may seem odd that we are emitting two values, but consider this: no matter what the next
    // value would be, Floor would always yield the same frame until we hit an interpolation_value
    // of 1.0 and had to advance the frame. We don't know what the future holds, so we should
    // continue yielding frames. 
    assert_eq!(conv.next(), Some([2.0]));
    assert_eq!(conv.next(), Some([2.0]));
    assert_eq!(conv.next(), None);
}

#[test]
fn test_linear_converter() {
    let frames: [[f64; 1]; 3] = [[0.0], [1.0], [2.0]];
    let mut source = frames.iter().cloned();
    let interp = Linear::from_source(&mut source).unwrap();
    let mut conv = Converter::scale_playback_hz(source, interp, 0.5);

    assert_eq!(conv.next(), Some([0.0]));
    assert_eq!(conv.next(), Some([0.5]));
    assert_eq!(conv.next(), Some([1.0]));
    assert_eq!(conv.next(), Some([1.5]));
    assert_eq!(conv.next(), Some([2.0]));
    // There's nothing else here to interpolate toward, but we do want to ensure that we're
    // emitting the correct number of frames.
    assert_eq!(conv.next(), Some([1.0]));
    assert_eq!(conv.next(), None);
}

