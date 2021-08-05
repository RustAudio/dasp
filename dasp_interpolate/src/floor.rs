//! A floor interpolator implementation.
//!
//! ### Required Features
//!
//! - When using `dasp_interpolate`, this module requires the **floor** feature to be enabled.
//! - When using `dasp`, this module requires the **interpolate-floor** feature to be enabled.

use crate::Interpolator;
use dasp_frame::Frame;
use dasp_sample::Duplex;

/// Interpolator that rounds off any values to the previous value from the source.
///
/// ### Required Features
///
/// - When using `dasp_interpolate`, this item requires the **floor** feature to be enabled.
/// - When using `dasp`, this item requires the **interpolate-floor** feature to be enabled.
pub struct Floor<F> {
    left: F,
}

impl<F> Floor<F> {
    /// Create a new Floor Interpolator.
    ///
    /// ### Required Features
    ///
    /// - When using `dasp_interpolate`, this item requires the **floor** feature to be enabled.
    /// - When using `dasp`, this item requires the **interpolate-floor** feature to be enabled.
    pub fn new(left: F) -> Floor<F> {
        Floor { left: left }
    }
}

impl<F> Interpolator for Floor<F>
where
    F: Frame,
    F::Sample: Duplex<f64>,
{
    type Frame = F;

    fn interpolate(&self, _x: f64) -> Self::Frame {
        self.left
    }

    fn next_source_frame(&mut self, source_frame: Self::Frame) {
        self.left = source_frame;
    }

    fn reset(&mut self) {
        self.left = Self::Frame::EQUILIBRIUM;
    }
}
