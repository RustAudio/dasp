//! Use the Frame trait to remain generic over the number of channels at
//! a single discrete moment in time.
//!
//! Implementations are provided for all fixed-size arrays up to 32 elements in length.

use conv::{
    DuplexBoxedFrameSlice, DuplexBoxedSampleSlice, DuplexBoxedSlice, DuplexFrameSlice,
    DuplexFrameSliceMut, DuplexSampleSlice, DuplexSampleSliceMut, DuplexSlice, DuplexSliceMut,
    FromBoxedFrameSlice, FromBoxedSampleSlice, FromFrameSlice, FromFrameSliceMut, FromSampleSlice,
    FromSampleSliceMut, ToBoxedFrameSlice, ToBoxedSampleSlice, ToFrameSlice, ToFrameSliceMut,
    ToSampleSlice, ToSampleSliceMut,
};
use Sample;

pub type Mono<S> = [S; 1];
pub type Stereo<S> = [S; 2];

/// Represents one sample from each channel at a single discrete instance in time within a
/// PCM signal.
///
/// We provide implementations for `Frame` for all fixed-size arrays up to a length of 32 elements.
pub trait Frame: Copy + Clone + PartialEq {
    /// The type of PCM sample stored at each channel within the frame.
    type Sample: Sample;
    /// A typified version of a number of channels in the `Frame`, used for safely mapping frames
    /// of the same length to other `Frame`s, perhaps with a different `Sample` associated type.
    type NumChannels: NumChannels;
    /// An iterator yielding the sample in each channel, starting from left (channel 0) and ending
    /// at the right (channel NumChannels-1).
    type Channels: Iterator<Item = Self::Sample>;
    /// A frame type with equilavent number of channels using the associated `Sample::Signed` format.
    type Signed: Frame<Sample = <Self::Sample as Sample>::Signed, NumChannels = Self::NumChannels>;
    /// A frame type with equilavent number of channels using the associated `Sample::Float` format.
    type Float: Frame<Sample = <Self::Sample as Sample>::Float, NumChannels = Self::NumChannels>;

    /// The equilibrium value for the wave that this `Sample` type represents. This is normally the
    /// value that is equal distance from both the min and max ranges of the sample.
    ///
    /// **NOTE:** This will likely be changed to an "associated const" if the feature lands.
    ///
    /// # Examples
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Frame;
    /// use sample::frame::{Mono, Stereo};
    ///
    /// fn main() {
    ///     assert_eq!(Mono::<f32>::equilibrium(), [0.0]);
    ///     assert_eq!(Stereo::<f32>::equilibrium(), [0.0, 0.0]);
    ///     assert_eq!(<[f32; 3]>::equilibrium(), [0.0, 0.0, 0.0]);
    ///     assert_eq!(<[u8; 2]>::equilibrium(), [128u8, 128]);
    /// }
    /// ```
    fn equilibrium() -> Self;

    /// Create a new `Frame` where the `Sample` for each channel is produced by the given function.
    ///
    /// The given function should map each channel index to its respective sample.
    fn from_fn<F>(from: F) -> Self
    where
        F: FnMut(usize) -> Self::Sample;

    /// Create a new `Frame` from a borrowed `Iterator` yielding samples for each channel.
    ///
    /// Returns `None` if the given `Iterator` does not yield enough `Sample`s.
    ///
    /// This is necessary for the `signal::FromSamples` `Iterator`, that converts some `Iterator`
    /// yielding `Sample`s to an `Iterator` yielding `Frame`s.
    fn from_samples<I>(samples: &mut I) -> Option<Self>
    where
        I: Iterator<Item = Self::Sample>;

    /// The total number of channels (and in turn samples) stored within the frame.
    fn n_channels() -> usize;

    /// Converts the frame into an iterator yielding the sample for each channel in the frame.
    fn channels(self) -> Self::Channels;

    /// Yields a reference to the `Sample` of the channel at the given index if there is one.
    fn channel(&self, idx: usize) -> Option<&Self::Sample>;

    /// Returns a pointer to the sample of the channel at the given index, without doing bounds
    /// checking.
    ///
    /// Note: This is primarily a necessity for efficient `Frame::map` and `Frame::zip_map`
    /// methods, as for those methods we can guarantee lengths of different `Frame`s to be the same
    /// at *compile-time*.
    unsafe fn channel_unchecked(&self, idx: usize) -> &Self::Sample;

