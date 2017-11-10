//! This module provides various helper functions for performing operations on slices of frames.

use {Box, Frame, Sample, ToSampleSlice, ToSampleSliceMut, ToBoxedSampleSlice, ToFrameSlice,
     ToFrameSliceMut, ToBoxedFrameSlice, FromSampleSlice, FromSampleSliceMut,
     FromBoxedSampleSlice, FromFrameSlice, FromFrameSliceMut, FromBoxedFrameSlice};


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
///     let bar = sample::slice::to_frame_slice(foo);
///     assert_eq!(bar, Some(&[[0.0, 0.5], [0.0, -0.5]][..]));
///
///     let foo = &[0.0, 0.5, 0.0][..];
///     let bar = sample::slice::to_frame_slice(foo);
///     assert_eq!(bar, None::<&[[f32; 2]]>);
/// }
/// ```
pub fn to_frame_slice<'a, T, F>(slice: T) -> Option<&'a [F]>
where
    F: Frame,
    T: ToFrameSlice<'a, F>,
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
///     let bar = sample::slice::to_frame_slice_mut(foo);
///     assert_eq!(bar, Some(&mut [[0.0, 0.5], [0.0, -0.5]][..]));
///
///     let foo = &mut [0.0, 0.5, 0.0][..];
///     let bar = sample::slice::to_frame_slice_mut(foo);
///     assert_eq!(bar, None::<&mut [[f32; 2]]>);
/// }
/// ```
pub fn to_frame_slice_mut<'a, T, F>(slice: T) -> Option<&'a mut [F]>
where
    F: Frame,
    T: ToFrameSliceMut<'a, F>,
{
    slice.to_frame_slice_mut()
}

/// Converts the given boxed slice into a boxed slice of `Frame`s.
///
/// Returns `None` if the number of channels in a single frame `F` is not a multiple of the number
/// of samples in the given slice.
///
/// This is a convenience function that wraps the `ToBoxedFrameSlice` trait.
///
/// # Examples
///
/// ```
/// extern crate sample;
///
/// fn main() {
///     let foo = vec![0.0, 0.5, 0.0, -0.5].into_boxed_slice();
///     let bar: Box<[[f32; 2]]> = sample::slice::to_boxed_frame_slice(foo).unwrap();
///     assert_eq!(bar.into_vec(), vec![[0.0, 0.5], [0.0, -0.5]]);
///
///     let foo = vec![0.0, 0.5, 0.0].into_boxed_slice();
///     let bar = sample::slice::to_boxed_frame_slice(foo);
///     assert_eq!(bar, None::<Box<[[f32; 2]]>>);
/// }
/// ```
pub fn to_boxed_frame_slice<T, F>(slice: T) -> Option<Box<[F]>>
where
    F: Frame,
    T: ToBoxedFrameSlice<F>,
{
    slice.to_boxed_frame_slice()
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
///     let bar = sample::slice::to_sample_slice(foo);
///     assert_eq!(bar, &[0.0, 0.5, 0.0, -0.5][..]);
/// }
/// ```
pub fn to_sample_slice<'a, T, S>(slice: T) -> &'a [S]
where
    S: Sample,
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
///     let bar = sample::slice::to_sample_slice_mut(foo);
///     assert_eq!(bar, &mut [0.0, 0.5, 0.0, -0.5][..]);
/// }
/// ```
pub fn to_sample_slice_mut<'a, T, S>(slice: T) -> &'a mut [S]
where
    S: Sample,
    T: ToSampleSliceMut<'a, S>,
{
    slice.to_sample_slice_mut()
}

/// Converts the given boxed slice into a boxed slice of `Sample`s.
///
/// This is a convenience function that wraps the `ToBoxedSampleSlice` trait.
///
/// # Examples
///
/// ```
/// extern crate sample;
///
/// fn main() {
///     let foo = vec![[0.0, 0.5], [0.0, -0.5]].into_boxed_slice();
///     let bar = sample::slice::to_boxed_sample_slice(foo);
///     assert_eq!(bar.into_vec(), vec![0.0, 0.5, 0.0, -0.5]);
/// }
/// ```
pub fn to_boxed_sample_slice<T, S>(slice: T) -> Box<[S]>
where
    S: Sample,
    T: ToBoxedSampleSlice<S>,
{
    slice.to_boxed_sample_slice()
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
///     let bar: Option<&_> = sample::slice::from_sample_slice(foo);
///     assert_eq!(bar, Some(&[[0.0, 0.5], [0.0, -0.5]][..]));
/// }
/// ```
pub fn from_sample_slice<'a, T, S>(slice: &'a [S]) -> Option<T>
where
    S: Sample,
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
///     let bar: Option<&mut _> = sample::slice::from_sample_slice_mut(foo);
///     assert_eq!(bar, Some(&mut [[0.0, 0.5], [0.0, -0.5]][..]));
/// }
/// ```
pub fn from_sample_slice_mut<'a, T, S>(slice: &'a mut [S]) -> Option<T>
where
    S: Sample,
    T: FromSampleSliceMut<'a, S>,
{
    T::from_sample_slice_mut(slice)
}

