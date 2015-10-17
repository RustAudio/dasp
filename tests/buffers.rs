
extern crate sample;

use sample::Sample;

#[test]
fn test_add_buffer() {
    let mut a = [0.5; 32];
    let b = [0.5; 32];
    f32::add_buffer(&mut a, &b);
    assert_eq!([1.0; 32], a);
}

#[test]
#[should_panic]
fn test_add_buffer_panic() {
    let mut a = [0.5; 31];
    let b = [0.5; 32];
    f32::add_buffer(&mut a, &b);
}

#[test]
fn test_write_buffer() {
    let mut a = [0.0; 32];
    let b = [1.0; 32];
    f32::write_buffer(&mut a, &b);
    assert_eq!([1.0; 32], a);
}

#[test]
#[should_panic]
fn test_write_buffer_panic() {
    let mut a = [0.0; 31];
    let b = [1.0; 32];
    f32::write_buffer(&mut a, &b);
}

#[test]
fn test_add_buffer_with_amp_per_channel() {
    let mut a = [0.5; 32];
    let b = [1.0; 32];
    let amp = [0.5; 2];
    f32::add_buffer_with_amp_per_channel(&mut a, &b, &amp);
    assert_eq!([1.0; 32], a);
}

#[test]
#[should_panic]
fn test_add_buffer_with_amp_per_channel_panic() {
    let mut a = [0.5; 31];
    let b = [1.0; 32];
    let amp = [0.5; 2];
    f32::add_buffer_with_amp_per_channel(&mut a, &b, &amp);
}

