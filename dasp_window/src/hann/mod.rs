use crate::Window;
use dasp_sample::Sample;
use ops::f64::cos;

mod ops;

/// A type of window function, also known as the "raised cosine window".
///
/// [Wiki entry](https://en.wikipedia.org/wiki/Hann_function).
///
/// ### Required Features
///
/// - When using `dasp_window`, this item requires the **hann** feature to be enabled.
/// - When using `dasp`, this item requires the **window-hann** feature to be enabled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hann;

impl<S> Window<S> for Hann
where
    S: Sample,
{
    type Output = S;
    fn window(phase: S) -> Self::Output {
        const PI_2: f64 = core::f64::consts::PI * 2.0;
        let v = phase.to_float_sample().to_sample() * PI_2;
        (0.5 * (1.0 - cos(v)))
            .to_sample::<S::Float>()
            .to_sample::<S>()
    }
}
