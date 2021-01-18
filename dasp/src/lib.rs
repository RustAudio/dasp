//! **dasp** (formerly known as ***sample***) is a suite of crates providing the fundamentals for
//! working with pulse-code modulation **digital audio signal processing**. In other words,
//! **dasp** provides a suite of low-level, high-performance tools including types, traits and
//! functions for working with digital audio signals.
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
//! - See the [**graph** module](./graph/index.html) for working with dynamic audio graphs.
//!
//! ## Optional Features
//!
//! By default, only the **sample** and **frame** modules and their respective traits are included
//! within this crate. You may pick and choose between the following features for additional
//! functionality.
//!
//! - The **all** feature enables all of the following features.
//! - The **std** feature enables the std library. This is enabled by default.
//! - The **all-no-std** feature enables all of the following features (without std).
//!
//! The following features map to each of the sub-crates and their respective features.
//!
//! - The **envelope** feature enables the `dasp_envelope` crate via the
//!   [envelope](./envelope/index.html) module.
//!     - The **envelope-peak** feature enables peak envelope detection.
//!     - The **envelope-rms** feature enables RMS envelope detection.
//! - The **graph** feature enables the `dasp_graph` crate via the [graph](./graph/index.html)
//!   module.
//!     - The **node-boxed** feature provides a `Node` implementation for `Box<dyn Node>`.
//!     - The **node-delay** feature provides a simple multi-channel `Delay` node.
//!     - The **node-graph** feature provides an implementation of `Node` for a type that encapsulates
//!       another `dasp` graph type.
//!     - The **node-pass** feature provides a `Pass` node that simply passes audio from its
//!       inputs to its outputs.
//!     - The **node-signal** feature provides an implementation of `Node` for `dyn Signal`.
//!     - The **node-sum** feature provides `Sum` and `SumBuffers` `Node` implementations.
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
//!     - The **signal-window-hann** enables the
//!       [**signal::window::hann**](./signal/window/fn.hann.html) window constructor.
//!     - The **signal-window-rectangle** enables the
//!       [**signal::window::rectangle**](./signal/window/fn.rectangle.html) window constructor.
//! - The **slice** feature enables the `dasp_slice` crate via the [slice](./slice/index.html)
//!   module.
//!     - The **slice-boxed** feature enables boxed slice conversion traits and functions.
//! - The **window** feature enables the `dasp_window` crate via the [window](./window/index.html)
//!   module.
//!     - The **window-hann** feature enables the [**Hann**](./window/struct.Hann.html)
//!       window implementation.
//!     - The **window-rectangle** feature enables the
//!       [**Rectangle**](./window/struct.Rectangle.html) window implementation.
//!
//! You can also enable all of the above features with the `--all-features` flag.
//!
//! ### no_std
//!
//! If working in a `no_std` context, you can disable the default **std** feature with
//! `--no-default-features`.
//!
//! To enable all of the above features in a `no_std` context, enable the **all-no-std** feature.
//!
//! *Note: The **graph** module is currently only available with the **std** feature enabled.
//! Adding support for `no_std` is pending the addition of support for `no_std` in petgraph. See
//! [this PR](https://github.com/petgraph/petgraph/pull/238).

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "envelope")]
#[doc(inline)]
pub use dasp_envelope as envelope;
#[doc(inline)]
pub use dasp_frame::{self as frame, Frame};
#[cfg(feature = "filter")]
#[doc(inline)]
pub use dasp_filter as filter;
// TODO: Remove `std` requirement once `dasp_graph` gains `no_std` support.
#[cfg(all(feature = "graph", feature = "std"))]
#[doc(inline)]
pub use dasp_graph as graph;
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
