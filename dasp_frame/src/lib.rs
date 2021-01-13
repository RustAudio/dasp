//! Use the [**Frame**](./trait.Frame.html) trait to remain generic over the number of channels at
//! a single discrete moment in time.
//!
//! Implementations are provided for all fixed-size arrays up to 32 elements in length.

#![cfg_attr(not(feature = "std"), no_std)]

use core::iter::DoubleEndedIterator;

use dasp_sample::Sample;

/// Represents one sample from each channel at a single discrete instance in time within a
/// PCM signal.
///
/// Implementations are provided for:
///
/// - All fixed-size arrays up to a length of 32 elements.
/// - All primitive types that implement `Sample`. These implementations assume `CHANNELS = 1`.
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
    /// # Examples
    ///
    /// ```rust
    /// use dasp_frame::{Frame, Mono, Stereo};
    ///
    /// fn main() {
    ///     assert_eq!(Mono::<f32>::EQUILIBRIUM, [0.0]);
    ///     assert_eq!(Stereo::<f32>::EQUILIBRIUM, [0.0, 0.0]);
    ///     assert_eq!(<[f32; 3]>::EQUILIBRIUM, [0.0, 0.0, 0.0]);
    ///     assert_eq!(<[u8; 2]>::EQUILIBRIUM, [128u8, 128]);
    /// }
    /// ```
    const EQUILIBRIUM: Self;

    /// The total number of channels within the frame.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dasp_frame::{Frame, Mono, Stereo};
    ///
    /// fn main() {
    ///     assert_eq!(Mono::<f32>::CHANNELS, 1);
    ///     assert_eq!(Stereo::<f32>::CHANNELS, 2);
    ///     assert_eq!(<[f32; 3]>::CHANNELS, 3);
    ///     assert_eq!(<[u8; 2]>::CHANNELS, 2);
    /// }
    /// ```
    const CHANNELS: usize;

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

    /// Converts the frame into an iterator yielding the sample for each channel in the frame.
    fn channels(self) -> Self::Channels;

    /// Returns an iterator yielding references to the sample for each channel in the frame.
    fn channels_ref(&self) -> ChannelsRef<'_, Self>;

    /// Like [`channels_ref()`], but yields mutable references instead.
    ///
    /// # Example
    ///
    /// ```rust
    /// use dasp_frame::Frame;
    ///
    /// fn main() {
    ///     let mut foo = [1000i32, 2000, 3000];
    ///     let mut offset = 100i32;
    ///     for f in foo.channels_mut() {
    ///         *f = *f + offset;
    ///         offset += 100;
    ///     }
    ///     assert_eq!(foo, [1100i32, 2200, 3300]);
    /// }
    /// ```
    fn channels_mut(&mut self) -> ChannelsMut<'_, Self>;

    /// Yields a reference to the `Sample` of the channel at the given index if there is one.
    fn channel(&self, idx: usize) -> Option<&Self::Sample>;

    /// Like [`channel()`], but yields a mutable reference instead.
    fn channel_mut(&mut self, idx: usize) -> Option<&mut Self::Sample>;

    /// Returns a pointer to the sample of the channel at the given index, without doing bounds
    /// checking.
    ///
    /// Note: This is primarily a necessity for efficient `Frame::map` and `Frame::zip_map`
    /// methods, as for those methods we can guarantee lengths of different `Frame`s to be the same
    /// at *compile-time*.
    unsafe fn channel_unchecked(&self, idx: usize) -> &Self::Sample;

    /// Like [`channel_unchecked()`], but yields a mutable reference instead.
    unsafe fn channel_unchecked_mut(&mut self, idx: usize) -> &mut Self::Sample;

    /// Applies the given function to each sample in the `Frame` in channel order and returns the
    /// result as a new `Frame`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use dasp_frame::Frame;
    /// use dasp_sample::Sample;
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
    /// use dasp_frame::Frame;
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
    /// use dasp_frame::Frame;
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
    /// use dasp_frame::Frame;
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
    /// use dasp_frame::Frame;
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
    /// use dasp_frame::Frame;
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
    /// use dasp_frame::Frame;
    ///
    /// fn main() {
    ///     let foo = [0.25, 0.4].mul_amp([0.2, 0.5]);
    ///     assert_eq!(foo, [0.05, 0.2]);
    ///
    ///     let bar = [192u8, 64].mul_amp([0.0, -1.0]);
    ///     assert_eq!(bar, [128, 192]);
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

/// Restricts the types that may be used as the `Frame::NumChannels` associated type.
///
/// `NumChannels` allows us to enforce the number of channels that a `Frame` must have in certain
/// operations. This is particularly useful for `Frame::map` and `Frame::zip_map`, as it allows us
/// to guarantee that the input and output frame types will retain the same number of channels at
/// compile-time, and in turn removes the need for bounds checking.
///
/// This trait is implemented for types `N1`...`N32`.
pub trait NumChannels {}

pub type Mono<S> = [S; 1];
pub type Stereo<S> = [S; 2];

/// An iterator that yields the sample for each channel in the frame by value.
#[derive(Clone)]
pub struct Channels<F> {
    next_idx: usize,
    frame: F,
}

