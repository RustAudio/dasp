//! Module for windowing over a batch of Frames. Includes default Hanning and Rectangle window
//! types.
//!
//! ### Optional Features
//!
//! - The **hanning** feature (or **window-hanning** feature if using `dasp`) provides the
//!   [**Hanning**](./struct.Hanning.html) window function implementation.
//! - The **rectangle** feature (or **window-rectangle** feature if using `dasp`) provides the
//!   [**Rectangle**](./struct.Rectangle.html) window function implementation.
//!
//! ### no_std
//!
//! If working in a `no_std` context, you can disable the default **std** feature with
//! `--no-default-features`.
//!
//! To enable all of the above features in a `no_std` context, enable the **all-no-std** feature.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(core_intrinsics))]

#[cfg(feature = "hanning")]
pub use hanning::Hanning;
#[cfg(feature = "rectangle")]
pub use rectangle::Rectangle;

#[cfg(feature = "hanning")]
mod hanning;
#[cfg(feature = "rectangle")]
mod rectangle;

/// An abstraction supporting different types of `Window` functions.
///
/// The type `S` represents the phase of the window, while the `Output` represents the window
/// amplitude.
pub trait Window<S> {
    /// The type used to represent the window amplitude.
    type Output;
    /// Returns the amplitude for the given phase, given as some `Sample` type.
    fn window(phase: S) -> Self::Output;
}
