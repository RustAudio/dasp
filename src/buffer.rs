//! This module provides various helper functions for performing operations on slices of frames.

use {
    Amplitude, Duplex, Frame, Sample,
    ToSampleSlice, ToSampleSliceMut, ToFrameSlice, ToFrameSliceMut,
    FromSampleSlice, FromSampleSliceMut, FromFrameSlice, FromFrameSliceMut,
};


///// Conversion Functions
/////
///// The following functions wrap the various DSP slice conversion traits for convenience.


/// Converts the given slice into a slice of `Frame`s.
///
/// Returns `None` if the number of channels in a single frame `F` is not a multiple of the number
/// of samples in the given slice.
///
/// This is a convenience function that wraps the `ToFrameSlice` trait.
///
/// # Examples
///
/// ```
/// extern crate sample;
///
/// fn main() {
///     let foo = &[0.0, 0.5, 0.0, -0.5][..];
///     let bar = sample::buffer::to_frame_slice(foo);
///     assert_eq!(bar, Some(&[[0.0, 0.5], [0.0, -0.5]][..]));
///
///     let foo = &[0.0, 0.5, 0.0][..];
///     let bar = sample::buffer::to_frame_slice(foo);
///     assert_eq!(bar, None::<&[[f32; 2]]>);
/// }
/// ```
pub fn to_frame_slice<'a, T, F>(slice: T) -> Option<&'a [F]>
    where F: Frame,
          T: ToFrameSlice<'a, F>
{
    slice.to_frame_slice()
}

/// Converts the given mutable slice into a mutable slice of `Frame`s.
///
/// Returns `None` if the number of channels in a single frame `F` is not a multiple of the number
/// of samples in the given slice.
///
/// This is a convenience function that wraps the `ToFrameSliceMut` trait.
///
/// # Examples
///
/// ```
/// extern crate sample;
///
/// fn main() {
///     let foo = &mut [0.0, 0.5, 0.0, -0.5][..];
///     let bar = sample::buffer::to_frame_slice_mut(foo);
///     assert_eq!(bar, Some(&mut [[0.0, 0.5], [0.0, -0.5]][..]));
///
///     let foo = &mut [0.0, 0.5, 0.0][..];
///     let bar = sample::buffer::to_frame_slice_mut(foo);
///     assert_eq!(bar, None::<&mut [[f32; 2]]>);
/// }
/// ```
pub fn to_frame_slice_mut<'a, T, F>(slice: T) -> Option<&'a mut [F]>
    where F: Frame,
          T: ToFrameSliceMut<'a, F>
{
    slice.to_frame_slice_mut()
}

/// Converts the given slice into a slice of `Sample`s.
///
/// This is a convenience function that wraps the `ToSampleSlice` trait.
///
/// # Examples
///
/// ```
/// extern crate sample;
///
/// fn main() {
///     let foo = &[[0.0, 0.5], [0.0, -0.5]][..];
///     let bar = sample::buffer::to_sample_slice(foo);
///     assert_eq!(bar, &[0.0, 0.5, 0.0, -0.5][..]);
/// }
/// ```
pub fn to_sample_slice<'a, T, S>(slice: T) -> &'a [S]
    where S: Sample,
          T: ToSampleSlice<'a, S>,
{
    slice.to_sample_slice()
}

/// Converts the given mutable slice of `Frame`s into a mutable slice of `Sample`s.
///
/// This is a convenience function that wraps the `ToSampleSliceMut` trait.
///
/// # Examples
///
/// ```
/// extern crate sample;
///
/// fn main() {
///     let foo = &mut [[0.0, 0.5], [0.0, -0.5]][..];
///     let bar = sample::buffer::to_sample_slice_mut(foo);
///     assert_eq!(bar, &mut [0.0, 0.5, 0.0, -0.5][..]);
/// }
/// ```
pub fn to_sample_slice_mut<'a, T, S>(slice: T) -> &'a mut [S]
    where S: Sample,
          T: ToSampleSliceMut<'a, S>,
{
    slice.to_sample_slice_mut()
}

/// Converts the given slice of `Sample`s into some slice `T`.
///
/// Returns `None` if the number of channels in a single frame is not a multiple of the number of
/// samples in the given slice.
///
/// This is a convenience function that wraps the `FromSampleSlice` trait.
///
/// # Examples
///
/// ```
/// extern crate sample;
///
/// fn main() {
///     let foo = &[0.0, 0.5, 0.0, -0.5][..];
///     let bar: Option<&_> = sample::buffer::from_sample_slice(foo);
///     assert_eq!(bar, Some(&[[0.0, 0.5], [0.0, -0.5]][..]));
/// }
/// ```
pub fn from_sample_slice<'a, T, S>(slice: &'a [S]) -> Option<T>
    where S: Sample,
          T: FromSampleSlice<'a, S>,
{
    T::from_sample_slice(slice)
}

