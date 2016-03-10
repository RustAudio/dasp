use {Amplitude, Duplex, Sample};
use std;

/// Represents one sample from each channel at a single discrete instance in time within a
/// PCM signal.
///
/// We provide implementations for `Frame` for all fixed-size arrays up to a length of 32 elements.
pub trait Frame: Copy
    + Clone
    + std::fmt::Debug
    + PartialEq
{
    /// The type of PCM samples stored within the frame.
    type Sample: Sample;
    /// The number of channels in the `Frame`.
    type NumChannels: NumChannels;
    /// An iterator yielding the sample in each channel.
    type Channels: Iterator<Item=Self::Sample>;

    /// The equilibrium value for the wave that this `Sample` type represents. This is normally the
    /// value that is equal distance from both the min and max ranges of the sample.
    ///
    /// **NOTE:** This will likely be changed to an "associated const" if the feature lands.
    fn equilibrium() -> Self;

    /// Create a new `Self` where the `Sample` for each channel is produced by the given function.
    fn from_fn<F>(from: F) -> Self
        where F: FnMut() -> Self::Sample;

    /// The total number of channels (and in turn samples) stored within the frame.
    fn n_channels() -> usize;

    /// Converts the frame into an iterator yielding the sample for each channel in the frame.
    fn channels(self) -> Self::Channels;

    /// Returns a pointer to the sample of the channel at the given index, without doing bounds
    /// checking.
    ///
    /// Note: This is primarily a necessity for efficient `Frame::map` and `Frame::zip_map`
    /// methods, as for those methods we can guarantee lengths of different `Frame`s to be the same
    /// at *compile-time*.
    unsafe fn channel_unchecked(&self, idx: usize) -> &Self::Sample;

    /// Applies the given function to each sample in the `Frame` in channel order and returns the
    /// result as a new `Frame`.
    fn map<F, M>(self, map: M) -> F
        where F: Frame<NumChannels=Self::NumChannels>,
              M: FnMut(Self::Sample) -> F::Sample;

    /// Calls the given function with the pair of elements at every index and returns the
    /// resulting Frame.
    ///
    /// On a `Vec` this would be akin to `.into_iter().zip(other).map(|(a, b)| ...).collect()`, though
    /// much quicker and tailored to fixed-size arrays of samples.
    fn zip_map<O, F, M>(self, other: O, zip_map: M) -> F
        where O: Frame<NumChannels=Self::NumChannels>,
              F: Frame<NumChannels=Self::NumChannels>,
              M: FnMut(Self::Sample, O::Sample) -> F::Sample;

    /// Multiplies each `Sample` in the `Frame` by the given amplitude (either `f32` or `f64`) and
    /// returns the resulting `Frame`.
    ///
    /// - A > 1.0 amplifies the sample.
    /// - A < 1.0 attenuates the sample.
    /// - A == 1.0 yields the same sample.
    /// - A == 0.0 yields the `Sample::equilibrium`.
    #[inline]
    fn scale_amplitude<A>(self, amplitude: A) -> Self
        where Self::Sample: Duplex<A>,
              A: Amplitude,
    {
        self.map(|s| s.scale_amplitude(amplitude))
    }

    /// Sums `other` with `self` and returns the resulting `Frame`.
    #[inline]
    fn add(self, other: Self) -> Self {
        self.zip_map(other, std::ops::Add::add)
    }

}

pub type Mono<S> = [S; 1];
pub type Stereo<S> = [S; 2];

/// An iterator that yields the sample for each channel in the frame by value.
#[derive(Clone, Debug)]
pub struct Channels<S, F> {
    next_idx: usize,
    frame: F,
    sample: std::marker::PhantomData<S>,
}

/// A trait wrapper around `[S]::get` for generic use within the `Channels` `Iterator` impl.
trait GetChannel {
    type Sample: Sample;
    fn get_channel(&self, idx: usize) -> Option<Self::Sample>;
}

