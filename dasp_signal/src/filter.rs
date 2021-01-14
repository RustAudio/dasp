//! An extension to the **Signal** trait that enables iterative filtering.
//!
//! ### Required Features
//!
//! - When using `dasp_signal`, this item requires the **filter** feature to be enabled.
//! - When using `dasp`, this item requires the **signal-filter** feature to be enabled.

use crate::Signal;
use dasp_filter as filter;
