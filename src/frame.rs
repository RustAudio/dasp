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
    /// An iterator yielding the sample in each channel.
    type Channels: Iterator<Item=Self::Sample>;

    /// The equilibrium value for the wave that this `Sample` type represents. This is normally the
    /// value that is equal distance from both the min and max ranges of the sample.
    ///
    /// **NOTE:** This will likely be changed to an "associated const" if the feature lands.
    fn equilibrium() -> Self;

    /// The total number of channels (and in turn samples) stored within the frame.
    fn n_channels() -> usize;

    /// Converts the frame into an iterator yielding the sample for each channel in the frame.
    fn channels(self) -> Self::Channels;

    /// Applies the given function to each sample in the `Frame` in channel order and returns the
    /// result as a new `Frame`.
    fn map<F>(self, map: F) -> Self
        where F: FnMut(Self::Sample) -> Self::Sample;

    /// Calls the given function with the pair of elements at every index and returns the
    /// resulting Frame.
    ///
    /// On a `Vec` this would be akin to `.into_iter().zip(other).map(|(a, b)| ...).collect()`, though
    /// much quicker and tailored to fixed-size arrays of samples.
    fn zip_map<F>(self, other: Self, zip_map: F) -> Self
        where F: FnMut(Self::Sample, Self::Sample) -> Self::Sample;

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


trait IndexChannel {
    type Sample: Sample;
    fn index_channel(&self, idx: usize) -> Option<Self::Sample>;
}

macro_rules! impl_index_channel {
    ($($N:expr)*) => {
        $(
            impl<S> IndexChannel for [S; $N]
                where S: Sample,
            {
                type Sample = S;
                #[inline]
                fn index_channel(&self, idx: usize) -> Option<Self::Sample> {
                    self.get(idx).map(|&s| s)
                }
            }
        )*
    };
}

impl_index_channel!{
    1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32
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

macro_rules! impl_frame {
    ($($N:expr, [$($idx:expr)*],)*) => {
        $(
            impl<S> Frame for [S; $N]
                where S: Sample,
            {
                type Sample = S;
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
                fn map<F>(self, mut map: F) -> Self
                    where F: FnMut(S) -> S,
                {
                    [$(map(self[$idx]), )*]
                }

                #[inline]
                fn zip_map<F>(self, other: Self, mut zip_map: F) -> Self
                    where F: FnMut(S, S) -> S,
                {
                    [$(zip_map(self[$idx], other[$idx]), )*]
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
    1,  [0],
    2,  [0 1],
    3,  [0 1 2],
    4,  [0 1 2 3],
    5,  [0 1 2 3 4],
    6,  [0 1 2 3 4 5],
    7,  [0 1 2 3 4 5 6],
    8,  [0 1 2 3 4 5 6 7],
    9,  [0 1 2 3 4 5 6 7 8],
    10, [0 1 2 3 4 5 6 7 8 9],
    11, [0 1 2 3 4 5 6 7 8 9 10],
    12, [0 1 2 3 4 5 6 7 8 9 10 11],
    13, [0 1 2 3 4 5 6 7 8 9 10 11 12],
    14, [0 1 2 3 4 5 6 7 8 9 10 11 12 13],
    15, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14],
    16, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15],
    17, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16],
    18, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17],
    19, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18],
    20, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19],
    21, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20],
    22, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21],
    23, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22],
    24, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23],
    25, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24],
    26, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25],
    27, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26],
    28, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27],
    29, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28],
    30, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29],
    31, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30],
    32, [0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31],
}


impl<S, F> Iterator for Channels<S, F>
    where S: Sample,
          F: IndexChannel<Sample=S>,
{
    type Item = S;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.frame.index_channel(self.next_idx).map(|s| {
            self.next_idx += 1;
            s
        })
    }
}
