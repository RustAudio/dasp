//! The Interpolate module allows for conversion between various sample rates.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(core_intrinsics))]

use dasp_frame::Frame;

#[cfg(feature = "floor")]
pub mod floor;
#[cfg(feature = "linear")]
pub mod linear;
#[cfg(feature = "sinc")]
pub mod sinc;

/// Types that can interpolate between two values.
///
/// Implementations should keep track of the necessary data both before and after the current
/// frame.
pub trait Interpolator {
    /// The type of frame over which the interpolate may operate.
    type Frame: Frame;

    /// Given a distance between [0.0 and 1.0) toward the following sample, return the interpolated
    /// value.
    fn interpolate(&self, x: f64) -> Self::Frame;

    /// To be called whenever the Interpolator value steps passed 1.0.
    fn next_source_frame(&mut self, source_frame: Self::Frame);
}
