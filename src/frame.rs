use Sample;
use std;

pub type Mono<S> = [S; 1];
pub type Stereo<S> = [S; 2];

/// An iterator that yields the sample for each channel in the frame by value.
#[derive(Clone, Debug)]
pub struct Channels<S, F> {
    next_idx: usize,
    frame: F,
    sample: std::marker::PhantomData<S>,
}

/// Represents a single discrete instance in time within a pulse-code modulation DSP signal.
pub trait Frame {
    /// The type of DSP sample stored within the frame.
    type Sample: Sample;
    /// An iterator yielding the sample in each channel.
    type Channels: Iterator<Item=Self::Sample>;

    /// The total number of channels (and in turn samples) stored within the frame.
    fn num_channels() -> usize;

    /// Converts the frame into an iterator yielding the sample for each channel in the frame.
    fn channels(self) -> Self::Channels;
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

impl<S> Frame for Mono<S>
    where S: Sample,
{
    type Sample = S;
    type Channels = std::iter::Once<S>;

    #[inline]
    fn num_channels() -> usize {
        1
    }

    #[inline]
    fn channels(self) -> Self::Channels {
        std::iter::once(self[0])
    }
}

macro_rules! impl_frame {
    ($($N:expr)*) => {
        $(
            impl<S> Frame for [S; $N]
                where S: Sample,
            {
                type Sample = S;
                type Channels = Channels<S, Self>;

                #[inline]
                fn num_channels() -> usize {
                    $N
                }

                #[inline]
                fn channels(self) -> Self::Channels {
                    Channels::new(self)
                }
            }
        )*
    };
}

impl_frame!{
    2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32
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
