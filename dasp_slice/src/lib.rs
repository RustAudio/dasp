//! For working with slices of PCM audio data.
//!
//! Items related to conversion between slices of frames and slices of samples, particularly useful
//! for working with interleaved data.

#![cfg_attr(not(feature = "std"), no_std)]

use dasp_frame::Frame;
use dasp_sample::Sample;

#[cfg(feature = "boxed")]
pub use boxed::{
    from_boxed_frame_slice, from_boxed_sample_slice, to_boxed_frame_slice, to_boxed_sample_slice,
    DuplexBoxedFrameSlice, DuplexBoxedSampleSlice, DuplexBoxedSlice, FromBoxedFrameSlice,
    FromBoxedSampleSlice, ToBoxedFrameSlice, ToBoxedSampleSlice,
};

pub use frame::{
    from_frame_slice, from_frame_slice_mut, to_frame_slice, to_frame_slice_mut, DuplexFrameSlice,
    DuplexFrameSliceMut, FromFrameSlice, FromFrameSliceMut, ToFrameSlice, ToFrameSliceMut,
};

#[cfg(feature = "boxed")]
pub mod boxed;

mod frame;

// Slice Conversion Traits
// ----------------------------------------------------------------------------

/// For converting from a slice of `Sample`s to a slice of `Frame`s.
pub trait FromSampleSlice<'a, S>: Sized
where
    S: Sample,
{
    fn from_sample_slice(slice: &'a [S]) -> Option<Self>;
}

/// For converting from a mutable slice of `Sample`s to a mutable slice of `Frame`s.
pub trait FromSampleSliceMut<'a, S>: Sized
where
    S: Sample,
{
    fn from_sample_slice_mut(slice: &'a mut [S]) -> Option<Self>;
}

/// For converting from a slice of `Frame`s to a slice of `Sample`s.
pub trait ToSampleSlice<'a, S>
where
    S: Sample,
{
    fn to_sample_slice(self) -> &'a [S];
}

/// For converting from a mutable slice of `Frame`s to a mutable slice of `Sample`s.
pub trait ToSampleSliceMut<'a, S>
where
    S: Sample,
{
    fn to_sample_slice_mut(self) -> &'a mut [S];
}

/// For converting to and from a slice of `Sample`s.
pub trait DuplexSampleSlice<'a, S>: FromSampleSlice<'a, S> + ToSampleSlice<'a, S>
where
    S: Sample,
{
}

/// For converting to and from a mutable slice of `Sample`s.
pub trait DuplexSampleSliceMut<'a, S>: FromSampleSliceMut<'a, S> + ToSampleSliceMut<'a, S>
where
    S: Sample,
{
}

/// For converting to and from a slice of `Sample`s of type `S` and a slice of `Frame`s of type
/// `F`.
pub trait DuplexSlice<'a, S, F>: DuplexSampleSlice<'a, S> + DuplexFrameSlice<'a, F>
where
    S: Sample,
    F: Frame<Sample = S>,
{
}

/// For converting to and from a mutable slice of `Sample`s of type `S` and a slice of `Frame`s of
/// type `F`.
pub trait DuplexSliceMut<'a, S, F>:
    DuplexSampleSliceMut<'a, S> + DuplexFrameSliceMut<'a, F>
where
    S: Sample,
    F: Frame<Sample = S>,
{
}

// Slice Conversion Trait Implementations
// ----------------------------------------------------------------------------

impl<'a, S> FromSampleSlice<'a, S> for &'a [S]
where
    S: Sample,
{
    #[inline]
    fn from_sample_slice(slice: &'a [S]) -> Option<Self> {
        Some(slice)
    }
}

impl<'a, S> FromSampleSliceMut<'a, S> for &'a mut [S]
where
    S: Sample,
{
    #[inline]
    fn from_sample_slice_mut(slice: &'a mut [S]) -> Option<Self> {
        Some(slice)
    }
}

impl<'a, S> ToSampleSlice<'a, S> for &'a [S]
where
    S: Sample,
{
    #[inline]
    fn to_sample_slice(self) -> &'a [S] {
        self
    }
}

impl<'a, S> ToSampleSliceMut<'a, S> for &'a mut [S]
where
    S: Sample,
{
    #[inline]
    fn to_sample_slice_mut(self) -> &'a mut [S] {
        self
    }
}

impl<'a, S, T> DuplexSampleSlice<'a, S> for T
where
    S: Sample,
    T: FromSampleSlice<'a, S> + ToSampleSlice<'a, S>,
{
}

impl<'a, S, T> DuplexSampleSliceMut<'a, S> for T
where
    S: Sample,
    T: FromSampleSliceMut<'a, S> + ToSampleSliceMut<'a, S>,
{
}

impl<'a, S, F, T> DuplexSlice<'a, S, F> for T
where
    S: Sample,
    F: Frame<Sample = S>,
    T: DuplexSampleSlice<'a, S> + DuplexFrameSlice<'a, F>,
{
}

impl<'a, S, F, T> DuplexSliceMut<'a, S, F> for T
where
    S: Sample,
    F: Frame<Sample = S>,
    T: DuplexSampleSliceMut<'a, S> + DuplexFrameSliceMut<'a, F>,
{
}

// Conversion Functions
// ----------------------------------------------------------------------------

/// Converts the given slice into a slice of `Sample`s.
///
/// This is a convenience function that wraps the `ToSampleSlice` trait.
///
/// # Examples
///
/// ```
/// fn main() {
///     let foo = &[[0.0, 0.5], [0.0, -0.5]][..];
///     let bar = dasp_slice::to_sample_slice(foo);
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
/// fn main() {
///     let foo = &mut [[0.0, 0.5], [0.0, -0.5]][..];
///     let bar = dasp_slice::to_sample_slice_mut(foo);
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
/// fn main() {
///     let foo = &[0.0, 0.5, 0.0, -0.5][..];
///     let bar: Option<&_> = dasp_slice::from_sample_slice(foo);
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
/// fn main() {
///     let foo = &mut [0.0, 0.5, 0.0, -0.5][..];
///     let bar: Option<&mut _> = dasp_slice::from_sample_slice_mut(foo);
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
