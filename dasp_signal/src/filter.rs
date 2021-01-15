//! An extension to the **Signal** trait that enables iterative filtering.
//!
//! ### Required Features
//!
//! - When using `dasp_signal`, this item requires the **filter** feature to be enabled.
//! - When using `dasp`, this item requires the **signal-filter** feature to be enabled.

use crate::Signal;
use dasp_filter as filter;
use dasp_frame::Frame;
use dasp_sample::{FromSample, Sample};

/// An extension to the **Signal** trait that enables iterative filtering.
///
/// # Example
///
/// ```
/// use dasp_filter::{self as filter, Coefficients};
/// use dasp_signal::{self as signal, Signal};
/// use dasp_signal::filter::SignalFilter;
///
/// fn main() {
///     let signal = signal::rate(48000.0).const_hz(997.0).sine();
///     // Notch filter to attenuate 997 Hz.
///     let coeff = Coefficients {
///         b0: 0.9157328640471359f64,
///         b1: -1.8158910212730535,
///         b2: 0.9157328640471359,
///         a1: -1.8158910212730535,
///         a2: 0.831465728094272,
///     };
///     let mut filtered = signal.filtered(coeff);
///     assert_eq!(
///         filtered.take(4).collect::<Vec<_>>(),
///         vec![0.0, 0.11917058366454024, 0.21640079287630784, 0.2938740006664008]
///     );
/// }
/// ```
///
/// ### Required Features
///
/// - When using `dasp_signal`, this item requires the **filter** feature to be enabled.
/// - When using `dasp`, this item requires the **signal-filter** feature to be enabled.
pub trait SignalFilter: Signal {
    fn filtered(
        self,
        coeff: filter::Coefficients<<<Self::Frame as Frame>::Sample as Sample>::Float>,
    ) -> FilteredSignal<Self>
    where
        Self: Sized,
        <Self::Frame as Frame>::Sample:
            FromSample<<<Self::Frame as Frame>::Sample as Sample>::Float>,
    {
        let biquad = filter::Biquad::from(coeff);

        FilteredSignal {
            signal: self,
            biquad,
        }
    }
}

/// An adaptor that calculates and yields a filtered signal.
///
/// ### Required Features
///
/// - When using `dasp_signal`, this item requires the **filter** feature to be enabled.
/// - When using `dasp`, this item requires the **signal-filter** feature to be enabled.
pub struct FilteredSignal<S>
where
    S: Signal,
    <S::Frame as Frame>::Sample: FromSample<<<S::Frame as Frame>::Sample as Sample>::Float>,
{
    signal: S,
    biquad: filter::Biquad<<S::Frame as Frame>::Float>,
}

impl<S> Signal for FilteredSignal<S>
where
    S: Signal,
    <S::Frame as Frame>::Sample: FromSample<<<S::Frame as Frame>::Sample as Sample>::Float>,
{
    // Output is the same type as the input.
    type Frame = S::Frame;

    fn next(&mut self) -> Self::Frame {
        self.biquad.apply(self.signal.next())
    }

    fn is_exhausted(&self) -> bool {
        self.signal.is_exhausted()
    }
}

// Impl this for all `Signal`s.
impl<T> SignalFilter for T where T: Signal {}
