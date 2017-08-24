//! Use the **Signal** trait for working with **Iterator**s that yield **Frame**s. To complement
//! the **Iterator** trait, **Signal** provides methods for adding, scaling, offsetting,
//! multiplying, clipping and generating frame iterators and more.
//!
//! You may also find a series of **Signal** source functions, including:
//!
//! - [equilibrium](./fn.equilibrium.html) for generating "silent" frames.
//! - [phase](./fn.phase.html) for a stepping phase, useful for oscillators.
//! - [sine](./fn.sine.html) for generating a sine waveform.
//! - [saw](./fn.saw.html) for generating a sawtooth waveform.
//! - [square](./fn.square.html) for generating a square waveform.
//! - [noise](./fn.noise.html) for generating a noise waveform.
//! - [noise_simplex](./fn.noise_simplex.html) for generating a 1D simplex noise waveform.
//! - [gen](./fn.gen.html) for generating frames of type F from some `Fn() -> F`.
//! - [gen_mut](./fn.gen_mut.html) for generating frames of type F from some `FnMut() -> F`.
//! - [from_iter](./fn.from_iter.html) for converting an iterator yielding frames to a signal.
//! - [from_interleaved_samples_iter](./fn.from_interleaved_samples_iter.html) for converting an
//! iterator yielding interleaved samples to a signal.
//!
//! Working with **Signal**s allows for easy, readable creation of rich and complex DSP graphs with
//! a simple and familiar API.

use {BTreeMap, Duplex, Frame, Sample, Rc, VecDeque};
use interpolate::{Converter, Interpolator};
use core;


/// Types that yield `Frame`s as a multi-channel PCM signal.
///
/// For example, `Signal` allows us to add two signals, modulate a signal's amplitude by another
/// signal, scale a signals amplitude and much more.
pub trait Signal {
    /// The `Frame` type returned by the `Signal`.
    type Frame: Frame;

    /// Yield the next `Frame` in the `Signal`.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::{signal, Signal};
    ///
    /// fn main() {
    ///     let frames = [[0.2], [-0.6], [0.4]];
    ///     let mut signal = signal::from_iter(frames.iter().cloned());
    ///     assert_eq!(signal.next(), [0.2]);
    ///     assert_eq!(signal.next(), [-0.6]);
    ///     assert_eq!(signal.next(), [0.4]);
    /// }
    /// ```
    fn next(&mut self) -> Self::Frame;

    /// Provides an iterator that yields the sum of the frames yielded by both `other` and `self`
    /// in lock-step.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::{signal, Signal};
    ///
    /// fn main() {
    ///     let a = [[0.2], [-0.6], [0.4]];
    ///     let b = [[0.2], [0.1], [-0.8]];
    ///     let a_signal = signal::from_iter(a.iter().cloned());
    ///     let b_signal = signal::from_iter(b.iter().cloned());
    ///     let added: Vec<_> = a_signal.add_amp(b_signal).take(3).collect();
    ///     assert_eq!(added, vec![[0.4], [-0.5], [-0.4]]);
    /// }
    /// ```
    #[inline]
    fn add_amp<S>(self, other: S) -> AddAmp<Self, S>
        where Self: Sized,
              S: Signal,
              S::Frame: Frame<Sample=<<Self::Frame as Frame>::Sample as Sample>::Signed,
                              NumChannels=<Self::Frame as Frame>::NumChannels>,
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
    /// use sample::{signal, Signal};
    ///
    /// fn main() {
    ///     let a = [[0.25], [-0.8], [-0.5]];
    ///     let b = [[0.2], [0.5], [0.8]];
    ///     let a_signal = signal::from_iter(a.iter().cloned());
    ///     let b_signal = signal::from_iter(b.iter().cloned());
    ///     let added: Vec<_> = a_signal.mul_amp(b_signal).take(3).collect();
    ///     assert_eq!(added, vec![[0.05], [-0.4], [-0.4]]);
    /// }
    /// ```
    #[inline]
    fn mul_amp<S>(self, other: S) -> MulAmp<Self, S>
        where Self: Sized,
              S: Signal,
              S::Frame: Frame<Sample=<<Self::Frame as Frame>::Sample as Sample>::Float,
                              NumChannels=<Self::Frame as Frame>::NumChannels>,
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
    /// use sample::{signal, Signal};
    ///
    /// fn main() {
    ///     let frames = [[0.25, 0.4], [-0.2, -0.5]];
    ///     let signal = signal::from_iter(frames.iter().cloned());
    ///     let offset: Vec<_> = signal.offset_amp(0.5).take(2).collect();
    ///     assert_eq!(offset, vec![[0.75, 0.9], [0.3, 0.0]]);
    /// }
    /// ```
    #[inline]
    fn offset_amp(self, offset: <<Self::Frame as Frame>::Sample as Sample>::Signed)
        -> OffsetAmp<Self>
        where Self: Sized,
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
    /// use sample::{signal, Signal};
    ///
    /// fn main() {
    ///     let frames = [[0.2], [-0.5], [-0.4], [0.3]];
    ///     let signal = signal::from_iter(frames.iter().cloned());
    ///     let scaled: Vec<_> = signal.scale_amp(2.0).take(4).collect();
    ///     assert_eq!(scaled, vec![[0.4], [-1.0], [-0.8], [0.6]]);
    /// }
    /// ```
    #[inline]
    fn scale_amp(self, amp: <<Self::Frame as Frame>::Sample as Sample>::Float) -> ScaleAmp<Self>
        where Self: Sized,
    {
        ScaleAmp {
            signal: self,
            amp: amp,
        }
    }

    /// Produces a new `Signal` that offsets the amplitude of every `Frame` in `self` by the
    /// respective amplitudes in each channel of the given `amp_frame`.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::{signal, Signal};
    ///
    /// fn main() {
    ///     let frames = [[0.5, 0.3], [-0.25, 0.9]];
    ///     let signal = signal::from_iter(frames.iter().cloned());
    ///     let offset: Vec<_> = signal.offset_amp_per_channel([0.25, -0.5]).take(2).collect();
    ///     assert_eq!(offset, vec![[0.75, -0.2], [0.0, 0.4]]);
    /// }
    /// ```
    #[inline]
    fn offset_amp_per_channel<F>(self, amp_frame: F) -> OffsetAmpPerChannel<Self, F>
        where Self: Sized,
              F: Frame<Sample=<<Self::Frame as Frame>::Sample as Sample>::Signed,
                       NumChannels=<Self::Frame as Frame>::NumChannels>,
    {
        OffsetAmpPerChannel {
            signal: self,
            amp_frame: amp_frame,
        }
    }

