use {Frame, Sample};
use std;


/// Implement `Signal` for all `Iterator`s that yield `Frame`s.
impl<I> Signal for I where I: Iterator, I::Item: Frame {}

/// A trait that allows us to treat `Iterator`s that yield `Frame`s as a multi-channel PCM signal.
///
/// For example, `Signal` allows us to add two signals, modulate a signal's amplitude by another
/// signal, scale a signals amplitude and much more.
///
/// `Signal` has a blanked implementation for all `Iterator`s whose `Item` associated types
/// implement `Frame`.
pub trait Signal: Iterator + Sized
    where Self::Item: Frame,
{

    /// Provides an iterator that yields the sum of the frames yielded by both `other` and `self`
    /// in lock-step.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Signal;
    ///
    /// fn main() {
    ///     let a = [[0.2], [-0.6], [0.5]];
    ///     let b = [[0.2], [0.1], [-0.8]];
    ///     let a_signal = a.iter().cloned();
    ///     let b_signal = b.iter().cloned();
    ///     let added: Vec<[f32; 1]> = a_signal.add_amp(b_signal).collect();
    ///     assert_eq!(added, vec![[0.4], [-0.5], [-0.3]]);
    /// }
    /// ```
    #[inline]
    fn add_amp<S>(self, other: S) -> AddAmp<Self, S>
        where S: Signal,
              S::Item: Frame<Sample=<<Self::Item as Frame>::Sample as Sample>::Signed,
                             NumChannels=<Self::Item as Frame>::NumChannels>,
    {
        AddAmp {
            a: self,
            b: other,
        }
    }

    /// Provides an iterator that yields the product of the frames yielded by both `other` and
    /// `self` in lock-step.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Signal;
    ///
    /// fn main() {
    ///     let a = [[0.25], [-0.8], [-0.5]];
    ///     let b = [[0.2], [0.5], [0.8]];
    ///     let a_signal = a.iter().cloned();
    ///     let b_signal = b.iter().cloned();
    ///     let added: Vec<[f32; 1]> = a_signal.mul_amp(b_signal).collect();
    ///     assert_eq!(added, vec![[0.05], [-0.4], [-0.4]]);
    /// }
    /// ```
    #[inline]
    fn mul_amp<S>(self, other: S) -> MulAmp<Self, S>
        where S: Signal,
              S::Item: Frame<Sample=<<Self::Item as Frame>::Sample as Sample>::Float,
                             NumChannels=<Self::Item as Frame>::NumChannels>,
    {
        MulAmp {
            a: self,
            b: other,
        }
    }

    /// Provides an iterator that offsets the amplitude of every channel in each frame of the
    /// signal by some sample value and yields the resulting frames.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Signal;
    ///
    /// fn main() {
    ///     let frames = [[0.25, 0.4], [-0.2, -0.5]];
    ///     let signal = frames.iter().cloned();
    ///     let offset: Vec<[f32; 2]> = signal.offset_amp(0.5).collect();
    ///     assert_eq!(offset, vec![[0.75, 0.9], [0.3, 0.0]]);
    /// }
    /// ```
    #[inline]
    fn offset_amp(self, offset: <<Self::Item as Frame>::Sample as Sample>::Signed)
        -> OffsetAmp<Self>
    {
        OffsetAmp {
            signal: self,
            offset: offset,
        }
    }

    /// Produces an `Iterator` that scales the amplitude of the sample of each channel in every
    /// `Frame` yielded by `self` by the given amplitude.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Signal;
    ///
    /// fn main() {
    ///     let frames = [[0.2], [-0.5], [-0.4], [0.3]];
    ///     let signal = frames.iter().cloned();
    ///     let scaled: Vec<[f32; 1]> = signal.scale_amp(2.0).collect();
    ///     assert_eq!(scaled, vec![[0.4], [-1.0], [-0.8], [0.6]]);
    /// }
    /// ```
    #[inline]
    fn scale_amp(self, amp: <<Self::Item as Frame>::Sample as Sample>::Float) -> ScaleAmp<Self> {
        ScaleAmp {
            signal: self,
            amp: amp,
        }
    }

    /// Produces an `Iterator` that offsets the amplitude of every `Frame` in `self` by the
    /// respective amplitudes in each channel of the given `amp_frame`.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Signal;
    ///
    /// fn main() {
    ///     let frames = [[0.5, 0.3], [-0.25, 0.9]];
    ///     let mut signal = frames.iter().cloned().offset_amp_per_channel([0.25, -0.5]);
    ///     assert_eq!(signal.next().unwrap(), [0.75, -0.2]);
    ///     assert_eq!(signal.next().unwrap(), [0.0, 0.4]);
    /// }
    /// ```
    #[inline]
    fn offset_amp_per_channel<F>(self, amp_frame: F) -> OffsetAmpPerChannel<Self, F>
        where F: Frame<Sample=<<Self::Item as Frame>::Sample as Sample>::Signed,
                       NumChannels=<Self::Item as Frame>::NumChannels>,
    {
        OffsetAmpPerChannel {
            signal: self,
            amp_frame: amp_frame,
        }
    }

    /// Produces an `Iterator` that scales the amplitude of every `Frame` in `self` by the
    /// respective amplitudes in each channel of the given `amp_frame`.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Signal;
    ///
    /// fn main() {
    ///     let frames = [[0.2, -0.5], [-0.4, 0.3]];
    ///     let mut signal = frames.iter().cloned().scale_amp_per_channel([0.5, 2.0]);
    ///     assert_eq!(signal.next().unwrap(), [0.1, -1.0]);
    ///     assert_eq!(signal.next().unwrap(), [-0.2, 0.6]);
    /// }
    /// ```
    #[inline]
    fn scale_amp_per_channel<F>(self, amp_frame: F) -> ScaleAmpPerChannel<Self, F>
        where F: Frame<Sample=<<Self::Item as Frame>::Sample as Sample>::Float,
                       NumChannels=<Self::Item as Frame>::NumChannels>,
    {
        ScaleAmpPerChannel {
            signal: self,
            amp_frame: amp_frame,
        }
    }

}