/// A trait to restrict types that may be used as the `Frame::NumChannels` assocaited type.
///
/// This trait is **only** implemented for fixed-size arrays with lengths from 1 to 32.
trait NumChannels {}

macro_rules! impl_get_channel {
    ($($N:expr)*) => {
        $(
            impl<S> GetChannel for [S; $N]
                where S: Sample,
            {
                type Sample = S;
                #[inline]
                fn get_channel(&self, idx: usize) -> Option<Self::Sample> {
                    self.get(idx).map(|&s| s)
                }
            }
        )*
    };
}

impl_get_channel! {
    1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32
}

macro_rules! impl_frame {
    ($($NChan:ident $N:expr, [$($idx:expr)*],)*) => {
        $(
            /// A typified version of the number of channels, used for safely mapping frames of the
            /// same length to another `Frame` with a different `Sample` associated type.
            pub struct $NChan;
            impl NumChannels for $NChan {}

            impl<S> Frame for [S; $N]
                where S: Sample,
            {
                type Sample = S;
                type NumChannels = $NChan;
                type Channels = Channels<S, Self>;

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
                    Channels::new(self)
                }

                #[inline]
                fn from_fn<F>(mut from: F) -> Self
                    where F: FnMut() -> S,
                {
                    [$( { $idx; from() }, )*]
                }

                #[inline(always)]
                unsafe fn channel_unchecked(&self, idx: usize) -> &Self::Sample {
                    self.get_unchecked(idx)
                }

                #[inline]
                fn map<F, M>(self, mut map: M) -> F
                    where F: Frame<NumChannels=Self::NumChannels>,
                          M: FnMut(Self::Sample) -> F::Sample,
                {
                    let mut channel_idx = 0;
                    F::from_fn(|| {

                        // Here we do not require run-time bounds checking as we have asserted that
                        // the two arrays have the same number of channels at compile time with our
                        // where clause, i.e.
                        //
                        // `F: Frame<NumChannels=Self::NumChannels>`
                        let sample = unsafe { map(*self.channel_unchecked(channel_idx)) };

                        channel_idx += 1;
                        sample
                    })
                }

                #[inline]
                fn zip_map<O, F, M>(self, other: O, mut zip_map: M) -> F
                    where O: Frame<NumChannels=Self::NumChannels>,
                          F: Frame<NumChannels=Self::NumChannels>,
                          M: FnMut(Self::Sample, O::Sample) -> F::Sample
                {
                    let mut channel_idx = 0;
                    F::from_fn(|| {

                        // Here we do not require run-time bounds checking as we have asserted that the two
                        // arrays have the same number of channels at compile time with our where clause, i.e.
                        //
                        // ```
                        // O: Frame<NumChannels=Self::NumChannels>
                        // F: Frame<NumChannels=Self::NumChannels>
                        // ```
                        let sample = unsafe {
                            zip_map(*self.channel_unchecked(channel_idx),
                                    *other.channel_unchecked(channel_idx))
                        };

                        channel_idx += 1;
                        sample
                    })
                }

                #[inline]
                fn add(self, other: Self) -> Self {
                    [$(self[$idx] + other[$idx], )*]
                }

                #[inline]
                fn scale_amplitude<A>(self, amplitude: A) -> Self
                    where Self::Sample: Duplex<A>,
                          A: Amplitude,
                {
                    [$(
                        (self[$idx].to_sample::<A>() * amplitude).to_sample(),
                    )*]
                }
            }
        )*
    };
}

impl_frame!{
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


impl<S, F> Channels<S, F> {
    #[inline]
    fn new(frame: F) -> Self {
        Channels {
            next_idx: 0,
            frame: frame,
            sample: std::marker::PhantomData,
        }
    }
}

impl<S, F> Iterator for Channels<S, F>
    where S: Sample,
          F: GetChannel<Sample=S>,
{
    type Item = S;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.frame.get_channel(self.next_idx).map(|s| {
            self.next_idx += 1;
            s
        })
    }
}
