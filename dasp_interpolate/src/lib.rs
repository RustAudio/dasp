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
//!
//! ### no_std
//!
//! If working in a `no_std` context, you can disable the default **std** feature with
//! `--no-default-features`.
//!
//! To enable all of the above features in a `no_std` context, enable the **all-no-std** feature.

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
///
/// # Rate Configuration
///
/// Some interpolators require sample rate information to operate correctly (e.g., sinc
/// interpolation needs the rate ratio for anti-aliasing). The `set_hz_to_hz`,
/// `set_playback_hz_scale`, and `set_sample_hz_scale` methods provide alternative ways
/// to configure this - use whichever matches the information available. These methods
/// are called automatically by the corresponding `Converter` methods.
///
/// Interpolators that don't need rate information (floor, linear) can use the default
/// no-op implementations.
pub trait Interpolator {
    /// The type of frame over which the interpolate may operate.
    type Frame: Frame;

    /// Given a distance between [0.0 and 1.0) toward the following sample, return the interpolated
    /// value.
    fn interpolate(&self, x: f64) -> Self::Frame;

    /// To be called whenever the Interpolator value steps passed 1.0.
    fn next_source_frame(&mut self, source_frame: Self::Frame);

    /// Resets the state of the interpolator.
    ///
    /// Call this when there's a break in the continuity of the input data stream.
    fn reset(&mut self);

    /// Configures the interpolator from absolute sample rates.
    fn set_hz_to_hz(&mut self, _source_hz: f64, _target_hz: f64) {}

    /// Configures the interpolator from playback rate scale (`source_hz / target_hz`).
    fn set_playback_hz_scale(&mut self, _scale: f64) {}

    /// Configures the interpolator from sample rate scale (`target_hz / source_hz`).
    fn set_sample_hz_scale(&mut self, _scale: f64) {}
}