/// An iterator that yields the sample for each channel in the frame by reference.
#[derive(Clone)]
pub struct ChannelsRef<'a, F: Frame>(core::slice::Iter<'a, F::Sample>);

/// Like [`ChannelsRef`], but yields mutable references instead.
pub struct ChannelsMut<'a, F: Frame>(core::slice::IterMut<'a, F::Sample>);

macro_rules! impl_frame_for_fixed_size_array {
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

                const EQUILIBRIUM: Self = [S::EQUILIBRIUM; $N];
                const CHANNELS: usize = $N;

                #[inline]
                fn channels(self) -> Self::Channels {
                    Channels {
                        next_idx: 0,
                        frame: self,
                    }
                }

                #[inline]
                fn channels_ref(&self) -> ChannelsRef<'_, Self> {
                    ChannelsRef(self.iter())
                }

                #[inline]
                fn channels_mut(&mut self) -> ChannelsMut<'_, Self> {
                    ChannelsMut(self.iter_mut())
                }

                #[inline]
                fn channel(&self, idx: usize) -> Option<&Self::Sample> {
                    self.get(idx)
                }

                #[inline]
                fn channel_mut(&mut self, idx: usize) -> Option<&mut Self::Sample> {
                    self.get_mut(idx)
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

                #[inline(always)]
                unsafe fn channel_unchecked_mut(&mut self, idx: usize) -> &mut Self::Sample {
                    self.get_unchecked_mut(idx)
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

impl_frame_for_fixed_size_array! {
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

macro_rules! impl_frame_for_sample {
    ($($T:ty)*) => {
        $(
            impl Frame for $T {
                type Sample = $T;
                type NumChannels = N1;
                type Channels = Channels<Self>;
                type Float = <$T as Sample>::Float;
                type Signed = <$T as Sample>::Signed;

                const EQUILIBRIUM: Self = <$T as Sample>::EQUILIBRIUM;
                const CHANNELS: usize = 1;

                #[inline]
                fn channels(self) -> Self::Channels {
                    Channels {
                        next_idx: 0,
                        frame: self,
                    }
                }

                #[inline]
                fn channels_ref(&self) -> ChannelsRef<'_, Self> {
                    ChannelsRef(core::slice::from_ref(self).iter())
                }

                #[inline]
                fn channels_mut(&mut self) -> ChannelsMut<'_, Self> {
                    ChannelsMut(core::slice::from_mut(self).iter_mut())
                }

                #[inline]
                fn channel(&self, idx: usize) -> Option<&Self::Sample> {
                    if idx == 0 {
                        Some(self)
                    } else {
                        None
                    }
                }

                #[inline]
                fn channel_mut(&mut self, idx: usize) -> Option<&mut Self::Sample> {
                    if idx == 0 {
                        Some(self)
                    } else {
                        None
                    }
                }

                #[inline]
                fn from_fn<F>(mut from: F) -> Self
                where
                    F: FnMut(usize) -> Self::Sample,
                {
                    from(0)
                }

                #[inline]
                fn from_samples<I>(samples: &mut I) -> Option<Self>
                where
                    I: Iterator<Item=Self::Sample>
                {
                    samples.next()
                }

                #[inline(always)]
                unsafe fn channel_unchecked(&self, _idx: usize) -> &Self::Sample {
                    self
                }

                #[inline(always)]
                unsafe fn channel_unchecked_mut(&mut self, _idx: usize) -> &mut Self::Sample {
                    self
                }

                #[inline]
                fn to_signed_frame(self) -> Self::Signed {
                    self.to_signed_sample()
                }

                #[inline]
                fn to_float_frame(self) -> Self::Float {
                    self.to_float_sample()
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
                fn scale_amp(self, amp: <$T as Sample>::Float) -> Self {
                    Sample::mul_amp(self, amp)
                }

                #[inline]
                fn add_amp<F>(self, other: F) -> Self
                where
                    F: Frame<Sample=<$T as Sample>::Signed, NumChannels=N1>,
                {
                    // Here we do not require run-time bounds checking as we have asserted that the two
                    // arrays have the same number of channels at compile time with our where clause, i.e.
                    unsafe {
                        Sample::add_amp(self, *other.channel_unchecked(0))
                    }
                }
            }
        )*
    };
}

impl_frame_for_sample! {
    i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
}
impl_frame_for_sample! {
    dasp_sample::types::I24
    dasp_sample::types::I48
    dasp_sample::types::U24
    dasp_sample::types::U48
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
        F::CHANNELS - self.next_idx
    }
}

impl<'a, F: Frame> Iterator for ChannelsRef<'a, F> {
    type Item = &'a F::Sample;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, F: Frame> ExactSizeIterator for ChannelsRef<'a, F> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, F: Frame> DoubleEndedIterator for ChannelsRef<'a, F> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<'a, F: Frame> Iterator for ChannelsMut<'a, F> {
    type Item = &'a mut F::Sample;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, F: Frame> ExactSizeIterator for ChannelsMut<'a, F> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, F: Frame> DoubleEndedIterator for ChannelsMut<'a, F> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}
