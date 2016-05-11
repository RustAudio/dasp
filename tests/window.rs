extern crate sample;

use sample::signal::ConstHz;
use sample::window::{Window, HanningWindow, WindowType};

#[test]
fn test_window_at_phase() {
    let mut window: Window<f64, HanningWindow> = Window::new(8);
    let exp = [0., 0.1464, 0.5000, 0.8536];
    for (r, e) in window.zip(exp.iter()) {
        println!("Exp: {}\t\tFound: {}", e, r[0]);
        assert!((r[0] - e).abs() < 0.001);
    }
}
