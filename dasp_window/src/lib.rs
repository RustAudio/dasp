//! Module for windowing over a batch of Frames. Includes default Hanning and Rectangle window
//! types.
//!
//! ### Optional Features
//!
//! - The **hanning** feature (or **window-hanning** feature if using `dasp`) provides the
//!   [**Hanning**](./struct.Hanning.html) window function implementation.
//! - The **rectangle** feature (or **window-rectangle** feature if using `dasp`) provides the
//!   [**Rectangle**](./struct.Rectangle.html) window function implementation.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(core_intrinsics))]

use dasp_sample::Sample;

#[cfg(feature = "hanning")]
pub use hanning::Hanning;
#[cfg(feature = "rectangle")]
pub use rectangle::Rectangle;

#[cfg(feature = "hanning")]
mod hanning;
#[cfg(feature = "rectangle")]
mod rectangle;

/// The window function used within a `Window`.
pub trait Window {
    /// Returns the amplitude for the given phase, given as some `Sample` type.
    fn at_phase<S: Sample>(phase: S) -> S;
}
