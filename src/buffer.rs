//! This module provides various helper functions for performing operations on slices of samples.

use {FromSample, Sample};

/// Mutate every element in the buffer with the given function.
#[inline]
pub fn map_in_place<S, F>(a: &mut [S], mut map: F)
    where F: FnMut(S) -> S,
          S: Sample,
{
    for i in 0..a.len() {
        a[i] = map(a[i]);
    }
}

/// Sets the buffer of samples at the `Sample`'s equilibrium value.
pub fn equilibrium<S>(a: &mut [S])
    where S: Sample,
{
    map_in_place(a, |_| S::equilibrium())
}

/// Mutate every element in buffer `a` while reading from each element from buffer `b` in lock-step
/// using the given function.
///
/// **Panics** if the length of `b` is not equal to the length of `a`.
#[inline]
pub fn map_in_place_with<S, F>(a: &mut [S], b: &[S], map_with: F)
    where F: FnMut(S, S) -> S,
          S: Sample,
{
    assert_eq!(a.len(), b.len());
    unchecked_map_in_place_with(a, b, map_with);
}

/// Writes every sample in buffer `b` to buffer `a`.
///
/// **Panics** if the buffer lengths differ.
#[inline]
pub fn write<S>(a: &mut [S], b: &[S])
    where S: Sample,
{
    assert_eq!(a.len(), b.len());
    unchecked_map_in_place_with(a, b, |_, b| b);
}

/// Adds every sample in buffer `b` to every sample in buffer `a` respectively.
#[inline]
pub fn add<S>(a: &mut [S], b: &[S])
    where S: Sample,
{
    assert_eq!(a.len(), b.len());
    unchecked_map_in_place_with(a, b, |a, b| a + b);
}

/// Sum buffer `b` onto buffer `a` after multiplying it by the amplitude per channel.
#[inline]
pub fn add_with_amp_per_channel<S>(a: &mut [S], b: &[S], amp_per_channel: &[f32])
    where S: Sample + FromSample<f32>,
          f32: FromSample<S>,
{
    let n_samples = a.len();
    let n_channels = amp_per_channel.len();
    assert_eq!(b.len(), n_samples);
    assert!(n_samples % n_channels == 0, "n_samples must be a multiple of n_channels");
    if n_channels > 0 {
        let n_frames = n_samples / n_channels;
        for i in 0..n_frames {
            for j in 0..n_channels {
                let idx = i * n_channels + j;
                let to_add: S = (b[idx].to_sample::<f32>() * amp_per_channel[j]).to_sample();
                a[idx] = a[idx] + to_add;
            }
        }
    } else {
        // If no amplitude per channel, simply add the buffers.
        unchecked_map_in_place_with(a, b, |a, b| a + b);
    }
}

/// Mutate every element in buffer `a` while reading from each element from buffer `b` in lock-step
/// using the given function.
///
/// This function does not check that the buffers are the same length and will panic on
/// index-out-of-bounds .
#[inline]
fn unchecked_map_in_place_with<S, F>(a: &mut [S], b: &[S], mut map_with: F)
    where F: FnMut(S, S) -> S,
          S: Sample,
{
    for i in 0..a.len() {
        a[i] = map_with(a[i], b[i]);
    }
}
