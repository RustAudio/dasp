//! A Hann-windowed sinc interpolator implementation.
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

/// Interpolator for Hann-windowed sinc interpolation.
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
    bandwidth: f64,
}

impl<S> Sinc<S> {
    /// Create a new **Sinc** (Hann-windowed) interpolator with the given ring buffer.
    ///
    /// The given ring buffer should have a length twice that of the desired interpolation `depth`.
    ///
    /// The initial contents of the ring buffer will act as padding for the interpolated signal.
    ///
    /// ### Panics
    ///
    /// If the given ring buffer's length is not a multiple of `2`.
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
            frames,
            idx: 0,
            bandwidth: 1.0,
        }
    }

    /// Get the interpolation depth (number of taps per side).
    pub fn depth(&self) -> usize
    where
        S: ring_buffer::Slice,
    {
        self.frames.len() / 2
    }

    /// Compute the Hann-windowed sinc coefficient for a given time offset.
    fn windowed_sinc(&self, t: f64, depth: usize, bandwidth: f64) -> f64 {
        let a = PI * bandwidth * t;
        let b = PI * t / depth as f64;
        let sinc = if a.abs() < f64::EPSILON {
            bandwidth
        } else {
            bandwidth * sin(a) / a
        };
        let hann_window = 0.5 + 0.5 * cos(b);
        sinc * hann_window
    }

    /// Accumulate a single tap of the Hann-windowed sinc filter.
    fn accumulate_tap(
        &self,
        accum: S::Element,
        frame_idx: usize,
        phase: f64,
        depth: usize,
        bandwidth: f64,
    ) -> S::Element
    where
        S: ring_buffer::Slice,
        S::Element: Frame,
        <S::Element as Frame>::Sample: Duplex<f64>,
    {
        let coeff = self.windowed_sinc(phase, depth, bandwidth);
        accum.zip_map(self.frames[frame_idx], |vs, frame_sample| {
            vs.add_amp(
                (coeff * frame_sample.to_sample::<f64>())
                    .to_sample::<<S::Element as Frame>::Sample>()
                    .to_signed_sample(),
            )
        })
    }
}

impl<S> Interpolator for Sinc<S>
where
    S: ring_buffer::SliceMut,
    S::Element: Frame,
    <S::Element as Frame>::Sample: Duplex<f64>,
{
    type Frame = S::Element;

    /// Hann-windowed sinc interpolation
    fn interpolate(&self, x: f64) -> Self::Frame {
        let phil = x;
        let phir = 1.0 - x;
        let nl = self.idx;
        let nr = self.idx + 1;
        let depth = self.depth();
        let bandwidth = self.bandwidth;

        let rightmost = nl + depth;
        let leftmost = nr as isize - depth as isize;
        let max_depth = if rightmost >= self.frames.len() {
            self.frames.len() - depth
        } else if leftmost < 0 {
            (depth as isize + leftmost) as usize
        } else {
            depth
        };

        (0..max_depth).fold(Self::Frame::EQUILIBRIUM, |v, n| {
            let v = self.accumulate_tap(v, nl - n, phil + n as f64, depth, bandwidth);
            self.accumulate_tap(v, nr + n, phir + n as f64, depth, bandwidth)
        })
    }

    fn next_source_frame(&mut self, source_frame: Self::Frame) {
        let _old_frame = self.frames.push(source_frame);
        if self.idx < self.depth() {
            self.idx += 1;
        }
    }

    fn reset(&mut self) {
        self.idx = 0;
        self.frames.set_first(0);
        for frame in self.frames.iter_mut() {
            *frame = Self::Frame::EQUILIBRIUM;
        }
    }

    fn set_bandwidth(&mut self, bandwidth: f64) {
        self.bandwidth = bandwidth.clamp(f64::MIN_POSITIVE, 1.0);
    }
}