/// Converts the given mutable slice of `Sample`s into some mutable slice `T`.
///
/// Returns `None` if the number of channels in a single frame is not a multiple of the number of
/// samples in the given slice.
///
/// This is a convenience function that wraps the `FromSampleSliceMut` trait.
///
/// # Examples
///
/// ```
/// extern crate sample;
///
/// fn main() {
///     let foo = &mut [0.0, 0.5, 0.0, -0.5][..];
///     let bar: Option<&mut _> = sample::buffer::from_sample_slice_mut(foo);
///     assert_eq!(bar, Some(&mut [[0.0, 0.5], [0.0, -0.5]][..]));
/// }
/// ```
pub fn from_sample_slice_mut<'a, T, S>(slice: &'a mut [S]) -> Option<T>
    where S: Sample,
          T: FromSampleSliceMut<'a, S>,
{
    T::from_sample_slice_mut(slice)
}

/// Converts the given slice of `Frame`s into some slice `T`.
///
/// This is a convenience function that wraps the `FromFrameSlice` trait.
///
/// # Examples
///
/// ```
/// extern crate sample;
///
/// fn main() {
///     let foo = &[[0.0, 0.5], [0.0, -0.5]][..];
///     let bar: &[f32] = sample::buffer::from_frame_slice(foo);
///     assert_eq!(bar, &[0.0, 0.5, 0.0, -0.5][..]);
/// }
/// ```
pub fn from_frame_slice<'a, T, F>(slice: &'a [F]) -> T
    where F: Frame,
          T: FromFrameSlice<'a, F>,
{
    T::from_frame_slice(slice)
}

/// Converts the given slice of mutable `Frame`s into some mutable slice `T`.
///
/// This is a convenience function that wraps the `FromFrameSliceMut` trait.
///
/// # Examples
///
/// ```
/// extern crate sample;
///
/// fn main() {
///     let foo = &mut [[0.0, 0.5], [0.0, -0.5]][..];
///     let bar: &mut [f32] = sample::buffer::from_frame_slice_mut(foo);
///     assert_eq!(bar, &mut [0.0, 0.5, 0.0, -0.5][..]);
/// }
/// ```
pub fn from_frame_slice_mut<'a, T, F>(slice: &'a mut [F]) -> T
    where F: Frame,
          T: FromFrameSliceMut<'a, F>,
{
    T::from_frame_slice_mut(slice)
}


///// Utility Functions


/// Mutate every element in the buffer with the given function.
#[inline]
pub fn map_in_place<F, M>(a: &mut [F], mut map: M)
    where M: FnMut(F) -> F,
          F: Frame,
{
    for f in a {
        *f = map(*f);
    }
}

/// Sets the buffer of samples at the `Sample`'s equilibrium value.
#[inline]
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
pub fn zip_map_in_place<FA, FB, M>(a: &mut [FA], b: &[FB], zip_map: M)
    where FA: Frame,
          FB: Frame,
          M: FnMut(FA, FB) -> FA,
{
    assert_eq!(a.len(), b.len());

    // We've asserted that the lengths are equal so we don't need bounds checking.
    unsafe {
        zip_map_in_place_unchecked(a, b, zip_map);
    }
}

/// Writes every sample in buffer `b` to buffer `a`.
///
/// **Panics** if the buffer lengths differ.
#[inline]
pub fn write<F>(a: &mut [F], b: &[F])
    where F: Frame,
{
    zip_map_in_place(a, b, |_, b| b);
}

/// Adds every sample in buffer `b` to every sample in buffer `a` respectively.
#[inline]
pub fn add_in_place<F>(a: &mut [F], b: &[F])
    where F: Frame,
{
    zip_map_in_place(a, b, |a, b| a.add(b));
}

/// Scale the amplitude of each frame in `b` by `amp_per_channel` before summing it onto `a`.
#[inline]
pub fn add_in_place_with_amp_per_channel<F, A>(a: &mut [F], b: &[F], amp_per_channel: A)
    where F: Frame,
          A: Frame<NumChannels=F::NumChannels>,
          A::Sample: Amplitude,
          F::Sample: Duplex<A::Sample>,
{
    zip_map_in_place(a, b, |af, bf| af.add(bf.zip_map(amp_per_channel, Sample::scale_amplitude)));
}

/// Mutate every element in buffer `a` while reading from each element from buffer `b` in lock-step
/// using the given function.
///
/// This function does not check that the buffers are the same length and will panic on
/// index-out-of-bounds .
#[inline]
unsafe fn zip_map_in_place_unchecked<FA, FB, M>(a: &mut [FA], b: &[FB], mut zip_map: M)
    where FA: Frame,
          FB: Frame,
          M: FnMut(FA, FB) -> FA,
{
    for i in 0..a.len() {
        *a.get_unchecked_mut(i) = zip_map(*a.get_unchecked(i), *b.get_unchecked(i));
    }
}