    /// Produces a new `Signal` that scales the amplitude of every `Frame` in `self` by the
    /// respective amplitudes in each channel of the given `amp_frame`.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::{signal, Signal};
    ///
    /// fn main() {
    ///     let frames = [[0.2, -0.5], [-0.4, 0.3]];
    ///     let signal = signal::from_iter(frames.iter().cloned());
    ///     let scaled: Vec<_> = signal.scale_amp_per_channel([0.5, 2.0]).take(2).collect();
    ///     assert_eq!(scaled, vec![[0.1, -1.0], [-0.2, 0.6]]);
    /// }
    /// ```
    #[inline]
    fn scale_amp_per_channel<F>(self, amp_frame: F) -> ScaleAmpPerChannel<Self, F>
        where Self: Sized,
              F: Frame<Sample=<<Self::Frame as Frame>::Sample as Sample>::Float,
                       NumChannels=<Self::Frame as Frame>::NumChannels>,
    {
        ScaleAmpPerChannel {
            signal: self,
            amp_frame: amp_frame,
        }
    }

    /// Multiplies the rate at which frames of `self` are yielded by the given `signal`.
    ///
    /// This happens by wrapping `self` in a `rate::Converter` and calling `set_playback_hz_scale`
    /// with each value yielded by `signal`
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::{signal, Signal};
    /// use sample::interpolate::Linear;
    ///
    /// fn main() {
    ///     let foo = [[0.0], [1.0], [0.0], [-1.0]];
    ///     let mul = [[1.0], [1.0], [0.5], [0.5], [0.5], [0.5]];
    ///     let mut source = signal::from_iter(foo.iter().cloned());
    ///     let interp = Linear::from_source(&mut source);
    ///     let hz_signal = signal::from_iter(mul.iter().cloned());
    ///     let frames: Vec<_> = source.mul_hz(interp, hz_signal).take(6).collect();
    ///     assert_eq!(&frames[..], &[[0.0], [1.0], [0.0], [-0.5], [-1.0], [-0.5]][..]);
    /// }
    /// ```
    fn mul_hz<M, I>(self, interpolator: I, mul_per_frame: M) -> MulHz<Self, M, I>
        where Self: Sized,
              M: Signal<Frame=[f64; 1]>,
              I: Interpolator,
    {
        MulHz {
            signal: Converter::scale_playback_hz(self, interpolator, 1.0),
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
    /// use sample::{signal, Signal};
    /// use sample::interpolate::Linear;
    ///
    /// fn main() {
    ///     let foo = [[0.0], [1.0], [0.0], [-1.0]];
    ///     let mut source = signal::from_iter(foo.iter().cloned());
    ///     let interp = Linear::from_source(&mut source);
    ///     let frames: Vec<_> = source.from_hz_to_hz(interp, 1.0, 2.0).take(8).collect();
    ///     assert_eq!(&frames[..], &[[0.0], [0.5], [1.0], [0.5], [0.0], [-0.5], [-1.0], [-0.5]][..]);
    /// }
    /// ```
    fn from_hz_to_hz<I>(self, interpolator: I, source_hz: f64, target_hz: f64) -> Converter<Self, I> 
        where Self: Sized,
              I: Interpolator,
    {
        Converter::from_hz_to_hz(self, interpolator, source_hz, target_hz)
    }

    /// Multiplies the rate at which frames of the `Signal` are yielded by the given value.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::{signal, Signal};
    /// use sample::interpolate::Linear;
    ///
    /// fn main() {
    ///     let foo = [[0.0], [1.0], [0.0], [-1.0]];
    ///     let mut source = signal::from_iter(foo.iter().cloned());
    ///     let interp = Linear::from_source(&mut source);
    ///     let frames: Vec<_> = source.scale_hz(interp, 0.5).take(8).collect();
    ///     assert_eq!(&frames[..], &[[0.0], [0.5], [1.0], [0.5], [0.0], [-0.5], [-1.0], [-0.5]][..]);
    /// }
    /// ```
    fn scale_hz<I>(self, interpolator: I, multi: f64) -> Converter<Self, I> 
        where Self: Sized,
              I: Interpolator,
    {
        Converter::scale_playback_hz(self, interpolator, multi)
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
    /// use sample::{signal, Signal};
    ///
    /// fn main() {
    ///     let frames = [[0.2], [0.4]];
    ///     let signal = signal::from_iter(frames.iter().cloned());
    ///     let delayed: Vec<_> = signal.delay(2).take(4).collect();
    ///     assert_eq!(delayed, vec![[0.0], [0.0], [0.2], [0.4]]);
    /// }
    /// ```
    fn delay(self, n_frames: usize) -> Delay<Self>
        where Self: Sized,
    {
        Delay {
            signal: self,
            n_frames: n_frames,
        }
    }

    /// Converts a `Signal` into a type that yields the interleaved `Sample`s.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::{signal, Signal};
    ///
    /// fn main() {
    ///     let frames = [[0.1, 0.2], [0.3, 0.4]];
    ///     let signal = signal::from_iter(frames.iter().cloned());
    ///     let samples = signal.into_interleaved_samples();
    ///     let samples: Vec<_> = samples.into_iter().take(4).collect();
    ///     assert_eq!(samples, vec![0.1, 0.2, 0.3, 0.4]);
    /// }
    /// ```
    fn into_interleaved_samples(mut self) -> IntoInterleavedSamples<Self>
        where Self: Sized,
    {
        let first = self.next().channels();
        IntoInterleavedSamples {
            signal: self,
            current_frame: first,
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
    /// use sample::{signal, Signal};
    ///
    /// fn main() {
    ///     let frames = [[1.2, 0.8], [-0.7, -1.4]];
    ///     let signal = signal::from_iter(frames.iter().cloned());
    ///     let clipped: Vec<_> = signal.clip_amp(0.9).take(2).collect();
    ///     assert_eq!(clipped, vec![[0.9, 0.8], [-0.7, -0.9]]);
    /// }
    /// ```
    fn clip_amp(self, thresh: <<Self::Frame as Frame>::Sample as Sample>::Signed) -> ClipAmp<Self>
        where Self: Sized,
    {
        ClipAmp {
            signal: self,
            thresh: thresh,
        }
    }

    /// Moves the `Signal` into a `Bus` from which its output may be divided into multiple other
    /// `Signal`s in the form of `Output`s.
    ///
    /// This method allows to create more complex directed acyclic graph structures that
    /// incorporate concepts like sends, side-chaining, etc, rather than being restricted to tree
    /// structures where signals can only ever be joined but never divided.
    ///
    /// Note: When using multiple `Output`s in this fashion, you will need to be sure to pull the
    /// frames from each `Output` in sync (whether per frame or per buffer). This is because when
    /// output A requests `Frame`s before output B, those frames must remain available for output
    /// B and in turn must be stored in an intermediary ring buffer.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::{signal, Signal};
    ///
    /// fn main() {
    ///     let frames = [[0.1], [0.2], [0.3], [0.4], [0.5], [0.6]];
    ///     let signal = signal::from_iter(frames.iter().cloned());
    ///     let bus = signal.bus();
    ///     let mut a = bus.send();
    ///     let mut b = bus.send();
    ///     assert_eq!(a.by_ref().take(3).collect::<Vec<_>>(), vec![[0.1], [0.2], [0.3]]);
    ///     assert_eq!(b.by_ref().take(3).collect::<Vec<_>>(), vec![[0.1], [0.2], [0.3]]);
    ///
    ///     let c = bus.send();
    ///     assert_eq!(c.take(3).collect::<Vec<_>>(), vec![[0.4], [0.5], [0.6]]);
    ///     assert_eq!(b.take(3).collect::<Vec<_>>(), vec![[0.4], [0.5], [0.6]]);
    ///     assert_eq!(a.take(3).collect::<Vec<_>>(), vec![[0.4], [0.5], [0.6]]);
    /// }
    /// ```
    fn bus(self) -> Bus<Self>
        where Self: Sized,
    {
        Bus::new(self, BTreeMap::new())
    }

    /// Converts the `Signal` into an `Iterator` that will yield the given number for `Frame`s
    /// before returning `None`.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::{signal, Signal};
    ///
    /// fn main() {
    ///     let frames = [[0.1], [0.2], [0.3], [0.4]];
    ///     let mut signal = signal::from_iter(frames.iter().cloned()).take(2);
    ///     assert_eq!(signal.next(), Some([0.1]));
    ///     assert_eq!(signal.next(), Some([0.2]));
    ///     assert_eq!(signal.next(), None);
    /// }
    /// ```
    fn take(self, n: usize) -> Take<Self>
        where Self: Sized,
    {
        Take {
            signal: self,
            n: n,
        }
    }

    /// Borrows a Signal rather than consuming it.
    ///
    /// This is useful to allow applying signal adaptors while still retaining ownership of the
    /// original signal.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::{signal, Signal};
    ///
    /// fn main() {
    ///     let frames = [[0], [1], [2], [3], [4]];
    ///     let mut signal = signal::from_iter(frames.iter().cloned());
    ///     assert_eq!(signal.next(), [0]);
    ///     assert_eq!(signal.by_ref().take(2).collect::<Vec<_>>(), vec![[1], [2]]);
    ///     assert_eq!(signal.next(), [3]);
    ///     assert_eq!(signal.next(), [4]);
    /// }
    /// ```
    fn by_ref(&mut self) -> &mut Self {
        self
    }
}


impl<'a, S> Signal for &'a mut S
    where S: Signal,
{
    type Frame = S::Frame;
    fn next(&mut self) -> Self::Frame {
        (**self).next()
    }
}


///// Signal Types


/// An iterator that endlessly yields `Frame`s of type `F` at equilibrium.
#[derive(Clone)]
pub struct Equilibrium<F> {
    frame: core::marker::PhantomData<F>,
}

/// A signal that generates frames using the given function.
#[derive(Clone)]
pub struct Gen<G, F> {
    gen: G,
    frame: core::marker::PhantomData<F>,
}

/// A signal that generates frames using the given function which may mutate some state.
#[derive(Clone)]
pub struct GenMut<G, F> {
    gen_mut: G,
    frame: core::marker::PhantomData<F>,
}

/// A signal that calls its enclosing function and returns the original value.
#[derive(Clone)]
pub struct Tap<S, Func, F> {
    signal: S,
    func: Func,
    frame: core::marker::PhantomData<F>,
}

/// A signal that calls its enclosing function and returns the original value which may 
/// mutate some state.
#[derive(Clone)]
pub struct TapMut<S, Func, F> {
    signal: S,
    func: Func,
    frame: core::marker::PhantomData<F>,
}

/// A type that wraps an Iterator and provides a `Signal` implementation for it.
#[derive(Clone)]
pub struct FromIterator<I> {
    iter: I,
}

/// An iterator that converts an iterator of `Sample`s to an iterator of `Frame`s.
#[derive(Clone)]
pub struct FromInterleavedSamplesIterator<I, F>
    where I: Iterator,
          I::Item: Sample,
          F: Frame<Sample=I::Item>,
{
    samples: I,
    frame: core::marker::PhantomData<F>,
}

/// The rate at which phrase a **Signal** is sampled.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rate {
    hz: f64,
}

/// A constant phase step size.
#[derive(Clone)]
pub struct ConstHz {
    step: f64,
}

/// An iterator that yields the step size for a phase.
#[derive(Clone)]
pub struct Hz<I> {
    hz: I,
    last_step_size: f64,
    rate: Rate,
}


/// An iterator that yields a phase, useful for waveforms like Sine or Saw.
#[derive(Clone)]
pub struct Phase<S> {
    step: S,
    next: f64,
}

/// A sine wave signal generator.
#[derive(Clone)]
pub struct Sine<S> {
    phase: Phase<S>,
}

/// A saw wave signal generator.
#[derive(Clone)]
pub struct Saw<S> {
    phase: Phase<S>,
}

/// A square wave signal generator.
#[derive(Clone)]
pub struct Square<S> {
    phase: Phase<S>,
}

/// A noise signal generator.
#[derive(Clone)]
pub struct Noise {
    seed: u64,
}

/// A 1D simplex-noise generator.
#[derive(Clone)]
pub struct NoiseSimplex<S> {
    phase: Phase<S>,
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
{
    signal: S,
    offset: <<S::Frame as Frame>::Sample as Sample>::Signed,
}

/// An `Iterator` that scales the amplitude of the sample of each channel in every `Frame` yielded
/// by `self` by the given amplitude.
#[derive(Clone)]
pub struct ScaleAmp<S>
    where S: Signal,
{
    signal: S,
    amp: <<S::Frame as Frame>::Sample as Sample>::Float,
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
/// This happens by wrapping `self` in a `rate::Converter` and calling `set_playback_hz_scale`
/// with the value yielded by `signal`
#[derive(Clone)]
pub struct MulHz<S, M, I>
    where S: Signal,
          I: Interpolator,
{
    signal: Converter<S, I>,
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

/// Converts a `Signal` to a type that yields the individual interleaved samples.
pub struct IntoInterleavedSamples<S>
    where S: Signal,
{
    signal: S,
    current_frame: <S::Frame as Frame>::Channels,
}

/// Converts the `IntoInterleavedSamples` into an `Iterator` that always returns `Some`.
pub struct IntoInterleavedSamplesIterator<S>
    where S: Signal,
{
    samples: IntoInterleavedSamples<S>,
}

/// Clips samples in each frame yielded by `signal` to the given threshhold amplitude.
#[derive(Clone)]
pub struct ClipAmp<S>
    where S: Signal,
{
    signal: S,
    thresh: <<S::Frame as Frame>::Sample as Sample>::Signed,
}

/// A type which allows for `send`ing a single `Signal` to multiple outputs.
///
/// This type manages
pub struct Bus<S>
    where S: Signal,
{
    node: Rc<core::cell::RefCell<SharedNode<S>>>,
}

/// The data shared between each `Output`.
struct SharedNode<S>
    where S: Signal,
{
    signal: S,
    // The buffer of frames that have not yet been consumed by all outputs.
    buffer: VecDeque<S::Frame>,
    // The number of frames in `buffer` that have already been read for each output.
    frames_read: BTreeMap<usize, usize>,
    // The next output key.
    next_key: usize,
}

/// An output node to which some signal `S` is `Output`ing its frames.
///
/// It may be more accurate to say that the `Output` "pull"s frames from the signal.
pub struct Output<S>
    where S: Signal,
{
    key: usize,
    node: Rc<core::cell::RefCell<SharedNode<S>>>,
}

/// An iterator that yields `n` number of `Frame`s from the inner `signal`.
#[derive(Clone)]
pub struct Take<S>
    where S: Signal,
{
    signal: S,
    n: usize,
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
    Equilibrium { frame: core::marker::PhantomData }
}


/// A signal that generates frames using the given function.
///
/// # Example
///
/// ```rust
/// extern crate sample;
///
/// use sample::{signal, Signal};
///
/// fn main() {
///     let mut frames = signal::gen(|| [0.5]);
///     assert_eq!(frames.next(), [0.5]);
///     assert_eq!(frames.next(), [0.5]);
///     assert_eq!(frames.next(), [0.5]);
/// }
/// ```
pub fn gen<G, F>(gen: G) -> Gen<G, F>
    where G: Fn() -> F,
          F: Frame,
{
    Gen {
        gen: gen,
        frame: core::marker::PhantomData,
    }
}


/// A signal that generates frames using the given function which may mutate some state.
///
/// # Example
///
/// ```rust
/// extern crate sample;
///
/// use sample::{signal, Signal};
///
/// fn main() {
///     let mut f = [0.0];
///     let mut signal = signal::gen_mut(|| {
///         let r = f;
///         f[0] += 0.1;
///         r
///     });
///     assert_eq!(signal.next(), [0.0]);
///     assert_eq!(signal.next(), [0.1]);
///     assert_eq!(signal.next(), [0.2]);
/// }
/// ```
pub fn gen_mut<G, F>(gen_mut: G) -> GenMut<G, F>
    where G: FnMut() -> F,
          F: Frame,
{
    GenMut {
        gen_mut: gen_mut,
        frame: core::marker::PhantomData,
    }
}

/// Create a new `Signal` that calls the enclosing function on each iteration.
///
/// # Example
///
/// ```rust
/// extern crate sample;
///
/// use sample::{signal, Signal};
///
/// fn main() {
///     let mut f = [0.0];
///     let mut signal = signal::gen_mut(move || {
///         f[0] += 0.1;
///         f
///     });
///     let func = |x: &[f64; 1]| {
///         assert_eq!(*x, [0.1]);
///     };
///     let mut tapped = signal::tap(signal, func);
///     let out = tapped.next();
///     assert_eq!(out, [0.1]);
/// }
/// ```
pub fn tap<S, Func, F>(signal: S, func: Func) -> Tap<S, Func, F>
    where S: Signal<Frame=F>,
          Func: Fn(&F) -> (),
          F: Frame,
{
    Tap {
        signal: signal,
        func: func,
        frame: core::marker::PhantomData,
    }
}

/// Create a new `Signal` that calls the enclosing function on each iteration. 
/// Function may mutate its environment.
///
/// # Example
///
/// ```rust
/// extern crate sample;
///
/// use sample::{signal, Signal};
///
/// fn main() {
///     let mut f = [0.0];
///     let mut signal = signal::gen_mut(move || {
///         f[0] += 0.1;
///         f
///     });
///
///     let mut out = [0.0];
///     // Need to enclose this segment to ensure that the borrow of out is limited
///     {
///         let func = |x: &[f64; 1]| {
///             let borrowed_out = &mut out;
///             *borrowed_out = *x;
///         };
///         let mut tapped = signal::tap_mut(signal, func);
///         let sig_out = tapped.next();
///         assert_eq!(sig_out, [0.1]);
///     }
///
///     assert_eq!(out, [0.1])
/// }
/// ```
pub fn tap_mut<S, Func, F>(signal: S, func: Func) -> TapMut<S, Func, F>
    where S: Signal<Frame=F>,
          Func: FnMut(&F) -> (),
          F: Frame,
{
    TapMut {
        signal: signal,
        func: func,
        frame: core::marker::PhantomData,
    }
}

/// Create a new `Signal` from the given `Frame`-yielding `Iterator`.
///
/// When the `Iterator` is exhausted, the new `Signal` will yield `F::equilibrium`.
///
/// # Example
///
/// ```rust
/// extern crate sample;
///
/// use sample::{signal, Signal};
///
/// fn main() {
///     let frames = [[1], [-3], [5], [6]];
///     let mut signal = signal::from_iter(frames.iter().cloned());
///     assert_eq!(signal.next(), [1]);
///     assert_eq!(signal.next(), [-3]);
///     assert_eq!(signal.next(), [5]);
///     assert_eq!(signal.next(), [6]);
///     assert_eq!(signal.next(), [0]);
/// }
/// ```
pub fn from_iter<I>(frames: I) -> FromIterator<I::IntoIter>
    where I: IntoIterator,
          I::Item: Frame,
{
    FromIterator {
        iter: frames.into_iter(),
    }
}


/// Create a new `Signal` from the given `Frame`-yielding `Iterator`.
///
/// When the `Iterator` is exhausted, the new `Signal` will yield `F::equilibrium`.
///
/// # Example
///
/// ```rust
/// extern crate sample;
///
/// use sample::{signal, Signal};
///
/// fn main() {
///     let foo = [0, 1, 2, 3];
///     let mut signal = signal::from_interleaved_samples_iter::<_, [i32; 2]>(foo.iter().cloned());
///     assert_eq!(signal.next(), [0, 1]);
///     assert_eq!(signal.next(), [2, 3]);
///     assert_eq!(signal.next(), [0, 0]);
///
///     let bar = [0, 1, 2];
///     let mut signal = signal::from_interleaved_samples_iter::<_, [i32; 2]>(bar.iter().cloned());
///     assert_eq!(signal.next(), [0, 1]);
///     assert_eq!(signal.next(), [0, 0]);
/// }
/// ```
pub fn from_interleaved_samples_iter<I, F>(samples: I) -> FromInterleavedSamplesIterator<I::IntoIter, F>
    where I: IntoIterator,
          I::Item: Sample,
          F: Frame<Sample=I::Item>,
{
    FromInterleavedSamplesIterator {
        samples: samples.into_iter(),
        frame: core::marker::PhantomData,
    }
}


/// Creates a `Phase` that continuously steps forward by the given `step` size yielder.
///
/// # Example
///
/// ```rust
/// extern crate sample;
///
/// use sample::{signal, Signal};
///
/// fn main() {
///     let step = signal::rate(4.0).const_hz(1.0);
///     // Note that this is the same as `step.phase()`, a composable alternative.
///     let mut phase = signal::phase(step);
///     assert_eq!(phase.next(), [0.0]);
///     assert_eq!(phase.next(), [0.25]);
///     assert_eq!(phase.next(), [0.5]);
///     assert_eq!(phase.next(), [0.75]);
///     assert_eq!(phase.next(), [0.0]);
///     assert_eq!(phase.next(), [0.25]);
/// }
/// ```
pub fn phase<S>(step: S) -> Phase<S>
    where S: Step,
{
    Phase {
        step: step,
        next: 0.0,
    }
}


/// Creates a frame `Rate` (aka sample rate) representing the rate at which a signal may be
/// sampled.
///
/// This is necessary for composing `Hz` or `ConstHz`, both of which may be used to step forward
/// the `Phase` for some kind of oscillator (i.e. `Sine`, `Saw`, `Square` or `NoiseSimplex`).
pub fn rate(hz: f64) -> Rate {
    Rate { hz: hz }
}


/// Produces a `Signal` that yields a sine wave oscillating at the given hz.
///
/// # Example
///
/// ```rust
/// extern crate sample;
///
/// use sample::{signal, Signal};
///
/// fn main() {
///     // Generates a sine wave signal at 1hz to be sampled 4 times per second.
///     let mut signal = signal::rate(4.0).const_hz(1.0).sine();
///     assert_eq!(signal.next(), [0.0]);
///     assert_eq!(signal.next(), [1.0]);
///     signal.next();
///     assert_eq!(signal.next(), [-1.0]);
/// }
/// ```
pub fn sine<S>(phase: Phase<S>) -> Sine<S> {
    Sine { phase: phase }
}

/// Produces a `Signal` that yields a saw wave oscillating at the given hz.
///
/// # Example
///
/// ```rust
/// extern crate sample;
///
/// use sample::{signal, Signal};
///
/// fn main() {
///     // Generates a saw wave signal at 1hz to be sampled 4 times per second.
///     let mut signal = signal::rate(4.0).const_hz(1.0).saw();
///     assert_eq!(signal.next(), [1.0]);
///     assert_eq!(signal.next(), [0.5]);
///     assert_eq!(signal.next(), [0.0]);
///     assert_eq!(signal.next(), [-0.5]);
/// }
/// ```
pub fn saw<S>(phase: Phase<S>) -> Saw<S> {
    Saw { phase: phase }
}

/// Produces a `Signal` that yields a square wave oscillating at the given hz.
///
/// # Example
///
/// ```rust
/// extern crate sample;
///
/// use sample::{signal, Signal};
///
/// fn main() {
///     // Generates a square wave signal at 1hz to be sampled 4 times per second.
///     let mut signal = signal::rate(4.0).const_hz(1.0).square();
///     assert_eq!(signal.next(), [1.0]);
///     assert_eq!(signal.next(), [1.0]);
///     assert_eq!(signal.next(), [-1.0]);
///     assert_eq!(signal.next(), [-1.0]);
/// }
/// ```
pub fn square<S>(phase: Phase<S>) -> Square<S> {
    Square { phase: phase }
}

/// Produces a `Signal` that yields random values between -1.0..1.0.
///
/// # Example
///
/// ```rust
/// extern crate sample;
///
/// use sample::{signal, Signal};
///
/// fn main() {
///     let mut noise = sample::signal::noise(0);
///     for n in noise.take(1_000_000) {
///         assert!(-1.0 <= n[0] && n[0] < 1.0);
///     }
/// }
/// ```
pub fn noise(seed: u64) -> Noise {
    Noise { seed: seed }
}

/// Produces a 1-dimensional simplex noise `Signal`.
///
/// This is sometimes known as the "drunken walk" or "noise walk".
///
/// # Example
///
/// ```rust
/// extern crate sample;
///
/// use sample::{signal, Signal};
///
/// fn main() {
///     // Creates a simplex noise signal oscillating at 440hz sampled 44_100 times per second.
///     let mut signal = signal::rate(44_100.0).const_hz(440.0).noise_simplex();
///     for n in signal.take(1_000_000) {
///         assert!(-1.0 <= n[0] && n[0] < 1.0);
///     }
/// }
/// ```
pub fn noise_simplex<S>(phase: Phase<S>) -> NoiseSimplex<S> {
    NoiseSimplex { phase: phase }
}


//// Trait Implementations for Signal Types.


impl<I> Signal for FromIterator<I>
    where I: Iterator,
          I::Item: Frame,
{
    type Frame = I::Item;
    #[inline]
    fn next(&mut self) -> Self::Frame {
        match self.iter.next() {
            Some(frame) => frame,
            None => Self::Frame::equilibrium(),
        }
    }
}


impl<I, F> Signal for FromInterleavedSamplesIterator<I, F>
    where I: Iterator,
          I::Item: Sample,
          F: Frame<Sample=I::Item>,
{
    type Frame = F;
    #[inline]
    fn next(&mut self) -> Self::Frame {
        F::from_samples(&mut self.samples).unwrap_or(F::equilibrium())
    }
}


impl<F> Signal for Equilibrium<F>
    where F: Frame,
{
    type Frame = F;
    #[inline]
    fn next(&mut self) -> Self::Frame {
        F::equilibrium()
    }
}


impl<G, F> Signal for Gen<G, F>
    where G: Fn() -> F,
          F: Frame,
{
    type Frame = F;
    #[inline]
    fn next(&mut self) -> Self::Frame {
        (self.gen)()
    }
}


impl<G, F> Signal for GenMut<G, F>
    where G: FnMut() -> F,
          F: Frame,
{
    type Frame = F;
    #[inline]
    fn next(&mut self) -> Self::Frame {
        (self.gen_mut)()
    }
}

impl<S, Func, F> Signal for Tap<S, Func, F>
    where S: Signal<Frame=F>,
          Func: Fn(&F) -> (),
          F: Frame
{
    type Frame = F;

    fn next(&mut self) -> Self::Frame {
        let out = self.signal.next();
        (self.func)(&out);
        out
    }
}

impl<S, Func, F> Signal for TapMut<S, Func, F>
    where S: Signal<Frame=F>,
          Func: FnMut(&F) -> (),
          F: Frame
{
    type Frame = F;

    fn next(&mut self) -> Self::Frame {
        let out = self.signal.next();
        (self.func)(&out);
        out
    }
}

impl<S> Signal for Hz<S>
    where S: Signal<Frame=[f64; 1]>,
{
    type Frame = [f64; 1];
    #[inline]
    fn next(&mut self) -> Self::Frame {
        [self.step()]
    }
}


impl Signal for ConstHz {
    type Frame = [f64; 1];
    #[inline]
    fn next(&mut self) -> Self::Frame {
        [self.step()]
    }
}


impl<S> Signal for Phase<S>
    where S: Step,
{
    type Frame = [f64; 1];
    #[inline]
    fn next(&mut self) -> Self::Frame {
        [self.next_phase()]
    }
}


impl<S> Signal for Sine<S>
    where S: Step,
{
    type Frame = [f64; 1];
    #[inline]
    fn next(&mut self) -> Self::Frame {
        const PI_2: f64 = core::f64::consts::PI * 2.0;
        let phase = self.phase.next_phase();
        [super::ops::f64::sin(PI_2 * phase)]
    }
}


impl<S> Signal for Saw<S>
    where S: Step,
{
    type Frame = [f64; 1];
    #[inline]
    fn next(&mut self) -> Self::Frame {
        let phase = self.phase.next_phase();
        [phase * -2.0 + 1.0]
    }
}


impl<S> Signal for Square<S>
    where S: Step,
{
    type Frame = [f64; 1];
    #[inline]
    fn next(&mut self) -> Self::Frame {
        let phase = self.phase.next_phase();
        [if phase < 0.5 { 1.0 } else { -1.0 }]
    }
}


impl Rate {
    /// Create a `ConstHz` iterator which consistently yields "hz / rate".
    pub fn const_hz(self, hz: f64) -> ConstHz {
        ConstHz { step: hz / self.hz }
    }

    /// Create a variable `hz` some iterator that yields hz and an initial hz.
    ///
    /// The `Hz` iterator yields phase step sizes equal to "hz / rate".
    pub fn hz<I>(self, init: f64, hz: I) -> Hz<I>
        where I: Iterator<Item=f64>,
    {
        Hz {
            hz: hz,
            last_step_size: init / self.hz,
            rate: self,
        }
    }
}

impl<S> Hz<S>
    where S: Signal<Frame=[f64; 1]>,
{
    /// Construct a `Phase` iterator that, for every `hz` yielded by `self`, yields a phase that is
    /// stepped by `hz / self.rate.hz`.
    #[inline]
    pub fn phase(self) -> Phase<Self> {
        phase(self)
    }

    /// A composable alternative to the `signal::sine` function.
    #[inline]
    pub fn sine(self) -> Sine<Self> {
        self.phase().sine()
    }

    /// A composable alternative to the `signal::saw` function.
    #[inline]
    pub fn saw(self) -> Saw<Self> {
        self.phase().saw()
    }

    /// A composable alternative to the `signal::square` function.
    #[inline]
    pub fn square(self) -> Square<Self> {
        self.phase().square()
    }

    /// A composable alternative to the `signal::noise_simplex` function.
    #[inline]
    pub fn noise_simplex(self) -> NoiseSimplex<Self> {
        self.phase().noise_simplex()
    }
}

impl ConstHz {
    /// Construct a `Phase` iterator that is incremented via the constant step size, `self.step`.
    #[inline]
    pub fn phase(self) -> Phase<Self> {
        phase(self)
    }

    /// A composable alternative to the `signal::sine` function.
    #[inline]
    pub fn sine(self) -> Sine<Self> {
        self.phase().sine()
    }

    /// A composable alternative to the `signal::saw` function.
    #[inline]
    pub fn saw(self) -> Saw<Self> {
        self.phase().saw()
    }

    /// A composable alternative to the `signal::square` function.
    #[inline]
    pub fn square(self) -> Square<Self> {
        self.phase().square()
    }

    /// A composable alternative to the `signal::noise_simplex` function.
    #[inline]
    pub fn noise_simplex(self) -> NoiseSimplex<Self> {
        self.phase().noise_simplex()
    }
}

/// Types that may be used to give a phase step size based on some `hz / sample rate`.
///
/// This allows the `Phase` to be generic over either `ConstHz` and `Hz<I>`.
///
/// Generally, users need not be concerned with this trait unless writing code that must remain
/// generic over phase stepping types like oscillators.
pub trait Step {
    /// Yield the phase step size (normally `hz / sampling rate`).
    ///
    /// The `Phase` calls this and uses the returned value to step forward its internal `phase`.
    fn step(&mut self) -> f64;
}

impl Step for ConstHz {
    #[inline]
    fn step(&mut self) -> f64 {
        self.step
    }
}

impl<S> Step for Hz<S>
    where S: Signal<Frame=[f64; 1]>,
{
    #[inline]
    fn step(&mut self) -> f64 {
        let hz = self.hz.next()[0];
        self.last_step_size = hz / self.rate.hz;
        hz
    }
}


impl<S> Phase<S>
    where S: Step,
{
    /// Before yielding the current phase, the internal phase is stepped forward and wrapped via
    /// the given value.
    #[inline]
    pub fn next_phase_wrapped_to(&mut self, rem: f64) -> f64 {
        let phase = self.next;
        self.next = (self.next + self.step.step()) % rem;
        phase
    }

    /// Calls `next_phase_wrapped_to`, with a wrapping value of `1.0`.
    #[inline]
    pub fn next_phase(&mut self) -> f64 {
        self.next_phase_wrapped_to(1.0)
    }

    /// A composable version of the `signal::sine` function.
    #[inline]
    pub fn sine(self) -> Sine<S> {
        sine(self)
    }

    /// A composable version of the `signal::saw` function.
    #[inline]
    pub fn saw(self) -> Saw<S> {
        saw(self)
    }

    /// A composable version of the `signal::square` function.
    #[inline]
    pub fn square(self) -> Square<S> {
        square(self)
    }

    /// A composable version of the `signal::noise_simplex` function.
    #[inline]
    pub fn noise_simplex(self) -> NoiseSimplex<S> {
        noise_simplex(self)
    }
}


impl Noise {
    #[inline]
    pub fn next_sample(&mut self) -> f64 {
        // A simple one-dimensional noise generator.
        //
        // Credit for the pseudo code from which this was translated goes to Hugo Elias and his
        // excellent primer on perlin noise at
        // http://freespace.virgin.net/hugo.elias/models/m_perlin.htm
        fn noise_1(seed: u64) -> f64 {
            const PRIME_1: u64 = 15_731;
            const PRIME_2: u64 = 789_221;
            const PRIME_3: u64 = 1_376_312_589;
            let x = (seed << 13) ^ seed;
            1.0 - (
                x.wrapping_mul(x.wrapping_mul(x).wrapping_mul(PRIME_1).wrapping_add(PRIME_2))
                    .wrapping_add(PRIME_3) & 0x7fffffff
            ) as f64 / 1_073_741_824.0
        }

        let noise = noise_1(self.seed);
        self.seed += 1;
        noise
    }
}

impl Signal for Noise {
    type Frame = [f64; 1];
    #[inline]
    fn next(&mut self) -> Self::Frame {
        [self.next_sample()]
    }
}


impl<S> NoiseSimplex<S>
    where S: Step,
{
    #[inline]
    pub fn next_sample(&mut self) -> f64 {
        // The constant remainder used to wrap the phase back to 0.0.
        //
        // This is the first power of two that is over double the human hearing range. This should
        // allow for simplex noise to be generated at a frequency matching the extent of the human
        // hearing range while never repeating more than once per second; the repetition would
        // likely be indistinguishable at such a high frequency, and in this should be practical
        // for audio simplex noise.
        const TWO_POW_SIXTEEN: f64 = 65_536.0;
        let phase = self.phase.next_phase_wrapped_to(TWO_POW_SIXTEEN);

        // 1D Perlin simplex noise.
        //
        // Takes a floating point x coordinate and yields a noise value in the range of -1..1, with
        // value of 0.0 on all integer coordinates.
        //
        // This function and the enclosing functions have been adapted from SRombauts' MIT licensed
        // C++ implementation at the following link: https://github.com/SRombauts/SimplexNoise
        fn simplex_noise_1d(x: f64) -> f64 {

            // Permutation table. This is a random jumble of all numbers 0...255.
            const PERM: [u8; 256] = [
                151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36,
                103, 30, 69, 142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0,
                26, 197, 62, 94, 252, 219, 203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87,
                174, 20, 125, 136, 171, 168, 68, 175, 74, 165, 71, 134, 139, 48, 27, 166, 77, 146,
                158, 231, 83, 111, 229, 122, 60, 211, 133, 230, 220, 105, 92, 41, 55, 46, 245, 40,
                244, 102, 143, 54, 65, 25, 63, 161, 1, 216, 80, 73, 209, 76, 132, 187, 208, 89, 18,
                169, 200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109, 198, 173, 186, 3, 64,
                52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212, 207, 206,
                59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213, 119, 248, 152, 2,
                44, 154, 163, 70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19, 98,
                108, 110, 79, 113, 224, 232, 178, 185, 112, 104, 218, 246, 97, 228, 251, 34, 242,
                193, 238, 210, 144, 12, 191, 179, 162, 241, 81, 51, 145, 235, 249, 14, 239, 107,
                49, 192, 214, 31, 181, 199, 106, 157, 184, 84, 204, 176, 115, 121, 50, 45, 127, 4,
                150, 254, 138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141, 128, 195, 78, 66,
                215, 61, 156, 180
            ];

            // Hashes the given integer with the above permutation table.
            fn hash(i: i64) -> u8 {
                PERM[(i as u8) as usize]
            }

            // Computes the gradients-dot-residual vectors (1D).
            fn grad(hash: i64, x: f64) -> f64 {
                // Convert low 4 bits of hash code.
                let h = hash & 0x0F;
                // Gradien value 1.0, 2.0, ..., 8.0.
                let mut grad = 1.0 + (h & 7) as f64;
                // Set a random sign for the gradient.
                if (h & 8) != 0 { grad = -grad; }
                // Multiply the gradient with the distance.
                grad * x
            }

            // Corners coordinates (nearest integer values).
            let i0 = super::ops::f64::floor(x) as i64;
            let i1 = i0 + 1;

            // Distances to corners (between 0 and 1);
            let x0 = x - i0 as f64;
            let x1 = x0 - 1.0;

            // Calculate the contribution from the first corner.
            let mut t0 = 1.0 - x0 * x0;
            t0 *= t0;
            let n0 = t0 * t0 * grad(hash(i0) as i64, x0);

            // Calculate the contribution rom the second corner.
            let mut t1 = 1.0 - x1 * x1;
            t1 *= t1;
            let n1 = t1 * t1 * grad(hash(i1) as i64, x1);

            // The max value of this noise is 2.53125. 0.395 scales to fit exactly within -1..1.
            0.395 * (n0 + n1)
        }

        simplex_noise_1d(phase)
    }
}

impl<S> Signal for NoiseSimplex<S>
    where S: Step,
{
    type Frame = [f64; 1];
    #[inline]
    fn next(&mut self) -> Self::Frame {
        [self.next_sample()]
    }
}


impl<A, B> Signal for AddAmp<A, B>
    where A: Signal,
          B: Signal,
          B::Frame: Frame<Sample=<<A::Frame as Frame>::Sample as Sample>::Signed,
                          NumChannels=<A::Frame as Frame>::NumChannels>,
{
    type Frame = A::Frame;
    #[inline]
    fn next(&mut self) -> Self::Frame {
        self.a.next().add_amp(self.b.next())
    }
}


impl<A, B> Signal for MulAmp<A, B>
    where A: Signal,
          B: Signal,
          B::Frame: Frame<Sample=<<A::Frame as Frame>::Sample as Sample>::Float,
                          NumChannels=<A::Frame as Frame>::NumChannels>,
{
    type Frame = A::Frame;
    #[inline]
    fn next(&mut self) -> Self::Frame {
        self.a.next().mul_amp(self.b.next())
    }
}


impl<S> Signal for ScaleAmp<S>
    where S: Signal,
{
    type Frame = S::Frame;
    #[inline]
    fn next(&mut self) -> Self::Frame {
        self.signal.next().scale_amp(self.amp)
    }
}


impl<S, F> Signal for ScaleAmpPerChannel<S, F>
    where S: Signal,
          F: Frame<Sample=<<S::Frame as Frame>::Sample as Sample>::Float,
                   NumChannels=<S::Frame as Frame>::NumChannels>,
{
    type Frame = S::Frame;
    #[inline]
    fn next(&mut self) -> Self::Frame {
        self.signal.next().mul_amp(self.amp_frame)
    }
}


impl<S> Signal for OffsetAmp<S>
    where S: Signal,
{
    type Frame = S::Frame;
    #[inline]
    fn next(&mut self) -> Self::Frame {
        self.signal.next().offset_amp(self.offset)
    }
}


impl<S, F> Signal for OffsetAmpPerChannel<S, F>
    where S: Signal,
          F: Frame<Sample=<<S::Frame as Frame>::Sample as Sample>::Signed,
                   NumChannels=<S::Frame as Frame>::NumChannels>,
{
    type Frame = S::Frame;
    #[inline]
    fn next(&mut self) -> Self::Frame {
        self.signal.next().add_amp(self.amp_frame)
    }
}


impl<S, M, I> Signal for MulHz<S, M, I>
    where S: Signal,
          <S::Frame as Frame>::Sample: Duplex<f64>,
          M: Signal<Frame=[f64; 1]>,
          I: Interpolator<Frame=S::Frame>,
{
    type Frame = S::Frame;
    #[inline]
    fn next(&mut self) -> Self::Frame {
        let mul = self.mul_per_frame.next()[0];
        self.signal.set_playback_hz_scale(mul);
        self.signal.next()
    }
}


impl<S> Signal for Delay<S>
    where S: Signal,
{
    type Frame = S::Frame;
    #[inline]
    fn next(&mut self) -> Self::Frame {
        if self.n_frames > 0 {
            self.n_frames -= 1;
            Self::Frame::equilibrium()
        } else {
            self.signal.next()
        }
    }
}


impl<S> IntoInterleavedSamples<S>
    where S: Signal,
{
    /// Yield the next interleaved sample from the inner `Signal`.
    #[inline]
    pub fn next_sample(&mut self) -> <S::Frame as Frame>::Sample {
        loop {
            match self.current_frame.next() {
                Some(channel) => return channel,
                None => self.current_frame = self.signal.next().channels(),
            }
        }
    }

    /// Convert the `ToInterleavedSamples` into an `Iterator`.
    #[inline]
    pub fn into_iter(self) -> IntoInterleavedSamplesIterator<S> {
        IntoInterleavedSamplesIterator {
            samples: self,
        }
    }
}

impl<S> Iterator for IntoInterleavedSamplesIterator<S>
    where S: Signal,
{
    type Item = <S::Frame as Frame>::Sample;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.samples.next_sample())
    }
}

impl<S> Clone for IntoInterleavedSamples<S>
    where S: Signal + Clone,
          <S::Frame as Frame>::Channels: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        IntoInterleavedSamples {
            signal: self.signal.clone(),
            current_frame: self.current_frame.clone(),
        }
    }
}

impl<S> Clone for IntoInterleavedSamplesIterator<S>
    where S: Signal,
          IntoInterleavedSamples<S>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        IntoInterleavedSamplesIterator {
            samples: self.samples.clone(),
        }
    }
}


impl<S> Signal for ClipAmp<S>
    where S: Signal,
{
    type Frame = S::Frame;
    #[inline]
    fn next(&mut self) -> Self::Frame {
        let f = self.signal.next();
        f.map(|s| {
            let s: <<S::Frame as Frame>::Sample as Sample>::Signed = s.to_sample();
            if s > self.thresh { self.thresh } else if s < -self.thresh { -self.thresh } else { s }
                .to_sample()
        })
    }
}


impl<S> Bus<S>
    where S: Signal,
{
    fn new(signal: S, frames_read: BTreeMap<usize, usize>) -> Self {
        Bus {
            node: Rc::new(core::cell::RefCell::new(SharedNode {
                signal: signal,
                buffer: VecDeque::new(),
                frames_read: frames_read,
                next_key: 0,
            })),
        }
    }

    /// Produce a new Output node to which the signal `S` will output its frames.
    #[inline]
    pub fn send(&self) -> Output<S> {
        let mut node = self.node.borrow_mut();

        // Get the key and increment for the next output.
        let key = node.next_key;
        node.next_key = node.next_key.wrapping_add(1);

        // Insert the number of frames read by the new output.
        let num_frames = node.buffer.len();
        node.frames_read.insert(key, num_frames);

        Output {
            key: key,
            node: self.node.clone(),
        }
    }
}

impl<S> SharedNode<S>
    where S: Signal,
{
    // Requests the next frame for the `Output` at the given key.
    //
    // If there are no frames pending for the output, a new frame will be requested from the
    // signal and appended to the ring buffer to be received by the other outputs.
    fn next_frame(&mut self, key: usize) -> S::Frame {
        let num_frames = self.buffer.len();
        let frames_read = self.frames_read
            .remove(&key)
            .expect("no frames_read for Output");

        let frame = if frames_read < num_frames {
            self.buffer[frames_read]
        } else {
            let frame = self.signal.next();
            self.buffer.push_back(frame);
            frame
        };

        // If the number of frames read by this output is the lowest, then we can pop the frame
        // from the front.
        let least_frames_read = !self.frames_read
            .values()
            .any(|&other_frames_read| other_frames_read <= frames_read);

        // If this output had read the least number of frames, pop the front frame and decrement
        // the frames read counters for each of the other outputs.
        let new_frames_read = if least_frames_read {
            self.buffer.pop_front();
            for other_frames_read in self.frames_read.values_mut() {
                *other_frames_read -= 1;
            }
            frames_read
        } else {
            frames_read + 1
        };

        self.frames_read.insert(key, new_frames_read);

        frame
    }

    #[inline]
    fn pending_frames(&self, key: usize) -> usize {
        self.buffer.len() - self.frames_read[&key]
    }

    // Drop the given output from the `Bus`.
    //
    // Called by the `Output::drop` implementation.
    fn drop_output(&mut self, key: usize) {
        self.frames_read.remove(&key);
        let least_frames_read = self
            .frames_read
            .values()
            .fold(self.buffer.len(), |a, &b| core::cmp::min(a, b));
        if least_frames_read > 0 {
            for frames_read in self.frames_read.values_mut() {
                *frames_read -= least_frames_read;
            }
            for _ in 0..least_frames_read {
                self.buffer.pop_front();
            }
        }
    }
}

impl<S> Output<S>
    where S: Signal,
{
    /// The number of frames that have been requested from the `Signal` `S` by some other `Output`
    /// that have not yet been requested by this `Output`.
    ///
    /// This is useful when using an `Output` to "monitor" some signal, allowing the user to drain
    /// only frames that have already been requested by some other `Output`.
    ///
    /// # Example
    ///
    /// ```
    /// extern crate sample;
    ///
    /// use sample::{signal, Signal};
    ///
    /// fn main() {
    ///     let frames = [[0.1], [0.2], [0.3]];
    ///     let bus = signal::from_iter(frames.iter().cloned()).bus();
    ///     let signal = bus.send();
    ///     let mut monitor = bus.send();
    ///     assert_eq!(signal.take(3).collect::<Vec<_>>(), vec![[0.1], [0.2], [0.3]]);
    ///     assert_eq!(monitor.pending_frames(), 3);
    ///     assert_eq!(monitor.next(), [0.1]);
    ///     assert_eq!(monitor.pending_frames(), 2);
    /// }
    /// ```
    #[inline]
    pub fn pending_frames(&self) -> usize {
        self.node.borrow().pending_frames(self.key)
    }
}

impl<S> Signal for Output<S>
    where S: Signal,
{
    type Frame = S::Frame;
    #[inline]
    fn next(&mut self) -> Self::Frame {
        self.node.borrow_mut().next_frame(self.key)
    }
}

impl<S> Drop for Output<S>
    where S: Signal,
{
    fn drop(&mut self) {
        self.node.borrow_mut().drop_output(self.key)
    }
}


impl<S> Iterator for Take<S>
    where S: Signal,
{
    type Item = S::Frame;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.n == 0 {
            return None;
        }
        self.n -= 1;
        Some(self.signal.next())
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.n, Some(self.n))
    }
}

impl<S> ExactSizeIterator for Take<S>
    where S: Signal,
{
    #[inline]
    fn len(&self) -> usize {
        self.n
    }
}
