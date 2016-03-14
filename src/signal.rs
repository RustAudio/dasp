//! Use the [**Signal** trait](./trait.Signal.html) for working with **Iterator**s that yield
//! **Frame**s. To complement the **Iterator** trait, **Signal** provides methods for adding,
//! scaling, offsetting, multiplying, clipping and generating frame iterators and more.
//!
//! You may also find a series of `Signal` source functions, including:
//!
//! - [equilibrium](./fn.equilibrium.html) for generating "silent" frames.
//! - [gen](./fn.gen.html) for generating frames of type F from some `Fn() -> F`.
//! - [gen_mut](./fn.gen_mut.html) for generating frames of type F from some `FnMut() -> F`.
//! - [from_samples](./fn.from_samples.html) for converting an iterator yielding samples to an
//! iterator yielding frames.
//!
//! Working with **Signal**s allows for easy, readable creation of rich and complex DSP graphs with
//! a simple and familiar API.

use {Duplex, Frame, Sample};
use rate;
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
    ///     let added: Vec<_> = a_signal.mul_amp(b_signal).collect();
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

    /// Multiplies the rate at which frames of `self` are yielded by the given `signal`.
    ///
    /// This happens by wrapping `self` in a `rate::Converter` and calling `set_rate_multiplier`
    /// with the value yielded by `signal`
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Signal;
    ///
    /// fn main() {
    ///     let foo = [[0.0], [1.0], [0.0], [-1.0]];
    ///     let mul = [1.0, 1.0, 2.0, 2.0, 2.0, 2.0];
    ///     let frames: Vec<_> = foo.iter().cloned().mul_hz(mul.iter().cloned()).collect();
    ///     assert_eq!(&frames[..], &[[0.0], [1.0], [0.0], [-0.5], [-1.0]][..]);
    /// }
    /// ```
    fn mul_hz<I>(self, mul_per_frame: I) -> MulHz<Self, I>
        where I: Iterator<Item=f64>,
    {
        MulHz {
            signal: rate::Converter::scale_hz(self, 1.0),
            mul_per_frame: mul_per_frame,
        }
    }

    /// Converts the rate at which frames of the `Signal` are yielded using interpolation.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Signal;
    ///
    /// fn main() {
    ///     let foo = [[0.0], [1.0], [0.0], [-1.0]];
    ///     let frames: Vec<_> = foo.iter().cloned().from_hz_to_hz(1.0, 2.0).collect();
    ///     assert_eq!(&frames[..], &[[0.0], [0.5], [1.0], [0.5], [0.0], [-0.5], [-1.0]][..]);
    /// }
    /// ```
    fn from_hz_to_hz(self, source_hz: f64, target_hz: f64) -> rate::Converter<Self> {
        rate::Converter::from_hz_to_hz(self, source_hz, target_hz)
    }

    /// Multiplies the rate at which frames of the `Signal` are yielded by the given value.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Signal;
    ///
    /// fn main() {
    ///     let foo = [[0.0], [1.0], [0.0], [-1.0]];
    ///     let frames: Vec<_> = foo.iter().cloned().scale_hz(0.5).collect();
    ///     assert_eq!(&frames[..], &[[0.0], [0.5], [1.0], [0.5], [0.0], [-0.5], [-1.0]][..]);
    /// }
    /// ```
    fn scale_hz(self, multi: f64) -> rate::Converter<Self> {
        rate::Converter::scale_hz(self, multi)
    }

    /// Delays the `Signal` by the given number of frames.
    ///
    /// The delay is performed by yielding `Frame::equilibrium()` `n_frames` times before
    /// continuing to yield frames from `signal`.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Signal;
    ///
    /// fn main() {
    ///     let frames = [[0.2], [0.4]];
    ///     let delayed: Vec<_> = frames.iter().cloned().delay(2).collect();
    ///     assert_eq!(delayed, vec![[0.0], [0.0], [0.2], [0.4]]);
    /// }
    /// ```
    fn delay(self, n_frames: usize) -> Delay<Self> {
        Delay {
            signal: self,
            n_frames: n_frames,
        }
    }

    /// Converts a `Iterator` yielding `Frame`s into an `Iterator` yielding `Sample`s.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Signal;
    ///
    /// fn main() {
    ///     let frames = [[0.1, 0.2], [0.3, 0.4]];
    ///     let samples: Vec<_> = frames.iter().cloned().to_samples().collect();
    ///     assert_eq!(samples, vec![0.1, 0.2, 0.3, 0.4]);
    /// }
    /// ```
    fn to_samples(self) -> ToSamples<Self> {
        ToSamples {
            signal: self,
            current_frame: None,
        }
    }

    /// Clips the amplitude of each channel in each `Frame` yielded by `self` to the given
    /// threshold amplitude.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Signal;
    ///
    /// fn main() {
    ///     let frames = [[1.2, 0.8], [-0.7, -1.4]];
    ///     let clipped: Vec<_> = frames.iter().cloned().clip_amp(0.9).collect();
    ///     assert_eq!(clipped, vec![[0.9, 0.8], [-0.7, -0.9]]);
    /// }
    /// ```
    fn clip_amp(self, thresh: <<Self::Item as Frame>::Sample as Sample>::Signed) -> ClipAmp<Self> {
        ClipAmp {
            signal: self,
            thresh: thresh,
        }
    }

}


