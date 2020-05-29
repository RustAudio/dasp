//! An abstraction supporting different kinds of envelope detection.
//!
//! - The [**Detect**](./trait.Detect.html) trait provides an abstraction for generalising over
//!   types of envelope detection.
//! - The [**Detector**](./struct.Detector.html) type allows for applying a **Detect**
//!   implementation in order to detect the envelope of a signal.
//!
//! See the `dasp_signal` crate (or `dasp::signal` module) **SignalWindow** trait for a convenient
//! way to detect envelopes over arbitrary signals.
//!
//! ### Optional Features
//!
//! - The **peak** feature (or **envelope-peak** feature if using `dasp`) provides a peak envelope
//!   detection implementation.
//! - The **rms** feature (or **envelope-rms** feature if using `dasp`) provides an RMS envelope
//!   detection implementation.
//!
//! ### no_std
//!
//! If working in a `no_std` context, you can disable the default **std** feature with
//! `--no-default-features`.
//!
//! To enable all of the above features in a `no_std` context, enable the **all-no-std** feature.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(core_intrinsics))]

pub mod detect;

pub use self::detect::{Detect, Detector};
