//! Module for windowing over a batch of Frames. Includes default Hanning and Rectangle window
//! types.

use {FloatSample, Sample};
use core;
use core::marker::PhantomData;
use frame::Frame;
use signal::{self, Signal};


/// The window function used within a `Window`.
pub trait Type {
    /// Returns the amplitude for the given phase, given as some `Sample` type.
    fn at_phase<S: Sample>(phase: S) -> S;
}


/// A type of window function, also known as teh "raised cosine window".
///
/// [Wiki entry](https://en.wikipedia.org/wiki/Window_function#Hann_.28Hanning.29_window).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hanning;

/// The simplest window type, equivalent to replacing all but *N* values of data sequence by
/// zeroes, making it appear as though the waveform suddenly turns on and off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rectangle;

/// A `Signal` type that for every yielded `phase`, yields the amplitude across the `window::Type`
/// for that phase.
#[derive(Clone)]
pub struct Window<F, W> 
    where F: Frame,
          W: Type,
{
    /// Yields phase stepped at a constant rate to be passed to the window function `W`.
    pub phase: signal::Phase<signal::ConstHz>,
    marker: PhantomData<(F, W)>,
}

/// Takes a long slice of frames and yields `Windowed` chunks of size `bin` once every `hop` frames.
#[derive(Clone)]
pub struct Windower<'a, F, W> 
    where F: 'a + Frame, 
          W: Type,
{
    /// The size of each `Windowed` chunk to be yielded.
    pub bin: usize,
    /// The step size over `frames` for the start of each new `Windowed` chunk.
    pub hop: usize,
    /// The beginning of the remaining slice to be yielded by the `Windower`.
    pub frames: &'a [F],
    wttype: PhantomData<W>
}

/// An Iterator that multiplies a Signal with a Window.
///
/// Returns `None` once the `Window` has been exhausted.
#[derive(Clone)]
pub struct Windowed<S, W>
    where S: Signal,
          W: Type,
{
    signal: S,
    window: Window<<S::Frame as Frame>::Float, W>,
}


impl Type for Hanning {
    fn at_phase<S: Sample>(phase: S) -> S {
        const PI_2: f64 = core::f64::consts::PI * 2.0;
        let v = phase.to_float_sample().to_sample() * PI_2;
        (0.5 * (1.0 - super::ops::f64::cos(v)))
            .to_sample::<S::Float>()
            .to_sample::<S>()
    }
}

impl Type for Rectangle {
    fn at_phase<S: Sample>(_phase: S) -> S {
        <S::Float as FloatSample>::identity().to_sample::<S>()
    }
}


impl<F, W> Window<F, W> 
    where F: Frame,
          W: Type
{
    /// Construct a new `Window` with the given length as a number of frames.
    pub fn new(len: usize) -> Self {
        let step = signal::rate(len as f64 - 1.0).const_hz(1.0);
        Window {
            phase: signal::phase(step),
            marker: PhantomData,
        }
    }
}


impl<'a, F, W> Windower<'a, F, W> 
    where F: 'a + Frame, 
          W: Type
{
    /// Constructor for a new `Windower` iterator.
    pub fn new(frames: &'a [F], bin: usize, hop: usize) -> Self {
        Windower {
            bin: bin,
            hop: hop,
            frames: frames,
            wttype: PhantomData
        }
    }
}

impl<'a, F> Windower<'a, F, Rectangle>
    where F: 'a + Frame,
{
    /// Constructor for a `Windower` using the `Rectangle` window function.
    pub fn rectangle(frames: &'a [F], bin: usize, hop: usize) -> Self {
        Windower::new(frames, bin, hop)
    }
}

impl<'a, F> Windower<'a, F, Hanning>
    where F: 'a + Frame,
{
    /// Constructor for a `Windower` using the `Hanning` window function.
    pub fn hanning(frames: &'a [F], bin: usize, hop: usize) -> Self {
        Windower::new(frames, bin, hop)
    }
}


impl<F, W> Iterator for Window<F, W> 
    where F: Frame, 
          W: Type
{
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        let v = W::at_phase(self.phase.next_phase());
        let v_f: <F::Sample as Sample>::Float = v.to_sample();
        Some(F::from_fn(|_| v_f.to_sample::<F::Sample>()))
    }
}

impl<'a, F, W> Iterator for Windower<'a, F, W> 
    where F: 'a + Frame, 
          W: Type
{
    type Item = Windowed<signal::FromIterator<core::iter::Cloned<core::slice::Iter<'a, F>>>, W>;

    fn next(&mut self) -> Option<Self::Item> {
        let num_frames = self.frames.len();
        if self.bin <= num_frames {
            let frames = &self.frames[..self.bin];
            let window = Window::new(self.bin);
            self.frames = if self.hop < num_frames { &self.frames[self.hop..] } else { &[] };
            Some(Windowed {
                signal: signal::from_slice(frames),
                window: window,
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let num_frames = self.frames.len();
        // Must have at least `bin` number of frames left to iterate at all.
        if self.bin < num_frames {
            // If the hop size is 0, we'll iterate forever.
            if self.hop == 0 {
                return (core::usize::MAX, None);
            }
            // Otherwise we can determine exactly how many iterations remain.
            let remaining_hop_frames = self.frames.len() - self.bin;
            let remaining_iterations = remaining_hop_frames / self.hop;
            (remaining_iterations, Some(remaining_iterations))
        } else {
            (0, Some(0))
        }
    }
}

impl<S, W> Iterator for Windowed<S, W>
    where S: Signal,
          W: Type,
{
    type Item = S::Frame;
    fn next(&mut self) -> Option<Self::Item> {
        self.window.next().map(|w_f| {
            let s_f = self.signal.next();
            s_f.mul_amp(w_f)
        })
    }
}

/// A helper function for constructing a `Window` that uses a `Hanning` `Type` function.
pub fn hanning<F>(num_frames: usize) -> Window<F, Hanning> 
    where F: Frame,
{
    Window::new(num_frames)
}

/// A helper function for constructing a `Window` that uses a `Rectangle` `Type` function.
pub fn rectangle<F>(num_frames: usize) -> Window<F, Rectangle> 
    where F: Frame,
{
    Window::new(num_frames)
}