    /// Applies the given function to each sample in the `Frame` in channel order and returns the
    /// result as a new `Frame`.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::{Frame, Sample};
    ///
    /// fn main() {
    ///     let foo = [0i16, 0];
    ///     let bar: [u8; 2] = foo.map(Sample::to_sample);
    ///     assert_eq!(bar, [128u8, 128]);
    /// }
    /// ```
    fn map<F, M>(self, map: M) -> F
    where
        F: Frame<NumChannels = Self::NumChannels>,
        M: FnMut(Self::Sample) -> F::Sample;

    /// Calls the given function with the pair of elements at every index and returns the
    /// resulting Frame.
    ///
    /// On a `Vec` this would be akin to `.into_iter().zip(other).map(|(a, b)| ...).collect()`, though
    /// much quicker and tailored to fixed-size arrays of samples.
    fn zip_map<O, F, M>(self, other: O, zip_map: M) -> F
    where
        O: Frame<NumChannels = Self::NumChannels>,
        F: Frame<NumChannels = Self::NumChannels>,
        M: FnMut(Self::Sample, O::Sample) -> F::Sample;

    /// Converts the frame type to the equivalent signal in its associated `Float`ing point format.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Frame;
    ///
    /// fn main() {
    ///     let foo = [128u8; 2];
    ///     let signed = foo.to_signed_frame();
    ///     assert_eq!(signed, [0i8; 2]);
    /// }
    /// ```
    fn to_signed_frame(self) -> Self::Signed;

    /// Converts the frame type to the equivalent signal in its associated `Signed` format.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Frame;
    ///
    /// fn main() {
    ///     let foo = [128u8; 2];
    ///     let float = foo.to_float_frame();
    ///     assert_eq!(float, [0.0; 2]);
    /// }
    /// ```
    fn to_float_frame(self) -> Self::Float;

    /// Offsets the amplitude of every channel in the frame by the given `offset` and yields the
    /// resulting frame.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Frame;
    ///
    /// fn main() {
    ///     assert_eq!([0.25, -0.5].offset_amp(0.5), [0.75, 0.0]);
    ///     assert_eq!([0.5, -0.25].offset_amp(-0.25), [0.25, -0.5]);
    ///     assert_eq!([128u8, 192].offset_amp(-64), [64, 128]);
    /// }
    /// ```
    #[inline]
    fn offset_amp(self, offset: <Self::Sample as Sample>::Signed) -> Self {
        self.map(|s| s.add_amp(offset))
    }

    /// Multiplies each `Sample` in the `Frame` by the given amplitude and returns the resulting
    /// `Frame`.
    ///
    /// - A > 1.0 amplifies the sample.
    /// - A < 1.0 attenuates the sample.
    /// - A == 1.0 yields the same sample.
    /// - A == 0.0 yields the `Sample::equilibrium`.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Frame;
    ///
    /// fn main() {
    ///     assert_eq!([0.1, 0.2, -0.1, -0.2].scale_amp(2.0), [0.2, 0.4, -0.2, -0.4]);
    /// }
    /// ```
    #[inline]
    fn scale_amp(self, amp: <Self::Sample as Sample>::Float) -> Self {
        self.map(|s| s.mul_amp(amp))
    }

    /// Sums each channel in `other` with each channel in `self` and returns the resulting `Frame`.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Frame;
    ///
    /// fn main() {
    ///     let foo = [0.25, 0.5].add_amp([-0.75, 0.25]);
    ///     assert_eq!(foo, [-0.5, 0.75]);
    /// }
    /// ```
    #[inline]
    fn add_amp<F>(self, other: F) -> Self
    where
        F: Frame<Sample = <Self::Sample as Sample>::Signed, NumChannels = Self::NumChannels>,
    {
        self.zip_map(other, Sample::add_amp)
    }

    /// Multiplies `other` with `self` and returns the resulting `Frame`.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Frame;
    ///
    /// fn main() {
    ///     let foo = [0.25, 0.4].mul_amp([0.2, 0.5]);
    ///     assert_eq!(foo, [0.05, 0.2]);
    ///
    ///     let bar = [192u8, 64].mul_amp([0.0, -2.0]);
    ///     assert_eq!(bar, [128, 0]);
    /// }
    /// ```
    #[inline]
    fn mul_amp<F>(self, other: F) -> Self
    where
        F: Frame<Sample = <Self::Sample as Sample>::Float, NumChannels = Self::NumChannels>,
    {
        self.zip_map(other, Sample::mul_amp)
    }
}

/// An iterator that yields the sample for each channel in the frame by value.
#[derive(Clone)]
pub struct Channels<F> {
    next_idx: usize,
    frame: F,
}

/// Restricts the types that may be used as the `Frame::NumChannels` associated type.
///
/// `NumChannels` allows us to enforce the number of channels that a `Frame` must have in certain
/// operations. This is particularly useful for `Frame::map` and `Frame::zip_map`, as it allows us
/// to guarantee that the input and output frame types will retain the same number of channels at
/// compile-time, and in turn removes the need for bounds checking.
///
/// This trait is implemented for types `N1`...`N32`.
pub trait NumChannels {}

