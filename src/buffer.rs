//! This module provides various helper functions for performing operations on slices of samples.

use {Duplex, Frame, Sample};

/// Mutate every element in the buffer with the given function.
#[inline]
pub fn map_in_place<F, M>(a: &mut [F], mut map: M)
    where M: FnMut(F) -> F,
          F: Frame,
{
    for i in 0..a.len() {
        a[i] = map(a[i]);
    }
}

/// Sets the buffer of samples at the `Sample`'s equilibrium value.
pub fn equilibrium<F>(a: &mut [F])
    where F: Frame,
{
    map_in_place(a, |_| F::equilibrium())
}

/// Mutate every element in buffer `a` while reading from each element from buffer `b` in lock-step
/// using the given function.
///
/// **Panics** if the length of `b` is not equal to the length of `a`.
#[inline]
pub fn zip_map_in_place<F, M>(a: &mut [F], b: &[F], zip_map: M)
    where M: FnMut(F, F) -> F,
          F: Frame,
{
    assert_eq!(a.len(), b.len());
    unchecked_zip_map_in_place(a, b, zip_map);
}

/// Writes every sample in buffer `b` to buffer `a`.
///
/// **Panics** if the buffer lengths differ.
#[inline]
pub fn write<F>(a: &mut [F], b: &[F])
    where F: Frame,
{
    assert_eq!(a.len(), b.len());
    unchecked_zip_map_in_place(a, b, |_, b| b);
}

/// Adds every sample in buffer `b` to every sample in buffer `a` respectively.
#[inline]
pub fn add<F>(a: &mut [F], b: &[F])
    where F: Frame,
{
    assert_eq!(a.len(), b.len());
    unchecked_zip_map_in_place(a, b, |a, b| a.add(b));
}

// /// Sum buffer `b` onto buffer `a` after multiplying it by the amplitude per channel.
// #[inline]
// pub fn add_with_amp_per_channel<F, A>(a: &mut [F], b: &[F], amp_per_channel: A)
//     where F: Frame,
//           A: Frame<Sample=f32, Channels=Frame::Channels> + Amplitude,
// {
//     let len = a.len();
//     assert_eq!(len, b.len());
//     for i in 0..len {
//         a[i] = a[i].add(b.zip_map(amp_per_channel, 
//     }
// }

/// Sum buffer `b` onto buffer `a` after multiplying it by the amplitude per channel.
#[inline]
pub fn add_with_amp_per_channel<S>(a: &mut [S], b: &[S], amp_per_channel: &[f32])
    where S: Sample + Duplex<f32>,
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
        unchecked_zip_map_in_place(a, b, |a, b| a + b);
    }
}

/// Mutate every element in buffer `a` while reading from each element from buffer `b` in lock-step
/// using the given function.
///
/// This function does not check that the buffers are the same length and will panic on
/// index-out-of-bounds .
#[inline]
fn unchecked_zip_map_in_place<F, M>(a: &mut [F], b: &[F], mut zip_map: M)
    where M: FnMut(F, F) -> F,
          F: Frame,
{
    for i in 0..a.len() {
        a[i] = zip_map(a[i], b[i]);
    }
}
