use crate::Interpolator;
use dasp_frame::Frame;
use dasp_sample::Duplex;

/// Interpolator that rounds off any values to the previous value from the source.
pub struct Floor<F> {
    left: F,
}

impl<F> Floor<F> {
    /// Create a new Floor Interpolator.
    pub fn new(left: F) -> Floor<F> {
        Floor { left: left }
    }
}

impl<F> Interpolator for Floor<F>
where
    F: Frame,
    F::Sample: Duplex<f64>,
{
    type Frame = F;

    fn interpolate(&self, _x: f64) -> Self::Frame {
        self.left
    }

    fn next_source_frame(&mut self, source_frame: Self::Frame) {
        self.left = source_frame;
    }
}