/// Converts the given boxed slice of `Sample`s into some slice `T`.
///
/// Returns `None` if the number of channels in a single frame is not a multiple of the number of
/// samples in the given slice.
///
/// This is a convenience function that wraps the `FromBoxedSampleSlice` trait.
///
/// # Examples
///
/// ```
/// extern crate sample;
///
/// fn main() {
///     let foo = vec![0.0, 0.5, 0.0, -0.5].into_boxed_slice();
///     let bar: Box<[[f32; 2]]> = sample::slice::from_boxed_sample_slice(foo).unwrap();
///     assert_eq!(bar.into_vec(), vec![[0.0, 0.5], [0.0, -0.5]]);
/// }
/// ```
pub fn from_boxed_sample_slice<T, S>(slice: Box<[S]>) -> Option<T>
where
    S: Sample,
    T: FromBoxedSampleSlice<S>,
{
    T::from_boxed_sample_slice(slice)
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
///     let bar: &[f32] = sample::slice::from_frame_slice(foo);
///     assert_eq!(bar, &[0.0, 0.5, 0.0, -0.5][..]);
/// }
/// ```
pub fn from_frame_slice<'a, T, F>(slice: &'a [F]) -> T
where
    F: Frame,
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
///     let bar: &mut [f32] = sample::slice::from_frame_slice_mut(foo);
///     assert_eq!(bar, &mut [0.0, 0.5, 0.0, -0.5][..]);
/// }
/// ```
pub fn from_frame_slice_mut<'a, T, F>(slice: &'a mut [F]) -> T
where
    F: Frame,
    T: FromFrameSliceMut<'a, F>,
{
    T::from_frame_slice_mut(slice)
}

/// Converts the given boxed slice of `Frame`s into some slice `T`.
///
/// This is a convenience function that wraps the `FromBoxedFrameSlice` trait.
///
/// # Examples
///
/// ```
/// extern crate sample;
///
/// fn main() {
///     let foo = vec![[0.0, 0.5], [0.0, -0.5]].into_boxed_slice();
///     let bar: Box<[f32]> = sample::slice::from_boxed_frame_slice(foo);
///     assert_eq!(bar.into_vec(), vec![0.0, 0.5, 0.0, -0.5]);
/// }
/// ```
pub fn from_boxed_frame_slice<T, F>(slice: Box<[F]>) -> T
where
    F: Frame,
    T: FromBoxedFrameSlice<F>,
{
    T::from_boxed_frame_slice(slice)
}


///// Utility Functions


/// Mutate every element in the slice with the given function.
#[inline]
pub fn map_in_place<F, M>(a: &mut [F], mut map: M)
where
    M: FnMut(F) -> F,
    F: Frame,
{
    for f in a {
        *f = map(*f);
    }
}

/// Sets the slice of frames at the associated `Sample`'s equilibrium value.
#[inline]
pub fn equilibrium<F>(a: &mut [F])
where
    F: Frame,
{
    map_in_place(a, |_| F::equilibrium())
}

/// Mutate every frame in slice `a` while reading from each frame in slice `b` in lock-step using
/// the given function.
///
/// **Panics** if the length of `b` is not equal to the length of `a`.
#[inline]
pub fn zip_map_in_place<FA, FB, M>(a: &mut [FA], b: &[FB], zip_map: M)
where
    FA: Frame,
    FB: Frame,
    M: FnMut(FA, FB) -> FA,
{
    assert_eq!(a.len(), b.len());

    // We've asserted that the lengths are equal so we don't need bounds checking.
    unsafe {
        zip_map_in_place_unchecked(a, b, zip_map);
    }
}

/// Writes every sample in slice `b` to slice `a`.
///
/// **Panics** if the slice lengths differ.
#[inline]
pub fn write<F>(a: &mut [F], b: &[F])
where
    F: Frame,
{
    zip_map_in_place(a, b, |_, b| b);
}

/// Adds every sample in slice `b` to every sample in slice `a` respectively.
#[inline]
pub fn add_in_place<FA, FB>(a: &mut [FA], b: &[FB])
where
    FA: Frame,
    FB: Frame<Sample = <FA::Sample as Sample>::Signed, NumChannels = FA::NumChannels>,
{
    zip_map_in_place(a, b, |a, b| a.add_amp(b));
}

/// Scale the amplitude of each frame in `b` by `amp_per_channel` before summing it onto `a`.
#[inline]
pub fn add_in_place_with_amp_per_channel<FA, FB, A>(a: &mut [FA], b: &[FB], amp_per_channel: A)
where
    FA: Frame,
    FB: Frame<Sample = <FA::Sample as Sample>::Signed, NumChannels = FA::NumChannels>,
    A: Frame<Sample = <FB::Sample as Sample>::Float, NumChannels = FB::NumChannels>,
{
    zip_map_in_place(a, b, |af, bf| af.add_amp(bf.mul_amp(amp_per_channel)));
}

/// Mutate every element in slice `a` while reading from each element from slice `b` in lock-step
/// using the given function.
///
/// This function does not check that the slices are the same length and will crash horrifically on
/// index-out-of-bounds.
#[inline]
unsafe fn zip_map_in_place_unchecked<FA, FB, M>(a: &mut [FA], b: &[FB], mut zip_map: M)
where
    FA: Frame,
    FB: Frame,
    M: FnMut(FA, FB) -> FA,
{
    for i in 0..a.len() {
        *a.get_unchecked_mut(i) = zip_map(*a.get_unchecked(i), *b.get_unchecked(i));
    }
}