/// An iterator that endlessly yields `Frame`s of type `F` at equilibrium.
#[derive(Clone)]
pub struct Equilibrium<F> {
    frame_type: std::marker::PhantomData<F>,
}

/// An iterator that yields the sum of the frames yielded by both `other` and `self` in lock-step.
#[derive(Clone)]
pub struct AddAmp<A, B> {
    a: A,
    b: B,
}

/// An iterator that yields the product of the frames yielded by both `other` and `self` in
/// lock-step.
#[derive(Clone)]
pub struct MulAmp<A, B> {
    a: A,
    b: B,
}

/// Provides an iterator that offsets the amplitude of every channel in each frame of the
/// signal by some sample value and yields the resulting frames.
#[derive(Clone)]
pub struct OffsetAmp<S>
    where S: Signal,
          S::Item: Frame,
{
    signal: S,
    offset: <<S::Item as Frame>::Sample as Sample>::Signed,
}

/// An `Iterator` that scales the amplitude of the sample of each channel in every `Frame` yielded
/// by `self` by the given amplitude.
#[derive(Clone)]
pub struct ScaleAmp<S>
    where S: Signal,
          S::Item: Frame,
{
    signal: S,
    amp: <<S::Item as Frame>::Sample as Sample>::Float,
}

/// An `Iterator` that scales the amplitude of every `Frame` in `self` by the respective amplitudes
/// in each channel of the given `amp` `Frame`.
#[derive(Clone)]
pub struct OffsetAmpPerChannel<S, F> {
    signal: S,
    amp_frame: F,
}

