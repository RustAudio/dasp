//! Items related to boxed-slice conversions.

#[cfg(not(feature = "std"))]
extern crate alloc;

use dasp_frame::Frame;
use dasp_sample::Sample;

#[cfg(not(feature = "std"))]
pub type Box<T> = alloc::boxed::Box<T>;
#[cfg(feature = "std")]
pub type Box<T> = std::boxed::Box<T>;

// Traits
// ----------------------------------------------------------------------------

/// For converting a boxed slice of `Sample`s to a boxed slice of `Frame`s.
pub trait FromBoxedSampleSlice<S>: Sized
where
    S: Sample,
{
    fn from_boxed_sample_slice(slice: Box<[S]>) -> Option<Self>;
}

/// For converting from a boxed slice of `Frame`s to a boxed slice of `Sample`s.
pub trait FromBoxedFrameSlice<F>
where
    F: Frame,
{
    fn from_boxed_frame_slice(slice: Box<[F]>) -> Self;
}

/// For converting from a boxed slice of `Frame`s to a boxed slice of `Sample`s.
pub trait ToBoxedSampleSlice<S>
where
    S: Sample,
{
    fn to_boxed_sample_slice(self) -> Box<[S]>;
}

/// For converting from a boxed slice of `Sample`s to a boxed slice of `Frame`s.
pub trait ToBoxedFrameSlice<F>
where
    F: Frame,
{
    fn to_boxed_frame_slice(self) -> Option<Box<[F]>>;
}

/// For converting to and from a boxed slice of `Sample`s.
pub trait DuplexBoxedSampleSlice<S>: FromBoxedSampleSlice<S> + ToBoxedSampleSlice<S>
where
    S: Sample,
{
}

/// For converting to and from a boxed slice of `Frame`s.
pub trait DuplexBoxedFrameSlice<F>: FromBoxedFrameSlice<F> + ToBoxedFrameSlice<F>
where
    F: Frame,
{
}

/// For converting to and from a boxed slice of `Sample`s of type `S` and a slice of `Frame`s of
/// type `F`.
pub trait DuplexBoxedSlice<S, F>: DuplexBoxedSampleSlice<S> + DuplexBoxedFrameSlice<F>
where
    S: Sample,
    F: Frame<Sample = S>,
{
}

// Implementations
// ----------------------------------------------------------------------------

impl<S> FromBoxedSampleSlice<S> for Box<[S]>
where
    S: Sample,
{
    #[inline]
    fn from_boxed_sample_slice(slice: Box<[S]>) -> Option<Self> {
        Some(slice)
    }
}

impl<F> FromBoxedFrameSlice<F> for Box<[F]>
where
    F: Frame,
{
    #[inline]
    fn from_boxed_frame_slice(slice: Box<[F]>) -> Self {
        slice
    }
}

impl<S> ToBoxedSampleSlice<S> for Box<[S]>
where
    S: Sample,
{
    #[inline]
    fn to_boxed_sample_slice(self) -> Box<[S]> {
        self
    }
}

impl<F> ToBoxedFrameSlice<F> for Box<[F]>
where
    F: Frame,
{
    #[inline]
    fn to_boxed_frame_slice(self) -> Option<Box<[F]>> {
        Some(self)
    }
}

impl<S, T> DuplexBoxedSampleSlice<S> for T
where
    S: Sample,
    T: FromBoxedSampleSlice<S> + ToBoxedSampleSlice<S>,
{
}

impl<F, T> DuplexBoxedFrameSlice<F> for T
where
    F: Frame,
    T: FromBoxedFrameSlice<F> + ToBoxedFrameSlice<F>,
{
}

impl<S, F, T> DuplexBoxedSlice<S, F> for T
where
    S: Sample,
    F: Frame<Sample = S>,
    T: DuplexBoxedSampleSlice<S> + DuplexBoxedFrameSlice<F>,
{
}

// Free Functions
// ----------------------------------------------------------------------------

/// Converts the given boxed slice into a boxed slice of `Sample`s.
///
/// This is a convenience function that wraps the `ToBoxedSampleSlice` trait.
///
/// # Examples
///
/// ```
/// fn main() {
///     let foo = vec![[0.0, 0.5], [0.0, -0.5]].into_boxed_slice();
///     let bar = dasp_slice::to_boxed_sample_slice(foo);
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
/// fn main() {
///     let foo = vec![0.0, 0.5, 0.0, -0.5].into_boxed_slice();
///     let bar: Box<[[f32; 2]]> = dasp_slice::to_boxed_frame_slice(foo).unwrap();
///     assert_eq!(bar.into_vec(), vec![[0.0, 0.5], [0.0, -0.5]]);
///
///     let foo = vec![0.0, 0.5, 0.0].into_boxed_slice();
///     let bar = dasp_slice::to_boxed_frame_slice(foo);
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
/// fn main() {
///     let foo = vec![0.0, 0.5, 0.0, -0.5].into_boxed_slice();
///     let bar: Box<[[f32; 2]]> = dasp_slice::from_boxed_sample_slice(foo).unwrap();
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

/// Converts the given boxed slice of `Frame`s into some slice `T`.
///
/// This is a convenience function that wraps the `FromBoxedFrameSlice` trait.
///
/// # Examples
///
/// ```
/// fn main() {
///     let foo = vec![[0.0, 0.5], [0.0, -0.5]].into_boxed_slice();
///     let bar: Box<[f32]> = dasp_slice::from_boxed_frame_slice(foo);
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
