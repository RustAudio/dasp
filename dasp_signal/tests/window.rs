#![cfg(feature = "window")]

use dasp_frame::Frame;
use dasp_signal::window::{self, Windower};

#[cfg(feature = "window-hann")]
#[test]
fn test_window_at_phase() {
    let window = window::hann::<f64>(9);
    let expected = [
        0.0, 0.1464, 0.5000, 0.8536, 1., 0.8536, 0.5000, 0.1464, 0., 0.1464,
    ];
    for (r, e) in window.zip(&expected) {
        println!("Expected: {}\t\tFound: {}", e, r);
        assert!((r - e).abs() < 0.001);
    }
}

#[test]
fn test_windower() {
    let data = [0.1f64, 0.1, 0.2, 0.2, 0.3, 0.3, 0.4, 0.4];
    let expected = [
        [0.1f64, 0.1],
        [0.1, 0.2],
        [0.2, 0.2],
        [0.2, 0.3],
        [0.3, 0.3],
        [0.3, 0.4],
        [0.4, 0.4],
    ];
    let windower = Windower::rectangle(&data, 2, 1);
    for (chunk, expected_chunk) in windower.zip(&expected) {
        for (r, e) in chunk.zip(expected_chunk.iter()) {
            for (r_chan, e_chan) in r.channels().zip(e.channels()) {
                println!("Expected: {}\t\tFound: {}", e_chan, r_chan);
                assert!((r_chan - e_chan).abs() < 0.001);
            }
        }
    }
}

#[cfg(feature = "window-hann")]
#[test]
fn test_window_size() {
    let v = [1f32; 16];
    let windows: Vec<Vec<_>> = Windower::hann(&v, 8, 4)
        .map(|i| i.take(8).collect::<Vec<f32>>())
        .take(3)
        .collect();
    assert_eq!(windows.len(), 3);
}
