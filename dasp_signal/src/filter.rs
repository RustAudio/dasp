//! An extension to the **Signal** trait that enables iterative filtering.
//!
//! ### Required Features
//!
//! - When using `dasp_signal`, this item requires the **filter** feature to be enabled.
//! - When using `dasp`, this item requires the **signal-filter** feature to be enabled.

use crate::Signal;
use dasp_frame::Frame;
use dasp_filter as filter;
use dasp_sample::{Sample, FromSample};

/// An extension to the **Signal** trait that enables iterative filtering.
///
/// ### Required Features
///
/// - When using `dasp_signal`, this item requires the **filter** feature to be enabled.
/// - When using `dasp`, this item requires the **signal-filter** feature to be enabled.
pub trait SignalFilter: Signal {
    /// An adaptor that calculates and yields a filtered signal.
    ///
    /// ### Required Features
    ///
    /// - When using `dasp_signal`, this item requires the **filter** feature to be enabled.
    /// - When using `dasp`, this item requires the **signal-filter** feature to be enabled.
    fn filtered(
        self,
        coeff: filter::Coefficients<<<Self::Frame as Frame>::Sample as Sample>::Float>,
    ) -> FilteredSignal<Self>
    where
        Self: Sized,
        <Self::Frame as Frame>::Sample: FromSample<<<Self::Frame as Frame>::Sample as Sample>::Float>,
    {
        let biquad = filter::Biquad::from(coeff);

        FilteredSignal {
            signal: self,
            biquad,
        }
    }
}

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
