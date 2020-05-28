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

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(core_intrinsics))]

#[cfg(not(feature = "std"))]
extern crate alloc;

use core;
use core::cell::RefCell;
use dasp_frame::Frame;
use dasp_interpolate::Interpolator;
use dasp_ring_buffer as ring_buffer;
use dasp_sample::{Duplex, Sample};
use interpolate::Converter;

pub mod interpolate;
mod ops;

#[cfg(features = "boxed")]
pub mod boxed;
#[cfg(feature = "bus")]
pub mod bus;
#[cfg(feature = "envelope")]
pub mod envelope;
#[cfg(feature = "rms")]
pub mod rms;
#[cfg(feature = "window")]
pub mod window;

#[cfg(not(feature = "std"))]
type Rc<T> = alloc::rc::Rc<T>;
#[cfg(feature = "std")]
type Rc<T> = std::rc::Rc<T>;

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
    /// use dasp_signal::{self as signal, Signal};
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

    /// Whether or not the signal is exhausted of meaningful frames.
    ///
    /// By default, this returns `false` and assumes that the `Signal` is infinite.
    ///
    /// As an example, `signal::FromIterator` becomes exhausted once the inner `Iterator` has been
    /// exhausted. `Sine` on the other hand will always return `false` as it will produce
    /// meaningful values infinitely.
    ///
    /// It should be rare for users to need to call this method directly, unless they are
    /// implementing their own custom `Signal`s. Instead, idiomatic code will tend toward the
    /// `Signal::until_exhasted` method which produces an `Iterator` that yields `Frame`s until
    /// `Signal::is_exhausted` returns `true`.
    ///
    /// Adaptors that source frames from more than one signal (`AddAmp`, `MulHz`, etc) will return
    /// `true` if *any* of the source signals return `true`. In this sense exhaustiveness is
    /// contagious. This can be likened to the way that `Iterator::zip` begins returning `None`
    /// when either `A` or `B` begins returning `None`.
    ///
    /// ```rust
    /// use dasp_signal::{self as signal, Signal};
    ///
    /// fn main() {
    ///     // Infinite signals always return `false`.
    ///     let sine_signal = signal::rate(44_100.0).const_hz(400.0).sine();
    ///     assert_eq!(sine_signal.is_exhausted(), false);
    ///
    ///     // Signals over iterators return `true` when the inner iterator is exhausted.
    ///     let frames = [[0.2], [-0.6], [0.4]];
    ///     let mut iter_signal = signal::from_iter(frames.iter().cloned());
    ///     assert_eq!(iter_signal.is_exhausted(), false);
    ///     iter_signal.by_ref().take(3).count();
    ///     assert_eq!(iter_signal.is_exhausted(), true);
    ///
    ///     // Adaptors return `true` when the first signal becomes exhausted.
    ///     let a = [[1], [2]];
    ///     let b = [[1], [2], [3], [4]];
    ///     let a_signal = signal::from_iter(a.iter().cloned());
    ///     let b_signal = signal::from_iter(b.iter().cloned());
    ///     let mut added = a_signal.add_amp(b_signal);
    ///     assert_eq!(added.is_exhausted(), false);
    ///     added.by_ref().take(2).count();
    ///     assert_eq!(added.is_exhausted(), true);
    /// }
    /// ```
    #[inline]
    fn is_exhausted(&self) -> bool {
        false
    }

    /// A signal that maps one set of frames to another.
    ///
    /// # Example
    ///
    /// ```rust
    /// use dasp_signal::{self as signal, Signal};
    ///
    /// fn main() {
    ///     let frames = signal::gen(|| [0.5]);
    ///     let mut mapper = frames.map(|f| [f[0], 0.25]);
    ///     assert_eq!(mapper.next(), [0.5, 0.25]);
    ///     assert_eq!(mapper.next(), [0.5, 0.25]);
    ///     assert_eq!(mapper.next(), [0.5, 0.25]);
    /// }
    /// ```
    ///
    /// This can also be useful for monitoring the peak values of a signal.
    ///
    /// ```
    /// use dasp_frame::Frame;
    /// use dasp_peak as peak;
    /// use dasp_signal::{self as signal, Signal};
    ///
    /// fn main() {
    ///     let sine_wave = signal::rate(4.0).const_hz(1.0).sine();
    ///     let mut peak = sine_wave
    ///         .map(peak::full_wave)
    ///         .map(|f| [f[0].round()]);
    ///     assert_eq!(
    ///         peak.take(4).collect::<Vec<_>>(),
    ///         vec![[0.0], [1.0], [0.0], [1.0]]
    ///     );
    /// }
    /// ```
    fn map<M, F>(self, map: M) -> Map<Self, M, F>
    where
        Self: Sized,
        M: FnMut(Self::Frame) -> F,
        F: Frame,
    {
        Map {
            signal: self,
            map: map,
            frame: core::marker::PhantomData,
        }
    }

    /// A signal that maps one set of frames to another.
    ///
    /// # Example
    ///
    /// ```rust
    /// use dasp_signal::{self as signal, Signal};
    ///
    /// fn main() {
    ///     let frames = signal::gen(|| [0.5]);
    ///     let more_frames = signal::gen(|| [0.25]);
    ///     let mut mapper = frames.zip_map(more_frames, |f, o| [f[0], o[0]]);
    ///     assert_eq!(mapper.next(), [0.5, 0.25]);
    ///     assert_eq!(mapper.next(), [0.5, 0.25]);
    ///     assert_eq!(mapper.next(), [0.5, 0.25]);
    /// }
    /// ```
    fn zip_map<O, M, F>(self, other: O, map: M) -> ZipMap<Self, O, M, F>
    where
        Self: Sized,
        M: FnMut(Self::Frame, O::Frame) -> F,
        O: Signal,
        F: Frame,
    {
        ZipMap {
            this: self,
            map: map,
            other: other,
            frame: core::marker::PhantomData,
        }
    }

    /// Provides an iterator that yields the sum of the frames yielded by both `other` and `self`
    /// in lock-step.
    ///
    /// # Example
    ///
    /// ```rust
    /// use dasp_signal::{self as signal, Signal};
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
    where
        Self: Sized,
        S: Signal,
        S::Frame: Frame<
            Sample = <<Self::Frame as Frame>::Sample as Sample>::Signed,
            NumChannels = <Self::Frame as Frame>::NumChannels,
        >,
    {
        AddAmp { a: self, b: other }
    }

    /// Provides an iterator that yields the product of the frames yielded by both `other` and
    /// `self` in lock-step.
    ///
    /// # Example
    ///
    /// ```rust
    /// use dasp_signal::{self as signal, Signal};
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
    where
        Self: Sized,
        S: Signal,
        S::Frame: Frame<
            Sample = <<Self::Frame as Frame>::Sample as Sample>::Float,
            NumChannels = <Self::Frame as Frame>::NumChannels,
        >,
    {
        MulAmp { a: self, b: other }
    }

    /// Provides an iterator that offsets the amplitude of every channel in each frame of the
    /// signal by some sample value and yields the resulting frames.
    ///
    /// # Example
    ///
    /// ```rust
    /// use dasp_signal::{self as signal, Signal};
    ///
    /// fn main() {
    ///     let frames = [[0.25, 0.4], [-0.2, -0.5]];
    ///     let signal = signal::from_iter(frames.iter().cloned());
    ///     let offset: Vec<_> = signal.offset_amp(0.5).take(2).collect();
    ///     assert_eq!(offset, vec![[0.75, 0.9], [0.3, 0.0]]);
    /// }
    /// ```
    #[inline]
    fn offset_amp(
        self,
        offset: <<Self::Frame as Frame>::Sample as Sample>::Signed,
    ) -> OffsetAmp<Self>
    where
        Self: Sized,
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
    /// use dasp_signal::{self as signal, Signal};
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
    where
        Self: Sized,
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
    /// use dasp_signal::{self as signal, Signal};
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
    where
        Self: Sized,
        F: Frame<
            Sample = <<Self::Frame as Frame>::Sample as Sample>::Signed,
            NumChannels = <Self::Frame as Frame>::NumChannels,
        >,
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
    /// use dasp_signal::{self as signal, Signal};
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
    where
        Self: Sized,
        F: Frame<
            Sample = <<Self::Frame as Frame>::Sample as Sample>::Float,
            NumChannels = <Self::Frame as Frame>::NumChannels,
        >,
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
    /// use dasp_interpolate::linear::Linear;
    /// use dasp_signal::{self as signal, Signal};
    ///
    /// fn main() {
    ///     let foo = [[0.0], [1.0], [0.0], [-1.0]];
    ///     let mul = [[1.0], [1.0], [0.5], [0.5], [0.5], [0.5]];
    ///     let mut source = signal::from_iter(foo.iter().cloned());
    ///     let a = source.next();
    ///     let b = source.next();
    ///     let interp = Linear::new(a, b);
    ///     let hz_signal = signal::from_iter(mul.iter().cloned());
    ///     let frames: Vec<_> = source.mul_hz(interp, hz_signal).take(6).collect();
    ///     assert_eq!(&frames[..], &[[0.0], [1.0], [0.0], [-0.5], [-1.0], [-0.5]][..]);
    /// }
    /// ```
    fn mul_hz<M, I>(self, interpolator: I, mul_per_frame: M) -> MulHz<Self, M, I>
    where
        Self: Sized,
        M: Signal<Frame = [f64; 1]>,
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
    /// use dasp_interpolate::linear::Linear;
    /// use dasp_signal::{self as signal, Signal};
    ///
    /// fn main() {
    ///     let foo = [[0.0], [1.0], [0.0], [-1.0]];
    ///     let mut source = signal::from_iter(foo.iter().cloned());
    ///     let a = source.next();
    ///     let b = source.next();
    ///     let interp = Linear::new(a, b);
    ///     let frames: Vec<_> = source.from_hz_to_hz(interp, 1.0, 2.0).take(8).collect();
    ///     assert_eq!(&frames[..], &[[0.0], [0.5], [1.0], [0.5], [0.0], [-0.5], [-1.0], [-0.5]][..]);
    /// }
    /// ```
    fn from_hz_to_hz<I>(self, interpolator: I, source_hz: f64, target_hz: f64) -> Converter<Self, I>
    where
        Self: Sized,
        I: Interpolator,
    {
        Converter::from_hz_to_hz(self, interpolator, source_hz, target_hz)
    }

    /// Multiplies the rate at which frames of the `Signal` are yielded by the given value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use dasp_interpolate::linear::Linear;
    /// use dasp_signal::{self as signal, Signal};
    ///
    /// fn main() {
    ///     let foo = [[0.0], [1.0], [0.0], [-1.0]];
    ///     let mut source = signal::from_iter(foo.iter().cloned());
    ///     let a = source.next();
    ///     let b = source.next();
    ///     let interp = Linear::new(a, b);
    ///     let frames: Vec<_> = source.scale_hz(interp, 0.5).take(8).collect();
    ///     assert_eq!(&frames[..], &[[0.0], [0.5], [1.0], [0.5], [0.0], [-0.5], [-1.0], [-0.5]][..]);
    /// }
    /// ```
    fn scale_hz<I>(self, interpolator: I, multi: f64) -> Converter<Self, I>
    where
        Self: Sized,
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
    /// use dasp_signal::{self as signal, Signal};
    ///
    /// fn main() {
    ///     let frames = [[0.2], [0.4]];
    ///     let signal = signal::from_iter(frames.iter().cloned());
    ///     let delayed: Vec<_> = signal.delay(2).take(4).collect();
    ///     assert_eq!(delayed, vec![[0.0], [0.0], [0.2], [0.4]]);
    /// }
    /// ```
    fn delay(self, n_frames: usize) -> Delay<Self>
    where
        Self: Sized,
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
    /// use dasp_signal::{self as signal, Signal};
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
    where
        Self: Sized,
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
    /// use dasp_signal::{self as signal, Signal};
    ///
    /// fn main() {
    ///     let frames = [[1.2, 0.8], [-0.7, -1.4]];
    ///     let signal = signal::from_iter(frames.iter().cloned());
    ///     let clipped: Vec<_> = signal.clip_amp(0.9).take(2).collect();
    ///     assert_eq!(clipped, vec![[0.9, 0.8], [-0.7, -0.9]]);
    /// }
    /// ```
    fn clip_amp(self, thresh: <<Self::Frame as Frame>::Sample as Sample>::Signed) -> ClipAmp<Self>
    where
        Self: Sized,
    {
        ClipAmp {
            signal: self,
            thresh: thresh,
        }
    }

    /// Create a new `Signal` that calls the enclosing function on each iteration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use dasp_signal::{self as signal, Signal};
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
    ///     let mut inspected = signal.inspect(func);
    ///     let out = inspected.next();
    ///     assert_eq!(out, [0.1]);
    /// }
    /// ```
    fn inspect<F>(self, inspect: F) -> Inspect<Self, F>
    where
        Self: Sized,
        F: FnMut(&Self::Frame),
    {
        Inspect {
            signal: self,
            inspect: inspect,
        }
    }

    /// Forks `Self` into two signals that produce the same frames.
    ///
    /// The given `ring_buffer` must be empty to ensure correct behaviour.
    ///
    /// Each time a frame is requested from the signal on one branch, that frame will be pushed to
    /// the given `ring_buffer` of pending frames to be collected by the other branch and a flag
    /// will be set to indicate that there are pending frames.
    ///
    /// **Fork** can be used to share the queue between the two branches by reference
    /// `fork.by_ref()` or via a reference counted pointer `fork.by_rc()`.
    ///
    /// **Fork** is a slightly more efficient alternative to **Bus** when only two branches are
    /// required.
    ///
    /// **Note:** It is up to the user to ensure that there are never more than
    /// `ring_buffer.max_len()` pending frames - otherwise the oldest frames will be overridden and
    /// glitching may occur on the lagging branch.
    ///
    /// **Panic!**s if the given `ring_buffer` is not empty in order to guarantee correct
    /// behaviour.
    ///
    /// ```
    /// use dasp_ring_buffer as ring_buffer;
    /// use dasp_signal::{self as signal, Signal};
    ///
    /// fn main() {
    ///     let signal = signal::rate(44_100.0).const_hz(440.0).sine();
    ///     let ring_buffer = ring_buffer::Bounded::<[[f64; 1]; 64]>::array();
    ///     let mut fork = signal.fork(ring_buffer);
    ///
    ///     // Forks can be split into their branches via reference.
    ///     {
    ///         let (mut a, mut b) = fork.by_ref();
    ///         assert_eq!(a.next(), b.next());
    ///         assert_eq!(a.by_ref().take(64).collect::<Vec<_>>(),
    ///                    b.by_ref().take(64).collect::<Vec<_>>());
    ///     }
    ///
    ///     // Forks can also be split via reference counted pointer.
    ///     let (mut a, mut b) = fork.by_rc();
    ///     assert_eq!(a.next(), b.next());
    ///     assert_eq!(a.by_ref().take(64).collect::<Vec<_>>(),
    ///                b.by_ref().take(64).collect::<Vec<_>>());
    ///
    ///     // The lagging branch will be missing frames if we exceed `ring_buffer.max_len()`
    ///     // pending frames.
    ///     assert!(a.by_ref().take(67).collect::<Vec<_>>() !=
    ///             b.by_ref().take(67).collect::<Vec<_>>())
    /// }
    /// ```
    fn fork<S>(self, ring_buffer: ring_buffer::Bounded<S>) -> Fork<Self, S>
    where
        Self: Sized,
        S: ring_buffer::SliceMut<Element = Self::Frame>,
    {
        assert!(ring_buffer.is_empty());
        let shared = ForkShared {
            signal: self,
            ring_buffer: ring_buffer,
            pending: Fork::<Self, S>::B,
        };
        Fork {
            shared: RefCell::new(shared),
        }
    }

    /// Converts the `Signal` into an `Iterator` that will yield the given number for `Frame`s
    /// before returning `None`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use dasp_signal::{self as signal, Signal};
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
    where
        Self: Sized,
    {
        Take { signal: self, n: n }
    }

    /// Converts the `Signal` into an `Iterator` yielding frames until the `signal.is_exhausted()`
    /// returns `true`.
    ///
    /// # Example
    ///
    /// ```
    /// use dasp_signal::{self as signal, Signal};
    ///
    /// fn main() {
    ///     let frames = [[1], [2]];
    ///     let signal = signal::from_iter(frames.iter().cloned());
    ///     assert_eq!(signal.until_exhausted().count(), 2);
    /// }
    /// ```
    fn until_exhausted(self) -> UntilExhausted<Self>
    where
        Self: Sized,
    {
        UntilExhausted { signal: self }
    }

    /// Buffers the signal using the given ring buffer.
    ///
    /// When `next` is called on the returned signal, it will first check if the ring buffer is
    /// empty. If so, it will completely fill the ring buffer with the inner signal before yielding
    /// the next value. If the ring buffer still contains un-yielded values, the next frame will be
    /// popped from the front of the ring buffer and immediately returned.
    ///
    /// ```
    /// use dasp_ring_buffer as ring_buffer;
    /// use dasp_signal::{self as signal, Signal};
    ///
    /// fn main() {
    ///     let frames = [[0.1], [0.2], [0.3], [0.4]];
    ///     let signal = signal::from_iter(frames.iter().cloned());
    ///     let ring_buffer = ring_buffer::Bounded::<[[f32; 1]; 2]>::array();
    ///     let mut buffered_signal = signal.buffered(ring_buffer);
    ///     assert_eq!(buffered_signal.next(), [0.1]);
    ///     assert_eq!(buffered_signal.next(), [0.2]);
    ///     assert_eq!(buffered_signal.next(), [0.3]);
    ///     assert_eq!(buffered_signal.next(), [0.4]);
    ///     assert_eq!(buffered_signal.next(), [0.0]);
    /// }
    /// ```
    ///
    /// If the given ring buffer already contains frames, those will be yielded first.
    ///
    /// ```
    /// use dasp_ring_buffer as ring_buffer;
    /// use dasp_signal::{self as signal, Signal};
    ///
    /// fn main() {
    ///     let frames = [[0.1], [0.2], [0.3], [0.4]];
    ///     let signal = signal::from_iter(frames.iter().cloned());
    ///     let ring_buffer = ring_buffer::Bounded::from_full([[0.8], [0.9]]);
    ///     let mut buffered_signal = signal.buffered(ring_buffer);
    ///     assert_eq!(buffered_signal.next(), [0.8]);
    ///     assert_eq!(buffered_signal.next(), [0.9]);
    ///     assert_eq!(buffered_signal.next(), [0.1]);
    ///     assert_eq!(buffered_signal.next(), [0.2]);
    ///     assert_eq!(buffered_signal.next(), [0.3]);
    ///     assert_eq!(buffered_signal.next(), [0.4]);
    ///     assert_eq!(buffered_signal.next(), [0.0]);
    /// }
    /// ```
    fn buffered<S>(self, ring_buffer: ring_buffer::Bounded<S>) -> Buffered<Self, S>
    where
        Self: Sized,
        S: ring_buffer::Slice<Element = Self::Frame> + ring_buffer::SliceMut,
    {
        Buffered {
            signal: self,
            ring_buffer: ring_buffer,
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
    /// use dasp_signal::{self as signal, Signal};
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
    fn by_ref(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }
}

/// Consumes the given `Iterator`, converts it to a `Signal`, applies the given function to the
/// `Signal` and returns an `Iterator` that will become exhausted when the consumed `Iterator`
/// does.
///
/// This is particularly useful when you want to apply `Signal` methods to an `Iterator` yielding
/// `Frame`s and return an `Iterator` as a result.
///
/// # Example
///
/// ```
/// use dasp_signal::{self as signal, Signal};
///
/// fn main() {
///     let frames = vec![[0], [1], [2], [3]];
///     let offset_frames = signal::lift(frames, |signal| signal.offset_amp(2));
///     assert_eq!(offset_frames.collect::<Vec<_>>(), vec![[2], [3], [4], [5]]);
/// }
/// ```
pub fn lift<I, F, S>(iter: I, f: F) -> UntilExhausted<S>
where
    I: IntoIterator,
    I::Item: Frame,
    F: FnOnce(FromIterator<I::IntoIter>) -> S,
    S: Signal<Frame = I::Item>,
{
    let iter = iter.into_iter();
    let signal = from_iter(iter);
    let new_signal = f(signal);
    new_signal.until_exhausted()
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

/// A signal that maps from one signal to another
#[derive(Clone)]
pub struct Map<S, M, F> {
    signal: S,
    map: M,
    frame: core::marker::PhantomData<F>,
}

/// A signal that iterates two signals in parallel and combines them with a function.
///
/// `ZipMap::is_exhausted` returns `true` if *either* of the two signals returns `true`.
#[derive(Clone)]
pub struct ZipMap<S, O, M, F> {
    this: S,
    other: O,
    map: M,
    frame: core::marker::PhantomData<F>,
}

/// A type that wraps an Iterator and provides a `Signal` implementation for it.
#[derive(Clone)]
pub struct FromIterator<I>
where
    I: Iterator,
{
    iter: I,
    next: Option<I::Item>,
}

/// An iterator that converts an iterator of `Sample`s to an iterator of `Frame`s.
#[derive(Clone)]
pub struct FromInterleavedSamplesIterator<I, F>
where
    I: Iterator,
    I::Item: Sample,
    F: Frame<Sample = I::Item>,
{
    samples: I,
    next: Option<F>,
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
pub struct Hz<S> {
    hz: S,
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
where
    S: Signal,
{
    signal: S,
    offset: <<S::Frame as Frame>::Sample as Sample>::Signed,
}

/// An `Iterator` that scales the amplitude of the sample of each channel in every `Frame` yielded
/// by `self` by the given amplitude.
#[derive(Clone)]
pub struct ScaleAmp<S>
where
    S: Signal,
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
where
    S: Signal,
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

/// A signal that calls its enclosing function and returns the original value. The signal may
/// mutate state.
#[derive(Clone)]
pub struct Inspect<S, F> {
    signal: S,
    inspect: F,
}

/// Converts a `Signal` to a type that yields the individual interleaved samples.
pub struct IntoInterleavedSamples<S>
where
    S: Signal,
{
    signal: S,
    current_frame: <S::Frame as Frame>::Channels,
}

/// Converts the `IntoInterleavedSamples` into an `Iterator` that always returns `Some`.
pub struct IntoInterleavedSamplesIterator<S>
where
    S: Signal,
{
    samples: IntoInterleavedSamples<S>,
}

/// Yields frames from the signal until the `signal.is_exhausted()` returns `true`.
#[derive(Clone)]
pub struct UntilExhausted<S>
where
    S: Signal,
{
    signal: S,
}

/// Clips samples in each frame yielded by `signal` to the given threshhold amplitude.
#[derive(Clone)]
pub struct ClipAmp<S>
where
    S: Signal,
{
    signal: S,
    thresh: <<S::Frame as Frame>::Sample as Sample>::Signed,
}

/// Represents a forked `Signal` that has not yet been split into its two branches.
///
/// A `Fork` can be split into its two branches via either of the following methods:
///
/// - `fork.by_rc()`: consumes self and shares the fork via `Rc<RefCell>`.
/// - `fork.by_ref()`: borrows self and shares the fork via `&RefCell`.
#[derive(Clone)]
pub struct Fork<S, D> {
    shared: RefCell<ForkShared<S, D>>,
}

#[derive(Clone)]
struct ForkShared<S, D> {
    signal: S,
    ring_buffer: ring_buffer::Bounded<D>,
    pending: bool,
}

impl<S, D> Fork<S, D> {
    const A: bool = true;
    const B: bool = false;

    /// Consumes the `Fork` and returns two branches that share the signal and inner ring buffer
    /// via a reference countered pointer (`Rc`).
    ///
    /// Note: This requires dynamical allocation as `Rc<RefCell<Self>>` is used to share the signal
    /// and ring buffer. A user may avoid this dynamic allocation by using the `Fork::by_ref`
    /// method instead, however this comes with the ergonomic cost of bounding the lifetime of the
    /// branches to the lifetime of the fork.
    /// `Fork::by_ref`
    pub fn by_rc(self) -> (BranchRcA<S, D>, BranchRcB<S, D>) {
        let Fork { shared } = self;
        let shared_fork = Rc::new(shared);
        let a = BranchRcA {
            shared_fork: shared_fork.clone(),
        };
        let b = BranchRcB {
            shared_fork: shared_fork,
        };
        (a, b)
    }

    /// Mutably borrows the `Fork` and returns two branches that share the signal and inner ring
    /// buffer via reference.
    ///
    /// This is more efficient than `Fork::by_rc` as it does not require `Rc`, however it may be
    /// less ergonomic in some cases as the returned branches are bound to the lifetime of `Fork`.
    pub fn by_ref(&mut self) -> (BranchRefA<S, D>, BranchRefB<S, D>) {
        let Fork { ref shared } = *self;
        let a = BranchRefA {
            shared_fork: shared,
        };
        let b = BranchRefB {
            shared_fork: shared,
        };
        (a, b)
    }
}

// A macro to simplify the boilerplate shared between the two branch types returned by `Fork`.
macro_rules! define_branch {
    ($TRc:ident, $TRef:ident, $SELF:ident, $OTHER:ident) => {
        /// One of the two `Branch` signals returned by `Fork::by_rc`.
        pub struct $TRc<S, D> {
            shared_fork: Rc<RefCell<ForkShared<S, D>>>,
        }

        /// One of the two `Branch` signals returned by `Fork::by_ref`.
        pub struct $TRef<'a, S: 'a, D: 'a> {
            shared_fork: &'a RefCell<ForkShared<S, D>>,
        }

        impl<S, D> Signal for $TRc<S, D>
        where
            S: Signal,
            D: ring_buffer::SliceMut<Element = S::Frame>,
        {
            type Frame = S::Frame;
            fn next(&mut self) -> Self::Frame {
                let mut fork = self.shared_fork.borrow_mut();
                if fork.pending == Fork::<S, D>::$SELF {
                    if let Some(frame) = fork.ring_buffer.pop() {
                        return frame;
                    }
                    fork.pending = Fork::<S, D>::$OTHER;
                }
                let frame = fork.signal.next();
                fork.ring_buffer.push(frame);
                frame
            }
        }

        impl<'a, S, D> Signal for $TRef<'a, S, D>
        where
            S: 'a + Signal,
            D: 'a + ring_buffer::SliceMut<Element = S::Frame>,
        {
            type Frame = S::Frame;
            fn next(&mut self) -> Self::Frame {
                let mut fork = self.shared_fork.borrow_mut();
                if fork.pending == Fork::<S, D>::$SELF {
                    if let Some(frame) = fork.ring_buffer.pop() {
                        return frame;
                    }
                    fork.pending = Fork::<S, D>::$OTHER;
                }
                let frame = fork.signal.next();
                fork.ring_buffer.push(frame);
                frame
            }
        }

        impl<S, D> $TRc<S, D>
        where
            D: ring_buffer::Slice,
            D::Element: Copy,
        {
            /// The number of frames that are pending collection by this branch.
            pub fn pending_frames(&self) -> usize {
                let fork = self.shared_fork.borrow();
                if fork.pending == Fork::<S, D>::$SELF {
                    fork.ring_buffer.len()
                } else {
                    0
                }
            }
        }

        impl<'a, S, D> $TRef<'a, S, D>
        where
            D: ring_buffer::Slice,
            D::Element: Copy,
        {
            /// The number of frames that are pending collection by this branch.
            pub fn pending_frames(&self) -> usize {
                let fork = self.shared_fork.borrow();
                if fork.pending == Fork::<S, D>::$SELF {
                    fork.ring_buffer.len()
                } else {
                    0
                }
            }
        }
    };
}

define_branch!(BranchRcA, BranchRefA, A, B);
define_branch!(BranchRcB, BranchRefB, B, A);

/// An iterator that yields `n` number of `Frame`s from the inner `signal`.
#[derive(Clone)]
pub struct Take<S>
where
    S: Signal,
{
    signal: S,
    n: usize,
}

/// Buffers the signal using the given ring buffer.
///
/// When `next` is called, `Buffered` will first check if the ring buffer is empty. If so, it will
/// completely fill the ring buffer with `signal` before yielding the next frame.
///
/// If `next` is called and the ring buffer still contains un-yielded values, the next frame will
/// be popped from the front of the ring buffer and immediately returned.
#[derive(Clone)]
pub struct Buffered<S, D> {
    signal: S,
    ring_buffer: ring_buffer::Bounded<D>,
}

/// An iterator that pops elements from the inner bounded ring buffer and yields them.
///
/// Returns `None` once the inner ring buffer is exhausted.
pub struct BufferedFrames<'a, D: 'a> {
    ring_buffer: &'a mut ring_buffer::Bounded<D>,
}

///// Signal Constructors

/// Provides an iterator that endlessly yields `Frame`s of type `F` at equilibrium.
///
/// # Example
///
/// ```rust
/// use dasp_signal::{self as signal, Signal};
///
/// fn main() {
///     let equilibrium: Vec<[f32; 1]> = signal::equilibrium().take(4).collect();
///     assert_eq!(equilibrium, vec![[0.0], [0.0], [0.0], [0.0]]);
///
///     let equilibrium: Vec<[u8; 2]> = signal::equilibrium().take(3).collect();
///     assert_eq!(equilibrium, vec![[128, 128], [128, 128], [128, 128]]);
/// }
/// ```
pub fn equilibrium<F>() -> Equilibrium<F>
where
    F: Frame,
{
    Equilibrium {
        frame: core::marker::PhantomData,
    }
}

/// A signal that generates frames using the given function.
///
/// The resulting signal is assumed to be infinite and `is_exhausted` will always return `false`.
/// To create an exhaustive signal first create an `Iterator` and then use `from_iter`.
///
/// # Example
///
/// ```rust
/// use dasp_signal::{self as signal, Signal};
///
/// fn main() {
///     let mut frames = signal::gen(|| [0.5]);
///     assert_eq!(frames.next(), [0.5]);
///     assert_eq!(frames.next(), [0.5]);
///     assert_eq!(frames.next(), [0.5]);
/// }
/// ```
pub fn gen<G, F>(gen: G) -> Gen<G, F>
where
    G: Fn() -> F,
    F: Frame,
{
    Gen {
        gen: gen,
        frame: core::marker::PhantomData,
    }
}

/// A signal that generates frames using the given function which may mutate some state.
///
/// The resulting signal is assumed to be infinite and `is_exhausted` will always return `false`.
/// To create an exhaustive signal first create an `Iterator` and then use `from_iter`.
///
/// # Example
///
/// ```rust
/// use dasp_signal::{self as signal, Signal};
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
where
    G: FnMut() -> F,
    F: Frame,
{
    GenMut {
        gen_mut: gen_mut,
        frame: core::marker::PhantomData,
    }
}

/// Create a new `Signal` from the given `Frame`-yielding `Iterator`.
///
/// When the `Iterator` is exhausted, the new `Signal` will yield `F::equilibrium`.
///
/// Note that `Iterator::next` will be called immediately so that `FromIterator` can store the next
/// pending frame and efficiently test for exhaustiveness.
///
/// # Example
///
/// ```rust
/// use dasp_signal::{self as signal, Signal};
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
where
    I: IntoIterator,
    I::Item: Frame,
{
    let mut iter = frames.into_iter();
    let next = iter.next();
    FromIterator {
        iter: iter,
        next: next,
    }
}

/// Create a new `Signal` from the given `Frame`-yielding `Iterator`.
///
/// When the `Iterator` is exhausted, the new `Signal` will yield `F::equilibrium`.
///
/// # Example
///
/// ```rust
/// use dasp_signal::{self as signal, Signal};
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
pub fn from_interleaved_samples_iter<I, F>(
    samples: I,
) -> FromInterleavedSamplesIterator<I::IntoIter, F>
where
    I: IntoIterator,
    I::Item: Sample,
    F: Frame<Sample = I::Item>,
{
    let mut samples = samples.into_iter();
    let next = Frame::from_samples(&mut samples);
    FromInterleavedSamplesIterator {
        samples: samples,
        next: next,
    }
}

/// Creates a `Phase` that continuously steps forward by the given `step` size yielder.
///
/// # Example
///
/// ```rust
/// use dasp_signal::{self as signal, Signal};
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
where
    S: Step,
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
/// use dasp_signal::{self as signal, Signal};
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
/// use dasp_signal::{self as signal, Signal};
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
/// use dasp_signal::{self as signal, Signal};
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
/// use dasp_signal::{self as signal, Signal};
///
/// fn main() {
///     let mut noise = signal::noise(0);
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
/// use dasp_signal::{self as signal, Signal};
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

impl<'a, S> Signal for &'a mut S
where
    S: Signal + ?Sized,
{
    type Frame = S::Frame;

    #[inline]
    fn next(&mut self) -> Self::Frame {
        (**self).next()
    }

    #[inline]
    fn is_exhausted(&self) -> bool {
        (**self).is_exhausted()
    }
}

impl<I> Signal for FromIterator<I>
where
    I: Iterator,
    I::Item: Frame,
{
    type Frame = I::Item;

    #[inline]
    fn next(&mut self) -> Self::Frame {
        match self.next.take() {
            Some(frame) => {
                self.next = self.iter.next();
                frame
            }
            None => Frame::equilibrium(),
        }
    }

    #[inline]
    fn is_exhausted(&self) -> bool {
        self.next.is_none()
    }
}

impl<I, F> Signal for FromInterleavedSamplesIterator<I, F>
where
    I: Iterator,
    I::Item: Sample,
    F: Frame<Sample = I::Item>,
{
    type Frame = F;

    #[inline]
    fn next(&mut self) -> Self::Frame {
        match self.next.take() {
            Some(frame) => {
                self.next = F::from_samples(&mut self.samples);
                frame
            }
            None => F::equilibrium(),
        }
    }

    #[inline]
    fn is_exhausted(&self) -> bool {
        self.next.is_none()
    }
}

impl<F> Signal for Equilibrium<F>
where
    F: Frame,
{
    type Frame = F;

    #[inline]
    fn next(&mut self) -> Self::Frame {
        F::equilibrium()
    }
}

impl<G, F> Signal for Gen<G, F>
where
    G: Fn() -> F,
    F: Frame,
{
    type Frame = F;

    #[inline]
    fn next(&mut self) -> Self::Frame {
        (self.gen)()
    }
}

impl<G, F> Signal for GenMut<G, F>
where
    G: FnMut() -> F,
    F: Frame,
{
    type Frame = F;

    #[inline]
    fn next(&mut self) -> Self::Frame {
        (self.gen_mut)()
    }
}

impl<S, M, F> Signal for Map<S, M, F>
where
    S: Signal,
    M: FnMut(S::Frame) -> F,
    F: Frame,
{
    type Frame = F;

    #[inline]
    fn next(&mut self) -> Self::Frame {
        (self.map)(self.signal.next())
    }

    fn is_exhausted(&self) -> bool {
        self.signal.is_exhausted()
    }
}

impl<S, O, M, F> Signal for ZipMap<S, O, M, F>
where
    S: Signal,
    O: Signal,
    M: FnMut(S::Frame, O::Frame) -> F,
    F: Frame,
{
    type Frame = F;

    #[inline]
    fn next(&mut self) -> Self::Frame {
        (self.map)(self.this.next(), self.other.next())
    }

    fn is_exhausted(&self) -> bool {
        self.this.is_exhausted() || self.other.is_exhausted()
    }
}

impl<S> Signal for Hz<S>
where
    S: Signal<Frame = [f64; 1]>,
{
    type Frame = [f64; 1];

    #[inline]
    fn next(&mut self) -> Self::Frame {
        [self.step()]
    }

    #[inline]
    fn is_exhausted(&self) -> bool {
        self.hz.is_exhausted()
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
where
    S: Step,
{
    type Frame = [f64; 1];

    #[inline]
    fn next(&mut self) -> Self::Frame {
        [self.next_phase()]
    }
}

impl<S> Signal for Sine<S>
where
    S: Step,
{
    type Frame = [f64; 1];

    #[inline]
    fn next(&mut self) -> Self::Frame {
        const PI_2: f64 = core::f64::consts::PI * 2.0;
        let phase = self.phase.next_phase();
        [ops::f64::sin(PI_2 * phase)]
    }
}

impl<S> Signal for Saw<S>
where
    S: Step,
{
    type Frame = [f64; 1];

    #[inline]
    fn next(&mut self) -> Self::Frame {
        let phase = self.phase.next_phase();
        [phase * -2.0 + 1.0]
    }
}

impl<S> Signal for Square<S>
where
    S: Step,
{
    type Frame = [f64; 1];

    #[inline]
    fn next(&mut self) -> Self::Frame {
        let phase = self.phase.next_phase();
        [if phase < 0.5 { 1.0 } else { -1.0 }]
    }
}

impl Rate {
    /// Create a `ConstHz` signal which consistently yields `hz / rate`.
    pub fn const_hz(self, hz: f64) -> ConstHz {
        ConstHz { step: hz / self.hz }
    }

    /// Create a `Hz` signal which yields phase step sizes controlled by an input
    /// signal `hz`.
    ///
    /// # Example
    ///
    /// ``` rust
    /// use dasp_signal::{self as signal, Signal};
    ///
    /// fn main() {
    ///     let step = signal::rate(4.0).hz(signal::gen(|| [1.0]));
    ///     let mut phase = signal::phase(step);
    ///     assert_eq!(phase.next(), [0.0]);
    ///     assert_eq!(phase.next(), [0.25]);
    ///     assert_eq!(phase.next(), [0.5]);
    ///     assert_eq!(phase.next(), [0.75]);
    ///     assert_eq!(phase.next(), [0.0]);
    ///     assert_eq!(phase.next(), [0.25]);
    /// }
    /// ```
    pub fn hz<S>(self, hz: S) -> Hz<S>
    where
        S: Signal<Frame = [f64; 1]>,
    {
        Hz { hz: hz, rate: self }
    }
}

impl<S> Hz<S>
where
    S: Signal<Frame = [f64; 1]>,
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
where
    S: Signal<Frame = [f64; 1]>,
{
    #[inline]
    fn step(&mut self) -> f64 {
        let hz = self.hz.next()[0];
        hz / self.rate.hz
    }
}

impl<S> Phase<S>
where
    S: Step,
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
            1.0 - (x
                .wrapping_mul(
                    x.wrapping_mul(x)
                        .wrapping_mul(PRIME_1)
                        .wrapping_add(PRIME_2),
                )
                .wrapping_add(PRIME_3)
                & 0x7fffffff) as f64
                / 1_073_741_824.0
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
where
    S: Step,
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
                215, 61, 156, 180,
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
                if (h & 8) != 0 {
                    grad = -grad;
                }
                // Multiply the gradient with the distance.
                grad * x
            }

            // Corners coordinates (nearest integer values).
            let i0 = ops::f64::floor(x) as i64;
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
where
    S: Step,
{
    type Frame = [f64; 1];

    #[inline]
    fn next(&mut self) -> Self::Frame {
        [self.next_sample()]
    }
}

impl<A, B> Signal for AddAmp<A, B>
where
    A: Signal,
    B: Signal,
    B::Frame: Frame<
        Sample = <<A::Frame as Frame>::Sample as Sample>::Signed,
        NumChannels = <A::Frame as Frame>::NumChannels,
    >,
{
    type Frame = A::Frame;

    #[inline]
    fn next(&mut self) -> Self::Frame {
        self.a.next().add_amp(self.b.next())
    }

    #[inline]
    fn is_exhausted(&self) -> bool {
        self.a.is_exhausted() || self.b.is_exhausted()
    }
}

impl<A, B> Signal for MulAmp<A, B>
where
    A: Signal,
    B: Signal,
    B::Frame: Frame<
        Sample = <<A::Frame as Frame>::Sample as Sample>::Float,
        NumChannels = <A::Frame as Frame>::NumChannels,
    >,
{
    type Frame = A::Frame;

    #[inline]
    fn next(&mut self) -> Self::Frame {
        self.a.next().mul_amp(self.b.next())
    }

    #[inline]
    fn is_exhausted(&self) -> bool {
        self.a.is_exhausted() || self.b.is_exhausted()
    }
}

impl<S> Signal for ScaleAmp<S>
where
    S: Signal,
{
    type Frame = S::Frame;

    #[inline]
    fn next(&mut self) -> Self::Frame {
        self.signal.next().scale_amp(self.amp)
    }

    #[inline]
    fn is_exhausted(&self) -> bool {
        self.signal.is_exhausted()
    }
}

impl<S, F> Signal for ScaleAmpPerChannel<S, F>
where
    S: Signal,
    F: Frame<
        Sample = <<S::Frame as Frame>::Sample as Sample>::Float,
        NumChannels = <S::Frame as Frame>::NumChannels,
    >,
{
    type Frame = S::Frame;

    #[inline]
    fn next(&mut self) -> Self::Frame {
        self.signal.next().mul_amp(self.amp_frame)
    }

    #[inline]
    fn is_exhausted(&self) -> bool {
        self.signal.is_exhausted()
    }
}

impl<S> Signal for OffsetAmp<S>
where
    S: Signal,
{
    type Frame = S::Frame;

    #[inline]
    fn next(&mut self) -> Self::Frame {
        self.signal.next().offset_amp(self.offset)
    }

    #[inline]
    fn is_exhausted(&self) -> bool {
        self.signal.is_exhausted()
    }
}

impl<S, F> Signal for OffsetAmpPerChannel<S, F>
where
    S: Signal,
    F: Frame<
        Sample = <<S::Frame as Frame>::Sample as Sample>::Signed,
        NumChannels = <S::Frame as Frame>::NumChannels,
    >,
{
    type Frame = S::Frame;

    #[inline]
    fn next(&mut self) -> Self::Frame {
        self.signal.next().add_amp(self.amp_frame)
    }

    #[inline]
    fn is_exhausted(&self) -> bool {
        self.signal.is_exhausted()
    }
}

impl<S, M, I> Signal for MulHz<S, M, I>
where
    S: Signal,
    <S::Frame as Frame>::Sample: Duplex<f64>,
    M: Signal<Frame = [f64; 1]>,
    I: Interpolator<Frame = S::Frame>,
{
    type Frame = S::Frame;

    #[inline]
    fn next(&mut self) -> Self::Frame {
        let mul = self.mul_per_frame.next()[0];
        self.signal.set_playback_hz_scale(mul);
        self.signal.next()
    }

    #[inline]
    fn is_exhausted(&self) -> bool {
        self.signal.is_exhausted() || self.mul_per_frame.is_exhausted()
    }
}

impl<S> Signal for Delay<S>
where
    S: Signal,
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

    #[inline]
    fn is_exhausted(&self) -> bool {
        self.n_frames == 0 && self.signal.is_exhausted()
    }
}

impl<S, F> Signal for Inspect<S, F>
where
    S: Signal,
    F: FnMut(&S::Frame),
{
    type Frame = S::Frame;

    #[inline]
    fn next(&mut self) -> Self::Frame {
        let out = self.signal.next();
        (self.inspect)(&out);
        out
    }

    #[inline]
    fn is_exhausted(&self) -> bool {
        self.signal.is_exhausted()
    }
}

impl<S> IntoInterleavedSamples<S>
where
    S: Signal,
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
        IntoInterleavedSamplesIterator { samples: self }
    }
}

impl<S> Iterator for IntoInterleavedSamplesIterator<S>
where
    S: Signal,
{
    type Item = <S::Frame as Frame>::Sample;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.samples.next_sample())
    }
}

