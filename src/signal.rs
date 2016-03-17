//! Use the [**Signal** trait](./trait.Signal.html) for working with **Iterator**s that yield
//! **Frame**s. To complement the **Iterator** trait, **Signal** provides methods for adding,
//! scaling, offsetting, multiplying, clipping and generating frame iterators and more.
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
//! - [from_interleaved_samples](./fn.from_interleaved_samples.html) for converting an iterator yielding samples to an
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

    /// Moves the `Signal` into a `Bus` from which its output may be divided into multiple other
    /// `Signal`s in the form of `Output`s.
    ///
    /// This method allows to create more complex directed acyclic graph structures that
    /// incorporate concepts like sends, side-chaining, etc, rather than being restricted to tree
    /// structures where signals can only ever be joined but never divided.
    ///
    /// Note: When using multiple `Output`s in this fashion, you will need to be sure to pull the
    /// frames from each `Output` in sync (whether per frame or per buffer). This is because when
    /// output A requests `Frame`s before output B, those frames mjust remain available for output
    /// B and in turn must be stored in an intermediary ring buffer.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Signal;
    ///
    /// fn main() {
    ///     let frames = [[0.1], [0.2], [0.3]];
    ///     let bus = frames.iter().cloned().bus();
    ///     let mut a = bus.send();
    ///     let mut b = bus.send();
    ///     assert_eq!(a.collect::<Vec<_>>(), vec![[0.1], [0.2], [0.3]]);
    ///     assert_eq!(b.collect::<Vec<_>>(), vec![[0.1], [0.2], [0.3]]);
    /// }
    /// ```
    fn bus(self) -> Bus<Self> {
        Bus {
            node: std::rc::Rc::new(std::cell::RefCell::new(SharedNode {
                signal: self,
                buffers: vec![std::collections::VecDeque::new()],
            })),
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
pub struct FromInterleavedSamples<I, F>
    where I: Iterator,
          I::Item: Sample,
          F: Frame<Sample=I::Item>,
{
    samples: I,
    frame: std::marker::PhantomData<F>,
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

impl Rate {

    /// Create a `ConstHz` iterator which consistently yields "hz / rate".
    pub fn const_hz(self, hz: f64) -> ConstHz {
        ConstHz { step: hz / self.hz }
    }

    /// Create a variable `hz` some iterator that yields hz and an initial hz.
    ///
    /// The `Hz` iterator yields phase step sizes equal to "hz / rate".
    pub fn hz<I>(self, init: f64, hz: I) -> Hz<I> {
        Hz {
            hz: hz,
            last_step_size: init / self.hz,
            rate: self,
        }
    }

}

pub trait Step {
    fn step(&mut self) -> f64;
}

// signal::rate(44_100.0).varistep(repeat(440.0), 440.0).sin()
// signal::rate(44_100.0).step(440.0).sin()
// step(440.0, 44_100.0).phase().sine()
// vari_step(repeat(440.0), 440.0, 44_100.0).phase().sine()

impl VariStep {
    pub fn phase(self) -> Phase<Self> {
        phase(self)
    }
}

impl ConstStep {
    pub fn phase(self) -> Phase<Self> {
        phase(self)
    }
}

impl Step for ConstStep {
    #[inline]
    fn step(&mut self) -> f64 {
        self.step
    }
}

impl<I> Step for VariStep<I>
    where I: Iterator<Item=f64>,
{
    #[inline]
    fn step(&mut self) -> f64 {
        match self.hz.next() {
            Some(hz) => {
                self.last_step_size = hz / self.rate;
                hz
            },
            None => self.last_step_size,
        }
    }
}

/// An iterator that yields a phase, useful for waveforms like Sine or Saw.
#[derive(Clone)]
pub struct Phase<S> {
    step: S,
    next: f64,
}

// /// An iterator that yields a phase, useful for waveforms like Sine or Saw.
// #[derive(Clone)]
// pub struct Phase<I> {
//     hz: I,
//     last_hz: f64,
//     rate: f64,
//     next: f64,
// }

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

/// A type which allows for `send`ing a single `Signal` to multiple outputs.
///
/// This type manages
pub struct Bus<S>
    where S: Signal,
          S::Item: Frame,
{
    node: std::rc::Rc<std::cell::RefCell<SharedNode<S>>>,
}

/// The data shared between each `Output`.
struct SharedNode<S>
    where S: Signal,
          S::Item: Frame,
{
    signal: S,
    buffers: Vec<std::collections::VecDeque<S::Item>>,
}

/// An output node to which some signal `S` is `Output`ing its frames.
///
/// It may be more accurate to say that the `Output` "pull"s frames from the signal.
pub struct Output<S>
    where S: Signal,
          S::Item: Frame,
{
    idx: usize,
    node: std::rc::Rc<std::cell::RefCell<SharedNode<S>>>,
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
///     let mut signal = signal::from_interleaved_samples::<_, [i32; 2]>(foo.iter().cloned());
///     assert_eq!(signal.next(), Some([0, 1]));
///     assert_eq!(signal.next(), Some([2, 3]));
///     assert_eq!(signal.next(), None);
///
///     let bar = [0, 1, 2];
///     let mut signal = signal::from_interleaved_samples::<_, [i32; 2]>(bar.iter().cloned());
///     assert_eq!(signal.next(), Some([0, 1]));
///     assert_eq!(signal.next(), None);
/// }
/// ```
pub fn from_interleaved_samples<I, F>(samples: I) -> FromInterleavedSamples<I, F>
    where I: Iterator,
          I::Item: Sample,
          F: Frame<Sample=I::Item>,
{
    FromInterleavedSamples {
        samples: samples,
        frame: std::marker::PhantomData,
    }
}


/// Creates a `Phase` that continuously steps forward by `hz / sample_rate`
///
/// # Example
///
/// ```rust
/// extern crate sample;
///
/// fn main() {
///     let mut phase = sample::signal::phase(1.0, 4.0);
///     assert_eq!(phase.next(), Some([0.0]));
///     assert_eq!(phase.next(), Some([0.25]));
///     assert_eq!(phase.next(), Some([0.5]));
///     assert_eq!(phase.next(), Some([0.75]));
///     assert_eq!(phase.next(), Some([0.0]));
///     assert_eq!(phase.next(), Some([0.25]));
/// }
/// ```
pub fn phase(hz: f64, sample_rate: f64) -> Phase<ConstStep> {
    Phase {
        step: ConstStep { step: hz / sample_rate },
        next: 0.0,
    }
}

pub fn vari_phase<I>(hz: I, init: f64, sample_rate: f64) -> Phase<VariStep<I>> {
    Phase {
        step: VariStep {
            hz: hz,
            rate: sample_rate,
            last_step_size: init,
        },
        next: 0.0,
    }
}


/// Produces a `Signal` that yields a sine wave oscillating at the given hz.
///
/// # Example
///
/// ```rust
/// extern crate sample;
///
/// use sample::signal;
///
/// fn main() {
///     // Generates a sine wave signal at 1hz to be sampled 4 times per second.
///     let mut signal = signal::sine(signal::phase(1.0, 4.0));
///     assert_eq!(signal.next(), Some([0.0]));
///     assert_eq!(signal.next(), Some([1.0]));
///     signal.next();
///     assert_eq!(signal.next(), Some([-1.0]));
/// }
/// ```
pub fn sine(phase: Phase) -> Sine {
    Sine { phase: phase }
}

/// Produces a `Signal` that yields a saw wave oscillating at the given hz.
///
/// # Example
///
/// ```rust
/// extern crate sample;
///
/// use sample::signal;
///
/// fn main() {
///     // Generates a saw wave signal at 1hz to be sampled 4 times per second.
///     let mut signal = signal::saw(signal::phase(1.0, 4.0));
///     assert_eq!(signal.next(), Some([1.0]));
///     assert_eq!(signal.next(), Some([0.5]));
///     assert_eq!(signal.next(), Some([0.0]));
///     assert_eq!(signal.next(), Some([-0.5]));
/// }
/// ```
pub fn saw(phase: Phase) -> Saw {
    Saw { phase: phase }
}

/// Produces a `Signal` that yields a square wave oscillating at the given hz.
///
/// # Example
///
/// ```rust
/// extern crate sample;
///
/// use sample::signal;
///
/// fn main() {
///     // Generates a square wave signal at 1hz to be sampled 4 times per second.
///     let mut signal = signal::square(signal::phase(1.0, 4.0));
///     assert_eq!(signal.next(), Some([1.0]));
///     assert_eq!(signal.next(), Some([1.0]));
///     assert_eq!(signal.next(), Some([-1.0]));
///     assert_eq!(signal.next(), Some([-1.0]));
/// }
/// ```
pub fn square(phase: Phase) -> Square {
    Square { phase: phase }
}

/// Produces a `Signal` that yields random values between -1.0..1.0.
///
/// # Example
///
/// ```rust
/// extern crate sample;
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
/// use sample::signal;
///
/// fn main() {
///     // Creates a simplex noise signal oscillating at 440hz sampled 44_100 times per second.
///     let mut signal = signal::noise_simplex(signal::phase(440.0, 44_100.0));
///     for n in signal.take(1_000_000) {
///         assert!(-1.0 <= n[0] && n[0] < 1.0);
///     }
/// }
/// ```
pub fn noise_simplex(phase: Phase) -> NoiseSimplex {
    NoiseSimplex { phase: phase }
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


impl<I, F> Iterator for FromInterleavedSamples<I, F>
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


impl Phase {

    /// Before yielding the current phase, the internal phase is stepped forward and wrapped via
    /// the given value.
    #[inline]
    pub fn next_phase_wrapped_to(&mut self, rem: f64) -> f64 {
        let phase = self.next;
        self.next = (self.next + self.step) % rem;
        phase
    }

    /// Calls `next_phase_wrapped_to`, with a wrapping value of `1.0`.
    #[inline]
    pub fn next_phase(&mut self) -> f64 {
        self.next_phase_wrapped_to(1.0)
    }

    /// A composable version of the `signal::sine` function.
    #[inline]
    pub fn sine(self) -> Sine {
        sine(self)
    }

    /// A composable version of the `signal::saw` function.
    #[inline]
    pub fn saw(self) -> Saw {
        saw(self)
    }

    /// A composable version of the `signal::square` function.
    #[inline]
    pub fn square(self) -> Square {
        square(self)
    }

    /// A composable version of the `signal::noise_simplex` function.
    #[inline]
    pub fn noise_simplex(self) -> NoiseSimplex {
        noise_simplex(self)
    }

}

impl Iterator for Phase {
    type Item = [f64; 1];
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Some([self.next_phase()])
    }
}


impl Iterator for Sine {
    type Item = [f64; 1];
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        const PI_2: f64 = std::f64::consts::PI * 2.0;
        let phase = self.phase.next_phase();
        Some([(PI_2 * phase).sin()])
    }
}


impl Iterator for Saw {
    type Item = [f64; 1];
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let phase = self.phase.next_phase();
        Some([phase * -2.0 + 1.0])
    }
}


impl Iterator for Square {
    type Item = [f64; 1];
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let phase = self.phase.next_phase();
        Some([if phase < 0.5 { 1.0 } else { -1.0 }])
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

impl Iterator for Noise {
    type Item = [f64; 1];
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Some([self.next_sample()])
    }
}


impl NoiseSimplex {
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
            let i0 = x.floor() as i64;
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

impl Iterator for NoiseSimplex {
    type Item = [f64; 1];
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Some([self.next_sample()])
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
    #[inline]
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

impl<S> ExactSizeIterator for Delay<S>
    where Delay<S>: Iterator,
          S: ExactSizeIterator,
{
    #[inline]
    fn len(&self) -> usize {
        self.signal.len() + self.n_frames
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
    #[inline]
    fn clone(&self) -> Self {
        ToSamples {
            signal: self.signal.clone(),
            current_frame: self.current_frame.clone(),
        }
    }
}

impl<S> ExactSizeIterator for ToSamples<S>
    where ToSamples<S>: Iterator,
          S: ExactSizeIterator,
          S::Item: Frame,
          <S::Item as Frame>::Channels: ExactSizeIterator,
{
    #[inline]
    fn len(&self) -> usize {
        self.signal.len() * <S::Item as Frame>::n_channels()
            + self.current_frame.as_ref().map(|f| f.len()).unwrap_or(0)
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


impl<S> Bus<S>
    where S: Signal,
          S::Item: Frame,
{
    /// Produce a new Output node to which the signal `S` will output its frames.
    #[inline]
    pub fn send(&self) -> Output<S> {
        let idx = self.node.borrow().buffers.len();
        self.node.borrow_mut().buffers.push(std::collections::VecDeque::new());
        Output {
            idx: idx,
            node: self.node.clone(),
        }
    }
}

impl<S> SharedNode<S>
    where S: Signal,
          S::Item: Frame,
{
    /// Requests the next frame for the `Output` whose ring buffer lies at the given index.
    ///
    /// If there are no frames waiting in the front of the ring buffer, a new frame will be
    /// requested from the `signal` and appended to the back of each ring buffer.
    #[inline]
    fn next_frame(&mut self, idx: usize) -> Option<S::Item> {
        loop {
            match self.buffers[idx].pop_front() {
                Some(frame) => return Some(frame),
                None => match self.signal.next() {
                    Some(frame) => {
                        for buffer in self.buffers.iter_mut() {
                            buffer.push_back(frame);
                        }
                    },
                    None => return None,
                }
            }
        }
    }
}

impl<S> Iterator for Output<S>
    where S: Signal,
          S::Item: Frame,
{
    type Item = S::Item;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.node.borrow_mut().next_frame(self.idx)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let node = self.node.borrow();
        let (lower, upper) = node.signal.size_hint();
        let n = node.buffers[self.idx].len();
        (lower + n, upper.map(|upper| upper + n))
    }
}