macro_rules! impl_frame {
    ($($NChan:ident $N:expr, [$($idx:expr)*],)*) => {
        $(
            /// A typified version of a number of channels.
            pub struct $NChan;
            impl NumChannels for $NChan {}

            impl<S> Frame for [S; $N]
            where
                S: Sample,
            {
                type Sample = S;
                type NumChannels = $NChan;
                type Channels = Channels<Self>;
                type Float = [S::Float; $N];
                type Signed = [S::Signed; $N];

                #[inline]
                fn equilibrium() -> Self {
                    [S::equilibrium(); $N]
                }

                #[inline]
                fn n_channels() -> usize {
                    $N
                }

                #[inline]
                fn channels(self) -> Self::Channels {
                    Channels {
                        next_idx: 0,
                        frame: self,
                    }
                }

                #[inline]
                fn channel(&self, idx: usize) -> Option<&Self::Sample> {
                    self.get(idx)
                }

                #[inline]
                fn from_fn<F>(mut from: F) -> Self
                where
                    F: FnMut(usize) -> S,
                {
                    [$(from($idx), )*]
                }

                #[inline]
                fn from_samples<I>(samples: &mut I) -> Option<Self>
                where
                    I: Iterator<Item=Self::Sample>
                {
                    Some([$( {
                        $idx;
                        match samples.next() {
                            Some(sample) => sample,
                            None => return None,
                        }
                    }, )*])
                }

                #[inline(always)]
                unsafe fn channel_unchecked(&self, idx: usize) -> &Self::Sample {
                    self.get_unchecked(idx)
                }

                #[inline]
                fn to_signed_frame(self) -> Self::Signed {
                    self.map(|s| s.to_sample())
                }

                #[inline]
                fn to_float_frame(self) -> Self::Float {
                    self.map(|s| s.to_sample())
                }

                #[inline]
                fn map<F, M>(self, mut map: M) -> F
                where
                    F: Frame<NumChannels=Self::NumChannels>,
                    M: FnMut(Self::Sample) -> F::Sample,
                {
                    F::from_fn(|channel_idx| {

                        // Here we do not require run-time bounds checking as we have asserted that
                        // the two arrays have the same number of channels at compile time with our
                        // where clause, i.e.
                        //
                        // `F: Frame<NumChannels=Self::NumChannels>`
                        unsafe { map(*self.channel_unchecked(channel_idx)) }
                    })
                }

                #[inline]
                fn zip_map<O, F, M>(self, other: O, mut zip_map: M) -> F
                where
                    O: Frame<NumChannels=Self::NumChannels>,
                    F: Frame<NumChannels=Self::NumChannels>,
                    M: FnMut(Self::Sample, O::Sample) -> F::Sample
                {
                    F::from_fn(|channel_idx| {

                        // Here we do not require run-time bounds checking as we have asserted that the two
                        // arrays have the same number of channels at compile time with our where clause, i.e.
                        //
                        // ```
                        // O: Frame<NumChannels=Self::NumChannels>
                        // F: Frame<NumChannels=Self::NumChannels>
                        // ```
                        unsafe {
                            zip_map(*self.channel_unchecked(channel_idx),
                                    *other.channel_unchecked(channel_idx))
                        }
                    })
                }

                #[inline]
                fn scale_amp(self, amp: S::Float) -> Self {
                    [$(self[$idx].mul_amp(amp), )*]
                }

                #[inline]
                fn add_amp<F>(self, other: F) -> Self
                where
                    F: Frame<Sample=S::Signed, NumChannels=$NChan>,
                {
                    // Here we do not require run-time bounds checking as we have asserted that the two
                    // arrays have the same number of channels at compile time with our where clause, i.e.
                    unsafe {
                        [$(self[$idx].add_amp(*other.channel_unchecked($idx)), )*]
                    }
                }

            }
        )*
    };
}

