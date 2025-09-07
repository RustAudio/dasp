//! An extension to the **Signal** trait that monitors the RMS of a signal.
//!
//! ### Required Features
//!
//! - When using `dasp_signal`, this module requires the **rms** feature to be enabled.
//! - When using `dasp`, this module requires the **signal-rms** feature to be enabled.

use crate::Signal;
use dasp_frame::Frame;
use dasp_ring_buffer as ring_buffer;
use dasp_rms as rms;

/// An extension to the **Signal** trait that monitors the RMS of a signal.
///
/// ### Required Features
///
/// - When using `dasp_signal`, this item requires the **rms** feature to be enabled.
/// - When using `dasp`, this item requires the **signal-rms** feature to be enabled.
pub trait SignalRms: Signal {
    /// An adaptor that yields the RMS of the signal.
    ///
    /// The window size of the RMS detector is equal to the given ring buffer length.
    ///
    /// # Example
    ///
    /// ```
    /// use dasp_ring_buffer as ring_buffer;
    /// use dasp_signal::{self as signal, Signal};
    /// use dasp_signal::rms::SignalRms;
    ///
    /// fn main() {
    ///     let frames = [[0.9], [-0.8], [0.6], [-0.9]];
    ///     let signal = signal::from_iter(frames.iter().cloned());
    ///     let ring_buffer = ring_buffer::Fixed::from([[0.0]; 2]);
    ///     let mut rms_signal = signal.rms(ring_buffer);
    ///
    ///     let result = rms_signal.next()[0];
    ///     assert!(f64::abs(result - 0.6363961030678927) < 0.000001,
    ///             "Expected ~0.6363961030678927, got {}", result);
    ///
    ///     let result = rms_signal.next()[0];
    ///     assert!(f64::abs(result - 0.8514693182963201) < 0.000001,
    ///             "Expected ~0.8514693182963201, got {}", result);
    ///
    ///     let result = rms_signal.next()[0];
    ///     assert!(f64::abs(result - 0.7071067811865476) < 0.000001,
    ///             "Expected ~0.7071067811865476, got {}", result);
    /// }
    /// ```
    ///
    /// ### Required Features
    ///
    /// - When using `dasp_signal`, this item requires the **rms** feature to be enabled.
    /// - When using `dasp`, this item requires the **signal-rms** feature to be enabled.
    fn rms<S>(self, ring_buffer: ring_buffer::Fixed<S>) -> Rms<Self, S>
    where
        Self: Sized,
        S: ring_buffer::Slice<Element = <Self::Frame as Frame>::Float> + ring_buffer::SliceMut,
    {
        Rms {
            signal: self,
            rms: rms::Rms::new(ring_buffer),
        }
    }
}

/// An adaptor that yields the RMS of the signal.
///
/// The window size of the RMS detector is equal to the given ring buffer length.
///
/// ### Required Features
///
/// - When using `dasp_signal`, this item requires the **rms** feature to be enabled.
/// - When using `dasp`, this item requires the **signal-rms** feature to be enabled.
#[derive(Clone)]
pub struct Rms<S, D>
where
    S: Signal,
    D: ring_buffer::Slice<Element = <S::Frame as Frame>::Float>,
{
    signal: S,
    rms: rms::Rms<S::Frame, D>,
}

impl<S, D> Rms<S, D>
where
    S: Signal,
    D: ring_buffer::Slice<Element = <S::Frame as Frame>::Float> + ring_buffer::SliceMut,
{
    /// The same as `Signal::next` but does not calculate the final square root required to
    /// determine the RMS.
    ///
    /// ### Required Features
    ///
    /// - When using `dasp_signal`, this item requires the **rms** feature to be enabled.
    /// - When using `dasp`, this item requires the **signal-rms** feature to be enabled.
    pub fn next_squared(&mut self) -> <Self as Signal>::Frame {
        self.rms.next_squared(self.signal.next())
    }

    /// Consumes the `Rms` signal and returns its inner signal `S` and `Rms` detector.
    ///
    /// ### Required Features
    ///
    /// - When using `dasp_signal`, this item requires the **rms** feature to be enabled.
    /// - When using `dasp`, this item requires the **signal-rms** feature to be enabled.
    pub fn into_parts(self) -> (S, rms::Rms<S::Frame, D>) {
        let Rms { signal, rms } = self;
        (signal, rms)
    }
}

impl<S, D> Signal for Rms<S, D>
where
    S: Signal,
    D: ring_buffer::Slice<Element = <S::Frame as Frame>::Float> + ring_buffer::SliceMut,
{
    type Frame = <S::Frame as Frame>::Float;
    fn next(&mut self) -> Self::Frame {
        self.rms.next(self.signal.next())
    }

    fn is_exhausted(&self) -> bool {
        self.signal.is_exhausted()
    }
}

impl<T> SignalRms for T where T: Signal {}
