//! Items to ease the application of windowing functions to signals.

use crate::{ConstHz, FromIterator, Phase, Signal};
use core::marker::PhantomData;
use dasp_frame::Frame;
use dasp_sample::Sample;
use dasp_window::Window as WindowType;

#[cfg(feature = "window-hanning")]
pub use hanning::hanning;
#[cfg(feature = "window-rectangle")]
pub use rectangle::rectangle;

#[cfg(feature = "window-hanning")]
mod hanning;
#[cfg(feature = "window-rectangle")]
mod rectangle;

/// A `Signal` type that for every yielded `phase`, yields the amplitude across the `window::Type`
/// for that phase.
///
/// ### Required Features
///
/// - When using `dasp_signal`, this item requires the **window** feature to be enabled.
/// - When using `dasp`, this item requires the **signal-window** feature to be enabled.
#[derive(Clone)]
pub struct Window<F, W>
where
    F: Frame,
    W: WindowType,
{
    /// Yields phase stepped at a constant rate to be passed to the window function `W`.
    pub phase: Phase<ConstHz>,
    marker: PhantomData<(F, W)>,
}

/// Takes a long slice of frames and yields `Windowed` chunks of size `bin` once every `hop` frames.
///
/// ### Required Features
///
/// - When using `dasp_signal`, this item requires the **window** feature to be enabled.
/// - When using `dasp`, this item requires the **signal-window** feature to be enabled.
#[derive(Clone)]
pub struct Windower<'a, F, W>
where
    F: 'a + Frame,
    W: WindowType,
{
    /// The size of each `Windowed` chunk to be yielded.
    pub bin: usize,
    /// The step size over `frames` for the start of each new `Windowed` chunk.
    pub hop: usize,
    /// The beginning of the remaining slice to be yielded by the `Windower`.
    pub frames: &'a [F],
    wttype: PhantomData<W>,
}

/// An Iterator that multiplies a Signal with a Window.
///
/// Returns `None` once the `Window` has been exhausted.
///
/// ### Required Features
///
/// - When using `dasp_signal`, this item requires the **window** feature to be enabled.
/// - When using `dasp`, this item requires the **signal-window** feature to be enabled.
#[derive(Clone)]
pub struct Windowed<S, W>
where
    S: Signal,
    W: WindowType,
{
    signal: S,
    window: Window<<S::Frame as Frame>::Float, W>,
}

impl<F, W> Window<F, W>
where
    F: Frame,
    W: WindowType,
{
    /// Construct a new `Window` with the given length as a number of frames.
    ///
    /// ### Required Features
    ///
    /// - When using `dasp_signal`, this item requires the **window** feature to be enabled.
    /// - When using `dasp`, this item requires the **signal-window** feature to be enabled.
    pub fn new(len: usize) -> Self {
        let step = crate::rate(len as f64 - 1.0).const_hz(1.0);
        Window {
            phase: crate::phase(step),
            marker: PhantomData,
        }
    }
}

impl<'a, F, W> Windower<'a, F, W>
where
    F: 'a + Frame,
    W: WindowType,
{
    /// Constructor for a new `Windower` iterator.
    ///
    /// ### Required Features
    ///
    /// - When using `dasp_signal`, this item requires the **window** feature to be enabled.
    /// - When using `dasp`, this item requires the **signal-window** feature to be enabled.
    pub fn new(frames: &'a [F], bin: usize, hop: usize) -> Self {
        Windower {
            bin: bin,
            hop: hop,
            frames: frames,
            wttype: PhantomData,
        }
    }
}

impl<F, W> Iterator for Window<F, W>
where
    F: Frame,
    W: WindowType,
{
    type Item = F;

    fn next(&mut self) -> Option<Self::Item> {
        let v = W::at_phase(self.phase.next_phase());
        let v_f: <F::Sample as Sample>::Float = v.to_sample();
        Some(F::from_fn(|_| v_f.to_sample::<F::Sample>()))
    }
}

impl<'a, F, W> Iterator for Windower<'a, F, W>
where
    F: 'a + Frame,
    W: WindowType,
{
    type Item = Windowed<FromIterator<core::iter::Cloned<core::slice::Iter<'a, F>>>, W>;

    fn next(&mut self) -> Option<Self::Item> {
        let num_frames = self.frames.len();
        if self.bin <= num_frames {
            let frames = &self.frames[..self.bin];
            let window = Window::new(self.bin);
            self.frames = if self.hop < num_frames {
                &self.frames[self.hop..]
            } else {
                &[]
            };
            Some(Windowed {
                signal: crate::from_iter(frames.iter().cloned()),
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
where
    S: Signal,
    W: WindowType,
{
    type Item = S::Frame;
    fn next(&mut self) -> Option<Self::Item> {
        self.window.next().map(|w_f| {
            let s_f = self.signal.next();
            s_f.mul_amp(w_f)
        })
    }
}
