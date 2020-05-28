use crate::Window;

use dasp_sample::{FloatSample, Sample};

/// The simplest window type, equivalent to replacing all but *N* values of data sequence by
/// zeroes, making it appear as though the waveform suddenly turns on and off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rectangle;

impl Window for Rectangle {
    fn at_phase<S: Sample>(_phase: S) -> S {
        <S::Float as FloatSample>::identity().to_sample::<S>()
    }
}
