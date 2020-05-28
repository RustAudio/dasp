//! A crate of fundamentals for audio PCM DSP.
//!
//! - Use the [**Sample** trait](./trait.Sample.html) to remain generic across bit-depth.
//! - Use the [**Frame** trait](./frame/trait.Frame.html) to remain generic over channel layout.
//! - Use the [**Signal** trait](./signal/trait.Signal.html) for working with **Iterators** that yield **Frames**.
//! - Use the [**slice** module](./slice/index.html) for working with slices of **Samples** and **Frames**.
//! - See the [**conv** module](./conv/index.html) for fast conversions between slices, frames and samples.
//! - See the [**types** module](./types/index.html) for provided custom sample types.
//! - See the [**interpolate** module](./interpolate/index.html) for sample rate conversion and scaling.
//! - See the [**ring_buffer** module](./ring_buffer/index.html) for fast FIFO queue options.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "envelope")]
#[doc(inline)]
pub use dasp_envelope as envelope;
#[doc(inline)]
pub use dasp_frame::{self as frame, Frame};
#[cfg(feature = "interpolate")]
#[doc(inline)]
pub use dasp_interpolate as interpolate;
#[cfg(feature = "peak")]
#[doc(inline)]
pub use dasp_peak as peak;
#[cfg(feature = "ring_buffer")]
#[doc(inline)]
pub use dasp_ring_buffer as ring_buffer;
#[doc(inline)]
pub use dasp_sample::{self as sample, Sample};
#[cfg(feature = "signal")]
#[doc(inline)]
pub use dasp_signal::{self as signal, Signal};
#[cfg(feature = "slice")]
#[doc(inline)]
pub use dasp_slice as slice;
#[cfg(feature = "signal")]
#[doc(inline)]
pub use dasp_window as window;
