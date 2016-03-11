//! Tests for the `Signal` trait.

extern crate sample;

use sample::Signal;

#[test]
fn test_equilibrium() {
    let equilibrium: Vec<[i8; 1]> = sample::signal::equilibrium().take(4).collect();
    assert_eq!(equilibrium, vec![[0], [0], [0], [0]]);
}

#[test]
fn test_zip_add() {
    let frames = [[0.0], [0.1], [0.2], [0.3]];
    let a = frames.iter().cloned();
    let b = frames.iter().cloned();
    let added: Vec<_> = a.zip_add(b).collect();
    assert_eq!(added, vec![[0.0], [0.2], [0.4], [0.6]]);
}

#[test]
fn test_zip_mod_amp() {
    let foo = [[255u8, 186], [64, 32]];
    let bar = [[0.0, 0.5], [-1.0, -0.25]];
    let a = foo.iter().cloned();
    let b = bar.iter().cloned();
    let amp_modulated: Vec<_> = a.zip_mod_amp(b).collect();
    assert_eq!(amp_modulated, vec![[128, 157], [192, 152]]);
}

#[test]
fn test_scale_amp() {
    let foo = [[0.5], [0.8], [-0.4], [-0.2]];
    let amp = 0.5;
    let amp_scaled: Vec<_> = foo.iter().cloned().scale_amp(amp).collect();
    assert_eq!(amp_scaled, vec![[0.25], [0.4], [-0.2], [-0.1]]);
}
