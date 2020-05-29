use crate::Window;

use dasp_sample::{FloatSample, Sample};

/// The simplest window type, equivalent to replacing all but *N* values of data sequence by
/// zeroes, making it appear as though the waveform suddenly turns on and off.
///
/// ### Required Features
///
/// - When using `dasp_window`, this item requires the **rectangle** feature to be enabled.
/// - When using `dasp`, this item requires the **window-rectangle** feature to be enabled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rectangle;

impl<S> Window<S> for Rectangle
where
    S: Sample,
{
    type Output = S;
    fn window(_phase: S) -> Self::Output {
        <S::Float as FloatSample>::identity().to_sample::<S>()
    }
}