///// Signal Types


/// An iterator that endlessly yields `Frame`s of type `F` at equilibrium.
#[derive(Clone)]
pub struct Equilibrium<F> {
    frame: std::marker::PhantomData<F>,
}

/// A signal that generates frames using the given function.
#[derive(Clone)]
pub struct Gen<G, F> {
    gen: G,
    frame: std::marker::PhantomData<F>,
}

/// A signal that generates frames using the given function which may mutate some state.
#[derive(Clone)]
pub struct GenMut<G, F> {
    gen_mut: G,
    frame: std::marker::PhantomData<F>,
}

/// An iterator that converts an iterator of `Sample`s to an iterator of `Frame`s.
#[derive(Clone)]
pub struct FromSamples<I, F>
    where I: Iterator,
          I::Item: Sample,
          F: Frame<Sample=I::Item>,
{
    samples: I,
    frame: std::marker::PhantomData<F>,
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

/// Multiplies the rate at which frames of `self` are yielded by the given `signal`.
///
/// This happens by wrapping `self` in a `rate::Converter` and calling `set_rate_multiplier`
/// with the value yielded by `signal`
#[derive(Clone)]
pub struct MulHz<S, M>
    where S: Signal,
          S::Item: Frame,
{
    signal: rate::Converter<S>,
    mul_per_frame: M,
}

/// Delays the `signal` by the given number of frames.
///
/// The delay is performed by yielding `Frame::equilibrium()` `n_frames` times before
/// continuing to yield frames from `signal`.
#[derive(Clone)]
pub struct Delay<S> {
    signal: S,
    n_frames: usize,
}

/// Converts a `Signal` to an `Iterator` yielding `Sample`s of the signal.
pub struct ToSamples<S>
    where S: Signal,
          S::Item: Frame,
{
    signal: S,
    current_frame: Option<<S::Item as Frame>::Channels>,
}

/// Clips samples in each frame yielded by `signal` to the given threshhold amplitude.
#[derive(Clone)]
pub struct ClipAmp<S>
    where S: Signal,
          S::Item: Frame,
{
    signal: S,
    thresh: <<S::Item as Frame>::Sample as Sample>::Signed,
}



///// Signal Constructors


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
pub fn equilibrium<F>() -> Equilibrium<F>
    where F: Frame,
{
    Equilibrium { frame: std::marker::PhantomData }
}


/// A signal that generates frames using the given function.
///
/// # Example
///
/// ```rust
/// extern crate sample;
///
/// fn main() {
///     let mut frames = sample::signal::gen(|| [0.5]);
///     assert_eq!(frames.next(), Some([0.5]));
///     assert_eq!(frames.next(), Some([0.5]));
/// }
/// ```
pub fn gen<G, F>(gen: G) -> Gen<G, F>
    where G: Fn() -> F,
          F: Frame,
{
    Gen {
        gen: gen,
        frame: std::marker::PhantomData,
    }
}


/// A signal that generates frames using the given function which may mutate some state.
///
/// # Example
///
/// ```rust
/// extern crate sample;
///
/// fn main() {
///     let mut f = [0.0];
///     let mut frames = sample::signal::gen_mut(|| {
///         let r = f;
///         f[0] += 0.1;
///         r
///     });
///     assert_eq!(frames.next(), Some([0.0]));
///     assert_eq!(frames.next(), Some([0.1]));
///     assert_eq!(frames.next(), Some([0.2]));
/// }
/// ```
pub fn gen_mut<G, F>(gen_mut: G) -> GenMut<G, F>
    where G: FnMut() -> F,
          F: Frame,
{
    GenMut {
        gen_mut: gen_mut,
        frame: std::marker::PhantomData,
    }
}


/// An iterator that converts the given `Iterator` yielding `Sample`s to a `Signal` yielding frames
/// of type `F`.
///
/// # Example
///
/// ```rust
/// extern crate sample;
///
/// use sample::signal;
///
/// fn main() {
///     let foo = [0, 1, 2, 3];
///     let mut signal = signal::from_samples::<_, [i32; 2]>(foo.iter().cloned());
///     assert_eq!(signal.next(), Some([0, 1]));
///     assert_eq!(signal.next(), Some([2, 3]));
///     assert_eq!(signal.next(), None);
///
///     let bar = [0, 1, 2];
///     let mut signal = signal::from_samples::<_, [i32; 2]>(bar.iter().cloned());
///     assert_eq!(signal.next(), Some([0, 1]));
///     assert_eq!(signal.next(), None);
/// }
/// ```
pub fn from_samples<I, F>(samples: I) -> FromSamples<I, F>
    where I: Iterator,
          I::Item: Sample,
          F: Frame<Sample=I::Item>,
{
    FromSamples {
        samples: samples,
        frame: std::marker::PhantomData,
    }
}


//// Trait Implementations for Signal Types.


impl<F> Iterator for Equilibrium<F>
    where F: Frame,
{
    type Item = F;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Some(F::equilibrium())
    }
}

