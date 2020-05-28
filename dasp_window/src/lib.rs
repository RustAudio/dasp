//! Module for windowing over a batch of Frames. Includes default Hanning and Rectangle window
//! types.

use dasp_sample::Sample;

#[cfg(feature = "hanning")]
pub mod hanning;
#[cfg(feature = "rectangle")]
pub mod rectangle;

/// The window function used within a `Window`.
pub trait Window {
    /// Returns the amplitude for the given phase, given as some `Sample` type.
    fn at_phase<S: Sample>(phase: S) -> S;
}