impl_frame! {
    N1  1,  [0],
    N2  2,  [0 1],
    N3  3,  [0 1 2],
    N4  4,  [0 1 2 3],
    N5  5,  [0 1 2 3 4],
    N6  6,  [0 1 2 3 4 5],
    N7  7,  [0 1 2 3 4 5 6],
    N8  8,  [0 1 2 3 4 5 6 7],
    N9  9,  [0 1 2 3 4 5 6 7 8],
    N10 10, [0 1 2 3 4 5 6 7 8 9],
    N11 11, [0 1 2 3 4 5 6 7 8 9 10],
    N12 12, [0 1 2 3 4 5 6 7 8 9 10 11],
    N13 13, [0 1 2 3 4 5 6 7 8 9 10 11 12],
    N14 14, [0 1 2 3 4 5 6 7 8 9 10 11 12 13],
    N15 15, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14],
    N16 16, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15],
    N17 17, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16],
    N18 18, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17],
    N19 19, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18],
    N20 20, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19],
    N21 21, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20],
    N22 22, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21],
    N23 23, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22],
    N24 24, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23],
    N25 25, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24],
    N26 26, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25],
    N27 27, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26],
    N28 28, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27],
    N29 29, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28],
    N30 30, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29],
    N31 31, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30],
    N32 32, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31],
}

impl<F> Iterator for Channels<F>
where
    F: Frame,
{
    type Item = F::Sample;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.frame.channel(self.next_idx).map(|&s| s).map(|s| {
            self.next_idx += 1;
            s
        })
    }
}

impl<F> ExactSizeIterator for Channels<F>
where
    F: Frame,
{
    #[inline]
    fn len(&self) -> usize {
        F::n_channels() - self.next_idx
    }
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

impl<F> FromBoxedFrameSlice<F> for Box<[F]>
where
    F: Frame,
{
    #[inline]
    fn from_boxed_frame_slice(slice: Box<[F]>) -> Self {
        slice
    }
}

/// A macro for implementing all audio slice conversion traits for each fixed-size array.
macro_rules! impl_from_slice_conversions {
    ($($N:expr)*) => {
        $(

            impl<'a, S> FromSampleSlice<'a, S> for &'a [[S; $N]]
                where S: Sample,
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
                where S: Sample,
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

            impl<S> FromBoxedSampleSlice<S> for Box<[[S; $N]]>
                where S: Sample,
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

            impl<'a, S> FromFrameSlice<'a, [S; $N]> for &'a [S]
                where [S; $N]: Frame,
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
                where [S; $N]: Frame,
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

            impl<S> FromBoxedFrameSlice<[S; $N]> for Box<[S]>
                where [S; $N]: Frame,
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

            impl<'a, S> ToSampleSlice<'a, S> for &'a [[S; $N]]
                where S: Sample,
            {
                #[inline]
                fn to_sample_slice(self) -> &'a [S] {
                    FromFrameSlice::from_frame_slice(self)
                }
            }

            impl<'a, S> ToSampleSliceMut<'a, S> for &'a mut [[S; $N]]
                where S: Sample,
            {
                #[inline]
                fn to_sample_slice_mut(self) -> &'a mut [S] {
                    FromFrameSliceMut::from_frame_slice_mut(self)
                }
            }

            impl<S> ToBoxedSampleSlice<S> for Box<[[S; $N]]>
                where S: Sample,
            {
                #[inline]
                fn to_boxed_sample_slice(self) -> Box<[S]> {
                    FromBoxedFrameSlice::from_boxed_frame_slice(self)
                }
            }

            impl<'a, S> ToFrameSlice<'a, [S; $N]> for &'a [S]
                where S: Sample,
                      [S; $N]: Frame,
            {
                #[inline]
                fn to_frame_slice(self) -> Option<&'a [[S; $N]]> {
                    FromSampleSlice::from_sample_slice(self)
                }
            }

            impl<'a, S> ToFrameSliceMut<'a, [S; $N]> for &'a mut [S]
                where S: Sample,
                      [S; $N]: Frame,
            {
                #[inline]
                fn to_frame_slice_mut(self) -> Option<&'a mut [[S; $N]]> {
                    FromSampleSliceMut::from_sample_slice_mut(self)
                }
            }

            impl<S> ToBoxedFrameSlice<[S; $N]> for Box<[S]>
                where S: Sample,
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

impl<'a, F, T> DuplexFrameSlice<'a, F> for T
where
    F: Frame,
    T: FromFrameSlice<'a, F> + ToFrameSlice<'a, F>,
{
}

impl<'a, S, F, T> DuplexSlice<'a, S, F> for T
where
    S: Sample,
    F: Frame<Sample = S>,
    T: DuplexSampleSlice<'a, S> + DuplexFrameSlice<'a, F>,
{
}

impl<'a, F, T> DuplexFrameSliceMut<'a, F> for T
where
    F: Frame,
    T: FromFrameSliceMut<'a, F> + ToFrameSliceMut<'a, F>,
{
}

impl<'a, S, F, T> DuplexSliceMut<'a, S, F> for T
where
    S: Sample,
    F: Frame<Sample = S>,
    T: DuplexSampleSliceMut<'a, S> + DuplexFrameSliceMut<'a, F>,
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