impl<F> DoubleEndedIterator for Equilibrium<F>
    where F: Frame,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        Some(F::equilibrium())
    }
}


impl<G, F> Iterator for Gen<G, F>
    where G: Fn() -> F,
{
    type Item = F;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Some((self.gen)())
    }
}


impl<G, F> Iterator for GenMut<G, F>
    where G: FnMut() -> F,
{
    type Item = F;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Some((self.gen_mut)())
    }
}


impl<I, F> Iterator for FromSamples<I, F>
    where I: Iterator,
          I::Item: Sample,
          F: Frame<Sample=I::Item>,
{
    type Item = F;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        F::from_samples(&mut self.samples)
    }
}


#[inline]
fn zipped_size_hint<A, B>(a: &A, b: &B) -> (usize, Option<usize>)
    where A: Iterator,
          B: Iterator,
{
    let (a_lower, a_upper) = a.size_hint();
    let (b_lower, b_upper) = b.size_hint();
    let lower = std::cmp::min(a_lower, b_lower);
    let upper = match (a_upper, b_upper) {
        (Some(a), Some(b)) => Some(std::cmp::min(a, b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };
    (lower, upper)
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
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        zipped_size_hint(&self.a, &self.b)
    }
}

impl<A, B> ExactSizeIterator for AddAmp<A, B>
    where AddAmp<A, B>: Iterator,
          A: ExactSizeIterator,
          B: ExactSizeIterator,
{
    #[inline]
    fn len(&self) -> usize {
        std::cmp::min(self.a.len(), self.b.len())
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
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        zipped_size_hint(&self.a, &self.b)
    }
}

impl<A, B> ExactSizeIterator for MulAmp<A, B>
    where MulAmp<A, B>: Iterator,
          A: ExactSizeIterator,
          B: ExactSizeIterator,
{
    fn len(&self) -> usize {
        std::cmp::min(self.a.len(), self.b.len())
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
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.signal.size_hint()
    }
}

impl<S> ExactSizeIterator for ScaleAmp<S>
    where S: Signal + ExactSizeIterator,
          S::Item: Frame,
{
    #[inline]
    fn len(&self) -> usize {
        self.signal.len()
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
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.signal.size_hint()
    }
}

impl<S, F> ExactSizeIterator for ScaleAmpPerChannel<S, F>
    where ScaleAmpPerChannel<S, F>: Iterator,
          S: ExactSizeIterator,
{
    #[inline]
    fn len(&self) -> usize {
        self.signal.len()
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
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.signal.size_hint()
    }
}

impl<S> ExactSizeIterator for OffsetAmp<S>
    where S: Signal + ExactSizeIterator,
          S::Item: Frame,
{
    #[inline]
    fn len(&self) -> usize {
        self.signal.len()
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
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.signal.size_hint()
    }
}

impl<S, F> ExactSizeIterator for OffsetAmpPerChannel<S, F>
    where OffsetAmpPerChannel<S, F>: Iterator,
          S: ExactSizeIterator,
{
    #[inline]
    fn len(&self) -> usize {
        self.signal.len()
    }
}


impl<S, M> Iterator for MulHz<S, M>
    where S: Signal,
          S::Item: Frame,
          <S::Item as Frame>::Sample: Duplex<f64>,
          M: Iterator<Item=f64>,
{
    type Item = S::Item;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.mul_per_frame.next().and_then(|mul| {
            self.signal.set_rate_multiplier(mul);
            self.signal.next()
        })
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // We can't make any guarantees about size here as the rate may change dramatically at any
        // point.
        (1, None)
    }
}