impl<S> Iterator for UntilExhausted<S>
where
    S: Signal,
{
    type Item = S::Frame;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.signal.is_exhausted() {
            return None;
        }
        Some(self.signal.next())
    }
}

impl<S> Clone for IntoInterleavedSamples<S>
where
    S: Signal + Clone,
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
where
    S: Signal,
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
where
    S: Signal,
{
    type Frame = S::Frame;

    #[inline]
    fn next(&mut self) -> Self::Frame {
        let f = self.signal.next();
        f.map(|s| {
            let s: <<S::Frame as Frame>::Sample as Sample>::Signed = s.to_sample();
            if s > self.thresh {
                self.thresh
            } else if s < -self.thresh {
                -self.thresh
            } else {
                s
            }
            .to_sample()
        })
    }

    #[inline]
    fn is_exhausted(&self) -> bool {
        self.signal.is_exhausted()
    }
}

impl<S> Iterator for Take<S>
where
    S: Signal,
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
where
    S: Signal,
{
    #[inline]
    fn len(&self) -> usize {
        self.n
    }
}

impl<S, D> Buffered<S, D>
where
    S: Signal,
    D: ring_buffer::Slice<Element = S::Frame> + ring_buffer::SliceMut,
{
    /// Produces an iterator yielding the next batch of buffered frames.
    ///
    /// The returned iterator returns `None` once the inner ring buffer becomes exhausted.
    ///
    /// If the inner ring buffer is empty when this method is called, the ring buffer will first be
    /// filled using `Buffered`'s inner `signal` before `BufferedFrames` is returned.
    ///
    /// ```
    /// use dasp_ring_buffer as ring_buffer;
    /// use dasp_signal::{self as signal, Signal};
    ///
    /// fn main() {
    ///     let frames = [[0.1], [0.2], [0.3], [0.4]];
    ///     let signal = signal::from_iter(frames.iter().cloned());
    ///     let ring_buffer = ring_buffer::Bounded::<[[f32; 1]; 2]>::array();
    ///     let mut buffered_signal = signal.buffered(ring_buffer);
    ///     assert_eq!(buffered_signal.next_frames().collect::<Vec<_>>(), vec![[0.1], [0.2]]);
    ///     assert_eq!(buffered_signal.next_frames().collect::<Vec<_>>(), vec![[0.3], [0.4]]);
    ///     assert_eq!(buffered_signal.next_frames().collect::<Vec<_>>(), vec![[0.0], [0.0]]);
    /// }
    /// ```
    pub fn next_frames(&mut self) -> BufferedFrames<D> {
        let Buffered {
            ref mut signal,
            ref mut ring_buffer,
        } = *self;
        if ring_buffer.len() == 0 {
            for _ in 0..ring_buffer.max_len() {
                ring_buffer.push(signal.next());
            }
        }
        BufferedFrames {
            ring_buffer: ring_buffer,
        }
    }

    /// Consumes the `Buffered` signal and returns its inner signal `S` and bounded ring buffer.
    pub fn into_parts(self) -> (S, ring_buffer::Bounded<D>) {
        let Buffered {
            signal,
            ring_buffer,
        } = self;
        (signal, ring_buffer)
    }
}

impl<S, D> Signal for Buffered<S, D>
where
    S: Signal,
    D: ring_buffer::Slice<Element = S::Frame> + ring_buffer::SliceMut,
{
    type Frame = S::Frame;

    fn next(&mut self) -> Self::Frame {
        let Buffered {
            ref mut signal,
            ref mut ring_buffer,
        } = *self;
        loop {
            match ring_buffer.pop() {
                Some(frame) => return frame,
                None => {
                    for _ in 0..ring_buffer.max_len() {
                        ring_buffer.push(signal.next());
                    }
                }
            }
        }
    }

    fn is_exhausted(&self) -> bool {
        self.ring_buffer.len() == 0 && self.signal.is_exhausted()
    }
}

impl<'a, D> Iterator for BufferedFrames<'a, D>
where
    D: ring_buffer::SliceMut,
    D::Element: Copy,
{
    type Item = D::Element;
    fn next(&mut self) -> Option<Self::Item> {
        self.ring_buffer.pop()
    }
}
