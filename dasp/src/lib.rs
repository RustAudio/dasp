//! **dasp** is a suite of crates, providing the fundamentals for working with pulse-code
//! modulation digital signal processing. In other words, `dasp` provides a suite of low-level,
//! high-performance tools including types, traits and functions for working with digital audio
//! signals.
//!
//! Each of the **dasp** crates are re-exported under their respective
//! [modules](file:///home/mindtree/programming/rust/dasp/target/doc/dasp/index.html#modules).
//!
//! ## Highlights
//!
//! The following are some of the more interesting items within the dasp collection:
//!
//! - Use the [**Sample** trait](./trait.Sample.html) to remain generic across bit-depth.
//! - Use the [**Frame** trait](./frame/trait.Frame.html) to remain generic over channel layout.
//! - Use the [**Signal** trait](./signal/trait.Signal.html) for working with **Iterators** that
//!   yield **Frames**.
//! - See the [**signal** module](./signal/index.html) for a collection of interesting signal
//!   constructors (e.g. `sine`, `noise`, `from_iter`, etc).
//! - Use the [**slice** module](./slice/index.html) for working with slices of **Samples** and
//!   **Frames**.
//! - See the [**sample::types** module](./sample/types/index.html) for provided custom sample
//!   types.
//! - See the [**Converter** type](./signal/interpolate/struct.Converter.html) for sample rate
//!   conversion and scaling.
//! - See the [**ring_buffer** module](./ring_buffer/index.html) for fast FIFO queue options.
//!
//! ## Optional Features
//!
//! By default, only the **sample** and **frame** modules and their respective traits are included
//! within this crate. You may pick and choose between the following features for additional
//! functionality.
//!
//! - The **envelope** feature enables the `dasp_envelope` crate via the
//!   [envelope](./envelope/index.html) module.
//!     - The **envelope-peak** feature enables peak envelope detection.
//!     - The **envelope-rms** feature enables RMS envelope detection.
//! - The **interpolate** feature enables the `dasp_interpolate` crate via the
//!   [interpolate](./interpolate/index.html) module.
//!     - The **interpolate-floor** feature enables a floor interpolation implementation.
//!     - The **interpolate-linear** feature enables a linear interpolation implementation.
//!     - The **interpolate-sinc** feature enables a sinc interpolation implementation.
//! - The **peak** feature enables the `dasp_peak` crate via the [peak](./peak/index.html) module.
//! - The **ring_buffer** feature enables the `dasp_ring_buffer` crate via the
//!   [ring_buffer](./peak/index.html) module.
//! - The **rms** feature enables the `dasp_rms` crate via the [rms](./rms/index.html) module.
//! - The **signal** feature enables the `dasp_signal` crate via the [signal](./signal/index.html)
//!   module.
//!     - The **signal-boxed** feature enables an implementation of **Signal** for `Box<dyn
//!       Signal>`.
//!     - The **signal-bus** feature enables the [**SignalBus**](./signal/bus/trait.SignalBus.html)
//!       trait.
//!     - The **signal-envelope** feature enables the
//!       [**SignalEnvelope**](./signal/envelope/trait.SignalEnvelope.html) trait.
//!     - The **signal-rms** feature enables the [**SignalRms**](./signal/rms/trait.SignalRms.html)
//!       trait.
//!     - The **signal-window** feature enables the
//!       [**signal::window**](./signal/window/index.html) module.
//!     - The **signal-window-hanning** enables the
//!       [**signal::window::hanning**](./signal/window/fn.hanning.html) window constructor.
//!     - The **signal-window-rectangle** enables the
//!       [**signal::window::rectangle**](./signal/window/fn.rectangle.html) window constructor.
//! - The **slice** feature enables the `dasp_slice` crate via the [slice](./slice/index.html)
//!   module.
//!     - The **slice-boxed** feature enables boxed slice conversion traits and functions.
//! - The **window** feature enables the `dasp_window` crate via the [window](./window/index.html)
//!   module.
//!     - The **window-hanning** feature enables the [**Hanning**](./window/struct.Hanning.html)
//!       window implementation.
//!     - The **window-rectangle** feature enables the
//!       [**Rectangle**](./window/struct.Rectangle.html) window implementation.

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
#[cfg(feature = "rms")]
#[doc(inline)]
pub use dasp_rms as rms;
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