impl<S> Iterator for Delay<S>
    where S: Signal,
          S::Item: Frame,
{
    type Item = S::Item;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.n_frames > 0 {
            self.n_frames -= 1;
            Some(Frame::equilibrium())
        } else {
            self.signal.next()
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.signal.size_hint();
        (lower + self.n_frames, upper.map(|n| n + self.n_frames))
    }
}


impl<S> Iterator for ToSamples<S>
    where S: Signal,
          S::Item: Frame,
{
    type Item = <S::Item as Frame>::Sample;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut frame) = self.current_frame {
                if let Some(channel) = frame.next() {
                    return Some(channel);
                }
            }
            self.current_frame = match self.signal.next() {
                Some(frame) => Some(frame.channels()),
                None => return None,
            };
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.signal.size_hint();
        let current_frame = self.current_frame.as_ref().map(|chans| chans.size_hint());
        let n_channels = <S::Item as Frame>::n_channels();
        let lower = lower * n_channels + current_frame.map(|sh| sh.0).unwrap_or(0);
        let upper = upper.and_then(|upper| {
            let current_upper = match current_frame.map(|sh| sh.1) {
                None => 0,
                Some(None) => return None,
                Some(Some(n)) => n,
            };
            Some(upper * n_channels + current_upper)
        });
        (lower, upper)
    }
}

impl<S> Clone for ToSamples<S>
    where S: Signal + Clone,
          S::Item: Frame,
          <S::Item as Frame>::Channels: Clone,
{
    fn clone(&self) -> Self {
        ToSamples {
            signal: self.signal.clone(),
            current_frame: self.current_frame.clone(),
        }
    }
}


impl<S> Iterator for ClipAmp<S>
    where S: Signal,
          S::Item: Frame,
{
    type Item = S::Item;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.signal.next().map(|f| f.map(|s| {
            let s: <<S::Item as Frame>::Sample as Sample>::Signed = s.to_sample();
            if s > self.thresh { self.thresh } else if s < -self.thresh { -self.thresh } else { s }
                .to_sample()
        }))
    }
}