/// An `Iterator` that scales the amplitude of every `Frame` in `self` by the respective amplitudes
/// in each channel of the given `amp` `Frame`.
#[derive(Clone)]
pub struct ScaleAmpPerChannel<S, F> {
    signal: S,
    amp_frame: F,
}

/// An iterator that takes two `Signal`s, steps them forward in lockstep, sums each pair of
/// `Frame`s together and yields the resulting `Frame`s.
#[derive(Clone)]
pub struct ZipAdd<A, B> {
    signal_a: A,
    signal_b: B,
}

/// An `Iterator` that modulates the amplitude of `self` with `other`.
///
/// The `Iterator` steps them forward in lockstep, multiplies the amplitude of each pair of
/// `Frame`s together and yields the resulting `Frame`s.
///
/// The `Iterator` will return `None` when either of the `Signal`s first yields `None`.
#[derive(Clone)]
pub struct ZipModAmp<A, B> {
    signal_a: A,
    signal_b: B,
}


/// Provides an iterator that endlessly yields `Frame`s of type `F` at equilibrium.
///
/// # Example
///
/// ```rust
/// extern crate sample;
///
/// use sample::Signal;
///
/// fn main() {
///     let equilibrium: Vec<[f32; 1]> = sample::signal::equilibrium().take(4).collect();
///     assert_eq!(equilibrium, vec![[0.0], [0.0], [0.0], [0.0]]);
///
///     let equilibrium: Vec<[u8; 2]> = sample::signal::equilibrium().take(3).collect();
///     assert_eq!(equilibrium, vec![[128, 128], [128, 128], [128, 128]]);
/// }
/// ```
pub fn equilibrium<F>() -> Equilibrium<F> {
    Equilibrium { frame_type: std::marker::PhantomData }
}


impl<F> Iterator for Equilibrium<F>
    where F: Frame,
{
    type Item = F;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Some(F::equilibrium())
    }
}

impl<A, B> Iterator for AddAmp<A, B>
    where A: Signal,
          B: Signal,
          A::Item: Frame,
          B::Item: Frame<Sample=<<A::Item as Frame>::Sample as Sample>::Signed,
                         NumChannels=<A::Item as Frame>::NumChannels>,
{
    type Item = A::Item;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.a.next().and_then(|a_f| self.b.next().map(|b_f| a_f.add_amp(b_f)))
    }
}

impl<A, B> Iterator for MulAmp<A, B>
    where A: Signal,
          B: Signal,
          A::Item: Frame,
          B::Item: Frame<Sample=<<A::Item as Frame>::Sample as Sample>::Float,
                         NumChannels=<A::Item as Frame>::NumChannels>,
{
    type Item = A::Item;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.a.next().and_then(|a_f| self.b.next().map(|b_f| a_f.mul_amp(b_f)))
    }
}

impl<S> Iterator for ScaleAmp<S>
    where S: Signal,
          S::Item: Frame,
{
    type Item = S::Item;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.signal.next().map(|f| f.scale_amp(self.amp))
    }
}

impl<S, F> Iterator for ScaleAmpPerChannel<S, F>
    where S: Signal,
          S::Item: Frame,
          F: Frame<Sample=<<S::Item as Frame>::Sample as Sample>::Float,
                   NumChannels=<S::Item as Frame>::NumChannels>,
{
    type Item = S::Item;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.signal.next().map(|f| f.mul_amp(self.amp_frame))
    }
}

impl<S> Iterator for OffsetAmp<S>
    where S: Signal,
          S::Item: Frame,
{
    type Item = S::Item;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.signal.next().map(|f| f.offset_amp(self.offset))
    }
}

impl<S, F> Iterator for OffsetAmpPerChannel<S, F>
    where S: Signal,
          S::Item: Frame,
          F: Frame<Sample=<<S::Item as Frame>::Sample as Sample>::Signed,
                   NumChannels=<S::Item as Frame>::NumChannels>,
{
    type Item = S::Item;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.signal.next().map(|f| f.add_amp(self.amp_frame))
    }
}
