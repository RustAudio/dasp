//! An extension to the **Signal** trait that enables iterative filtering.
//!
//! ### Required Features
//!
//! - When using `dasp_signal`, this item requires the **filter** feature to be enabled.
//! - When using `dasp`, this item requires the **signal-filter** feature to be enabled.

use crate::Signal;
use dasp_frame::Frame;
use dasp_filter as filter;
use dasp_sample::Sample;

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
{
    signal: S,
    biquad: filter::Biquad<<S::Frame as Frame>::Float>,
}
