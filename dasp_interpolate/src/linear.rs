//! A linear interpolator implementation.
//!
//! ### Required Features
//!
//! - When using `dasp_interpolate`, this module requires the **linear** feature to be enabled.
//! - When using `dasp`, this module requires the **interpolate-linear** feature to be enabled.

use crate::Interpolator;
use dasp_frame::Frame;
use dasp_sample::{Duplex, Sample};

/// Interpolator that interpolates linearly between the previous value and the next value
///
/// ### Required Features
///
/// - When using `dasp_interpolate`, this item requires the **linear** feature to be enabled.
/// - When using `dasp`, this item requires the **interpolate-linear** feature to be enabled.
pub struct Linear<F> {
    left: F,
    right: F,
}

impl<F> Linear<F> {
    /// Create a new Linear Interpolator, where `left` and `right` are the first two frames to be
    /// interpolated.
    ///
    /// ### Required Features
    ///
    /// - When using `dasp_interpolate`, this item requires the **linear** feature to be enabled.
    /// - When using `dasp`, this item requires the **interpolate-linear** feature to be enabled.
    pub fn new(left: F, right: F) -> Linear<F> {
        Linear {
            left: left,
            right: right,
        }
    }
}

impl<F> Interpolator for Linear<F>
where
    F: Frame,
    F::Sample: Duplex<f64>,
{
    type Frame = F;

    /// Converts linearly from the previous value, using the next value to interpolate. It is
    /// possible, although not advisable, to provide an x > 1.0 or < 0.0, but this will just
    /// continue to be a linear ramp in one direction or another.
    fn interpolate(&self, x: f64) -> Self::Frame {
        self.left.zip_map(self.right, |l, r| {
            let l_f = l.to_sample::<f64>();
            let r_f = r.to_sample::<f64>();
            let diff = r_f - l_f;
            ((diff * x) + l_f).to_sample::<<Self::Frame as Frame>::Sample>()
        })
    }

    fn next_source_frame(&mut self, source_frame: Self::Frame) {
        self.left = self.right;
        self.right = source_frame;
    }

    fn reset(&mut self) {
        self.left = Self::Frame::EQUILIBRIUM;
        self.right = Self::Frame::EQUILIBRIUM;
    }
}
