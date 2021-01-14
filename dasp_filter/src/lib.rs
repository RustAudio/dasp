
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
    // Numerator coefficients (normally b0, b1, b2).
    pub n0: S,
    pub n1: S,
    pub n2: S,

    // Denominator coefficients (normally a1, a2).
    pub d1: S,
    pub d2: S,
}

/// An implementation of a digital biquad filter, using the Direct Form 2
/// Transposed (DF2T) representation.
pub struct Biquad<F>
where
    F: Frame,
    F::Sample: FloatSample,
{
    pub coeff: Coefficients<F::Sample>,

    // Since biquad filters are second-order, we require two historical buffers.
    // This state is updated each time the filter is applied to a `Frame`.
    t0: F,
    t1: F,
}

impl<F> Biquad<F>
where
    F: Frame,
    F::Sample: FloatSample,
{
    pub fn new(coeff: Coefficients<F::Sample>) -> Self {
        Self {
            coeff,
            t0: F::EQUILIBRIUM,
            t1: F::EQUILIBRIUM,
        }
    }

    /// Performs a single iteration of this filter, calculating a new filtered
    /// `Frame` from an input `Frame`.
    pub fn apply<I>(&mut self, input: I) -> I
    where
        I: Frame<NumChannels = F::NumChannels>,
        I::Sample: ToSample<F::Sample> + FromSample<F::Sample>,
    {
        // Convert into floating point representation.
        let input: F = input.map(ToSample::to_sample_);

        // Calculate scaled inputs.
        let input_by_n0 = input.scale_amp(self.coeff.n0);
        let input_by_n1 = input.scale_amp(self.coeff.n1);
        let input_by_n2 = input.scale_amp(self.coeff.n2);

        // This is the new filtered `Frame`.
        let output: F = self.t0.add_amp(input_by_n0);

        // Calculate scaled outputs.
        // NOTE: Negative signs on the scaling factors for these.
        let output_by_neg_d1 = output.scale_amp(-self.coeff.d1);
        let output_by_neg_d2 = output.scale_amp(-self.coeff.d2);

        // Update buffers.
        self.t0 = self.t1.add_amp(input_by_n1).add_amp(output_by_neg_d1);
        self.t1 = input_by_n2.add_amp(output_by_neg_d2);

        // Convert back into the original `Frame` format.
        output.map(FromSample::from_sample_)
    }
}
