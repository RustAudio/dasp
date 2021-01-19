//! A sinc interpolator implementation.
//!
//! ### Required Features
//!
//! - When using `dasp_interpolate`, this module requires the **sinc** feature to be enabled.
//! - When using `dasp`, this module requires the **interpolate-sinc** feature to be enabled.

use crate::Interpolator;
use core::f64::consts::PI;
use dasp_frame::Frame;
use dasp_ring_buffer as ring_buffer;
use dasp_sample::{Duplex, Sample};
use ops::f64::{cos, sin};

mod ops;

/// Interpolator for sinc interpolation.
///
/// Generally accepted as one of the better sample rate converters, although it uses significantly
/// more computation.
///
/// ### Required Features
///
/// - When using `dasp_interpolate`, this item requires the **sinc** feature to be enabled.
/// - When using `dasp`, this item requires the **interpolate-sinc** feature to be enabled.
pub struct Sinc<S> {
    frames: ring_buffer::Fixed<S>,
    idx: usize,
}

impl<S> Sinc<S> {
    /// Create a new **Sinc** interpolator with the given ring buffer.
    ///
    /// The given ring buffer should have a length twice that of the desired sinc interpolation
    /// `depth`.
    ///
    /// The initial contents of the ring_buffer will act as padding for the interpolated signal.
    ///
    /// **panic!**s if the given ring buffer's length is not a multiple of `2`.
    ///
    /// ### Required Features
    ///
    /// - When using `dasp_interpolate`, this item requires the **sinc** feature to be enabled.
    /// - When using `dasp`, this item requires the **interpolate-sinc** feature to be enabled.
    pub fn new(frames: ring_buffer::Fixed<S>) -> Self
    where
        S: ring_buffer::SliceMut,
        S::Element: Frame,
    {
        assert!(frames.len() % 2 == 0);
        Sinc {
            frames: frames,
            idx: 0,
        }
    }

    fn depth(&self) -> usize
    where
        S: ring_buffer::Slice,
    {
        self.frames.len() / 2
    }
}

impl<S> Interpolator for Sinc<S>
where
    S: ring_buffer::SliceMut,
    S::Element: Frame,
    <S::Element as Frame>::Sample: Duplex<f64>,
{
    type Frame = S::Element;

    /// Sinc interpolation
    fn interpolate(&self, x: f64) -> Self::Frame {
        let phil = x;
        let phir = 1.0 - x;
        let nl = self.idx;
        let nr = self.idx + 1;
        let depth = self.depth();

        let rightmost = nl + depth;
        let leftmost = nr as isize - depth as isize;
        let max_depth = if rightmost >= self.frames.len() {
            self.frames.len() - depth
        } else if leftmost < 0 {
            (depth as isize + leftmost) as usize
        } else {
            depth
        };

        (0..max_depth).fold(Self::Frame::EQUILIBRIUM, |mut v, n| {
            v = {
                let a = PI * (phil + n as f64);
                let first = if a == 0.0 { 1.0 } else { sin(a) / a };
                let second = 0.5 + 0.5 * cos(a / depth as f64);
                v.zip_map(self.frames[nl - n], |vs, r_lag| {
                    vs.add_amp(
                        (first * second * r_lag.to_sample::<f64>())
                            .to_sample::<<Self::Frame as Frame>::Sample>()
                            .to_signed_sample(),
                    )
                })
            };

            let a = PI * (phir + n as f64);
            let first = if a == 0.0 { 1.0 } else { sin(a) / a };
            let second = 0.5 + 0.5 * cos(a / depth as f64);
            v.zip_map(self.frames[nr + n], |vs, r_lag| {
                vs.add_amp(
                    (first * second * r_lag.to_sample::<f64>())
                        .to_sample::<<Self::Frame as Frame>::Sample>()
                        .to_signed_sample(),
                )
            })
        })
    }

    fn next_source_frame(&mut self, source_frame: Self::Frame) {
        let _old_frame = self.frames.push(source_frame);
        if self.idx < self.depth() {
            self.idx += 1;
        }
    }
}
