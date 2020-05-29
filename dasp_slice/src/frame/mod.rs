use dasp_frame::Frame;

mod fixed_size_array;

/// For converting from a slice of `Frame`s to a slice of `Sample`s.
pub trait FromFrameSlice<'a, F>
where
    F: Frame,
{
    fn from_frame_slice(slice: &'a [F]) -> Self;
}

/// For converting from a slice of `Frame`s to a slice of `Sample`s.
pub trait FromFrameSliceMut<'a, F>
where
    F: Frame,
{
    fn from_frame_slice_mut(slice: &'a mut [F]) -> Self;
}

/// For converting from a slice of `Sample`s to a slice of `Frame`s.
pub trait ToFrameSlice<'a, F>
where
    F: Frame,
{
    fn to_frame_slice(self) -> Option<&'a [F]>;
}

/// For converting from a mutable slice of `Sample`s to a mutable slice of `Frame`s.
pub trait ToFrameSliceMut<'a, F>
where
    F: Frame,
{
    fn to_frame_slice_mut(self) -> Option<&'a mut [F]>;
}

/// For converting to and from a slice of `Frame`s.
pub trait DuplexFrameSlice<'a, F>: FromFrameSlice<'a, F> + ToFrameSlice<'a, F>
where
    F: Frame,
{
}

/// For converting to and from a mutable slice of `Frame`s.
pub trait DuplexFrameSliceMut<'a, F>: FromFrameSliceMut<'a, F> + ToFrameSliceMut<'a, F>
where
    F: Frame,
{
}

impl<'a, F> FromFrameSlice<'a, F> for &'a [F]
where
    F: Frame,
{
    #[inline]
    fn from_frame_slice(slice: &'a [F]) -> Self {
        slice
    }
}

impl<'a, F> FromFrameSliceMut<'a, F> for &'a mut [F]
where
    F: Frame,
{
    #[inline]
    fn from_frame_slice_mut(slice: &'a mut [F]) -> Self {
        slice
    }
}

impl<'a, F> ToFrameSlice<'a, F> for &'a [F]
where
    F: Frame,
{
    #[inline]
    fn to_frame_slice(self) -> Option<&'a [F]> {
        Some(self)
    }
}

impl<'a, F> ToFrameSliceMut<'a, F> for &'a mut [F]
where
    F: Frame,
{
    #[inline]
    fn to_frame_slice_mut(self) -> Option<&'a mut [F]> {
        Some(self)
    }
}

impl<'a, F, T> DuplexFrameSlice<'a, F> for T
where
    F: Frame,
    T: FromFrameSlice<'a, F> + ToFrameSlice<'a, F>,
{
}

impl<'a, F, T> DuplexFrameSliceMut<'a, F> for T
where
    F: Frame,
    T: FromFrameSliceMut<'a, F> + ToFrameSliceMut<'a, F>,
{
}

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
/// fn main() {
///     let foo = &[0.0, 0.5, 0.0, -0.5][..];
///     let bar = dasp_slice::to_frame_slice(foo);
///     assert_eq!(bar, Some(&[[0.0, 0.5], [0.0, -0.5]][..]));
///
///     let foo = &[0.0, 0.5, 0.0][..];
///     let bar = dasp_slice::to_frame_slice(foo);
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
/// fn main() {
///     let foo = &mut [0.0, 0.5, 0.0, -0.5][..];
///     let bar = dasp_slice::to_frame_slice_mut(foo);
///     assert_eq!(bar, Some(&mut [[0.0, 0.5], [0.0, -0.5]][..]));
///
///     let foo = &mut [0.0, 0.5, 0.0][..];
///     let bar = dasp_slice::to_frame_slice_mut(foo);
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

/// Converts the given slice of `Frame`s into some slice `T`.
///
/// This is a convenience function that wraps the `FromFrameSlice` trait.
///
/// # Examples
///
/// ```
/// fn main() {
///     let foo = &[[0.0, 0.5], [0.0, -0.5]][..];
///     let bar: &[f32] = dasp_slice::from_frame_slice(foo);
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
/// fn main() {
///     let foo = &mut [[0.0, 0.5], [0.0, -0.5]][..];
///     let bar: &mut [f32] = dasp_slice::from_frame_slice_mut(foo);
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
