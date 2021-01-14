
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(core_intrinsics))]

use dasp_frame::Frame;
use dasp_sample::{FloatSample, FromSample, ToSample};

/// Coefficients for a digital biquad filter.
/// It is assumed that the `a0` coefficient is always normalized to 1.0,
/// and thus not included.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Coefficients<S>
where
    S: FloatSample,
{
    // Numerator coefficients.
    pub b0: S,
    pub b1: S,
    pub b2: S,

    // Denominator coefficients.
    pub a1: S,
    pub a2: S,
}

/// An implementation of a digital biquad filter.
pub struct Biquad<S, F>
where
    S: FloatSample,
    F: Frame<Sample = S>,
{
    coeff: Coefficients<S>,

    // Since biquad filters are second-order, we require two historical buffers.
    m1: F,
    m2: F,
}

impl<S, F> Biquad<S, F>
where
    S: FloatSample,
    F: Frame<Sample = S>,
{
    pub fn new(coeff: Coefficients<S>) -> Self {
        Self {
            coeff,
            m1: F::EQUILIBRIUM,
            m2: F::EQUILIBRIUM,
        }
    }

    pub fn apply<I>(&mut self, input: I) -> I
    where
        I: Frame<NumChannels = F::NumChannels>,
        I::Sample: ToSample<S> + FromSample<S>,
    {
        let input: F = input.map(ToSample::to_sample_);

        // Alias to make calculations less verbose.
        let co = &self.coeff;

        let output = self.m1.add_amp(input.scale_amp(co.b0));

        // Update buffers, which depend on new output.
        self.m1 = self.m2.add_amp(input.scale_amp(co.b1).add_amp(output.scale_amp(-co.a1)));
        self.m2 = input.scale_amp(co.b2).add_amp(output.scale_amp(-co.a2));

        output.map(FromSample::from_sample_)
    }
}
