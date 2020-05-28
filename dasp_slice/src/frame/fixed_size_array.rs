//! Implementations of the slice conversion traits for slices of fixed-size-array frames.

use crate::{
    FromFrameSlice, FromFrameSliceMut, FromSampleSlice, FromSampleSliceMut, ToFrameSlice,
    ToFrameSliceMut, ToSampleSlice, ToSampleSliceMut,
};
#[cfg(feature = "boxed")]
use crate::boxed::{
    Box, FromBoxedFrameSlice, FromBoxedSampleSlice, ToBoxedFrameSlice, ToBoxedSampleSlice,
};
use dasp_frame::Frame;
use dasp_sample::Sample;

/// A macro for implementing all audio slice conversion traits for each fixed-size array.
macro_rules! impl_from_slice_conversions {
    ($($N:expr)*) => {
        $(
            impl<'a, S> FromSampleSlice<'a, S> for &'a [[S; $N]]
            where
                S: Sample,
                [S; $N]: Frame,
            {
                #[inline]
                fn from_sample_slice(slice: &'a [S]) -> Option<Self> {
                    let len = slice.len();
                    if len % $N == 0 {
                        let new_len = len / $N;
                        let ptr = slice.as_ptr() as *const _;
                        let new_slice = unsafe {
                            core::slice::from_raw_parts(ptr, new_len)
                        };
                        Some(new_slice)
                    } else {
                        None
                    }
                }
            }

            impl<'a, S> FromSampleSliceMut<'a, S> for &'a mut [[S; $N]]
            where
                S: Sample,
                [S; $N]: Frame,
            {
                #[inline]
                fn from_sample_slice_mut(slice: &'a mut [S]) -> Option<Self> {
                    let len = slice.len();
                    if len % $N == 0 {
                        let new_len = len / $N;
                        let ptr = slice.as_ptr() as *mut _;
                        let new_slice = unsafe {
                            core::slice::from_raw_parts_mut(ptr, new_len)
                        };
                        Some(new_slice)
                    } else {
                        None
                    }
                }
            }

            impl<'a, S> FromFrameSlice<'a, [S; $N]> for &'a [S]
            where
                [S; $N]: Frame,
            {
                #[inline]
                fn from_frame_slice(slice: &'a [[S; $N]]) -> Self {
                    let new_len = slice.len() * $N;
                    let ptr = slice.as_ptr() as *const _;
                    unsafe {
                        core::slice::from_raw_parts(ptr, new_len)
                    }
                }
            }

            impl<'a, S> FromFrameSliceMut<'a, [S; $N]> for &'a mut [S]
            where
                [S; $N]: Frame,
            {
                #[inline]
                fn from_frame_slice_mut(slice: &'a mut [[S; $N]]) -> Self {
                    let new_len = slice.len() * $N;
                    let ptr = slice.as_ptr() as *mut _;
                    unsafe {
                        core::slice::from_raw_parts_mut(ptr, new_len)
                    }
                }
            }

            impl<'a, S> ToSampleSlice<'a, S> for &'a [[S; $N]]
            where
                S: Sample,
            {
                #[inline]
                fn to_sample_slice(self) -> &'a [S] {
                    FromFrameSlice::from_frame_slice(self)
                }
            }

            impl<'a, S> ToSampleSliceMut<'a, S> for &'a mut [[S; $N]]
            where
                S: Sample,
            {
                #[inline]
                fn to_sample_slice_mut(self) -> &'a mut [S] {
                    FromFrameSliceMut::from_frame_slice_mut(self)
                }
            }

            impl<'a, S> ToFrameSlice<'a, [S; $N]> for &'a [S]
            where
                S: Sample,
                [S; $N]: Frame,
            {
                #[inline]
                fn to_frame_slice(self) -> Option<&'a [[S; $N]]> {
                    FromSampleSlice::from_sample_slice(self)
                }
            }

            impl<'a, S> ToFrameSliceMut<'a, [S; $N]> for &'a mut [S]
            where
                S: Sample,
                [S; $N]: Frame,
            {
                #[inline]
                fn to_frame_slice_mut(self) -> Option<&'a mut [[S; $N]]> {
                    FromSampleSliceMut::from_sample_slice_mut(self)
                }
            }

            #[cfg(feature = "boxed")]
            impl<S> FromBoxedSampleSlice<S> for Box<[[S; $N]]>
            where
                S: Sample,
                [S; $N]: Frame,
            {
                #[inline]
                fn from_boxed_sample_slice(mut slice: Box<[S]>) -> Option<Self> {
                    // First, we need a raw pointer to the slice and to make sure that the `Box` is
                    // forgotten so that our slice does not get deallocated.
                    let len = slice.len();
                    let slice_ptr = &mut slice as &mut [S] as *mut [S];
                    core::mem::forget(slice);
                    let sample_slice = unsafe {
                        core::slice::from_raw_parts_mut((*slice_ptr).as_mut_ptr(), len)
                    };

                    // Convert to our frame slice if possible.
                    let frame_slice = match <&mut [[S; $N]]>::from_sample_slice_mut(sample_slice) {
                        Some(slice) => slice,
                        None => return None,
                    };
                    let ptr = frame_slice as *mut [[S; $N]];

                    // Take ownership over the slice again before returning it.
                    let new_slice = unsafe {
                        Box::from_raw(ptr)
                    };

                    Some(new_slice)
                }
            }

            #[cfg(feature = "boxed")]
            impl<S> FromBoxedFrameSlice<[S; $N]> for Box<[S]>
            where
                [S; $N]: Frame,
            {
                #[inline]
                fn from_boxed_frame_slice(mut slice: Box<[[S; $N]]>) -> Self {
                    let new_len = slice.len() * $N;
                    let frame_slice_ptr = &mut slice as &mut [[S; $N]] as *mut [[S; $N]];
                    core::mem::forget(slice);
                    let sample_slice_ptr = frame_slice_ptr as *mut [S];
                    unsafe {
                        let ptr = (*sample_slice_ptr).as_mut_ptr();
                        let sample_slice = core::slice::from_raw_parts_mut(ptr, new_len);
                        Box::from_raw(sample_slice as *mut _)
                    }
                }
            }

            #[cfg(feature = "boxed")]
            impl<S> ToBoxedSampleSlice<S> for Box<[[S; $N]]>
            where
                S: Sample,
            {
                #[inline]
                fn to_boxed_sample_slice(self) -> Box<[S]> {
                    FromBoxedFrameSlice::from_boxed_frame_slice(self)
                }
            }

            #[cfg(feature = "boxed")]
            impl<S> ToBoxedFrameSlice<[S; $N]> for Box<[S]>
            where
                S: Sample,
                [S; $N]: Frame,
            {
                #[inline]
                fn to_boxed_frame_slice(self) -> Option<Box<[[S; $N]]>> {
                    FromBoxedSampleSlice::from_boxed_sample_slice(self)
                }
            }
        )*
    };
}

impl_from_slice_conversions! {
    1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32
}
