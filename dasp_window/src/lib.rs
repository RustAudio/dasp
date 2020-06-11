//! Module for windowing over a batch of Frames. Includes default Hann and Rectangle window
//! types.
//!
//! ### Optional Features
//!
//! - The **hann** feature (or **window-hann** feature if using `dasp`) provides the
//!   [**Hann**](./struct.Hann.html) window function implementation.
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

#[cfg(feature = "hann")]
pub use hann::Hann;
#[cfg(feature = "rectangle")]
pub use rectangle::Rectangle;

#[cfg(feature = "hann")]
mod hann;
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
