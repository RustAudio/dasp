//! An abstraction for sample/frame rate interpolation.
//!
//! The [**Interpolator**](./trait.Interpolator.html) trait provides an abstraction over different
//! types of rate interpolation.
//!
//! See the `dasp_signal` crate (or `dasp::signal` module) **Converter** type for a convenient way
//! to interpolate the rate of arbitrary signals.
//!
//! ### Optional Features
//!
//! - The **floor** feature (or **interpolate-floor** feature if using `dasp`) provides a floor
//!   interpolator implementation.
//! - The **linear** feature (or **interpolate-linear** feature if using `dasp`) provides a linear
//!   interpolator implementation.
//! - The **sinc** feature (or **interpolate-sinc** feature if using `dasp`) provides a sinc
//!   interpolator implementation.

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
