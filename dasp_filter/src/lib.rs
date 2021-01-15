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
    // Transfer function numerator coefficients.
    pub b0: S,
    pub b1: S,
    pub b2: S,

    // Transfer function denominator coefficients.
    pub a1: S,
    pub a2: S,
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
    ///
    /// ```rust
    /// use dasp_filter::{Coefficients, Biquad};
    ///
    /// fn main() {
    ///     // Notch boost filter.
    ///     let co = Coefficients {
    ///         b0: 1.0469127398708575f64,
    ///         b1: -0.27732002669854483,
    ///         b2: 0.8588151488168104,
    ///         a1: -0.27732002669854483,
    ///         a2: 0.9057278886876682,
    ///     };
    ///
    ///     // Note that this type argument defines the format of the temporary
    ///     // values, as well as the number of channels required for input
    ///     // `Frame`s.
    ///     let mut b = Biquad::<[f64; 2]>::new(co);
    ///
    ///     assert_eq!(b.apply([32i8, -64]), [33, -67]);
    ///     assert_eq!(b.apply([0.1f32, -0.3]), [0.107943736, -0.32057875]);
    /// }
    /// ```
    pub fn apply<I>(&mut self, input: I) -> I
    where
        I: Frame<NumChannels = F::NumChannels>,
        I::Sample: ToSample<F::Sample> + FromSample<F::Sample>,
    {
        // Convert into floating point representation.
        let input: F = input.map(ToSample::to_sample_);

        // Calculate scaled inputs.
        let input_by_b0 = input.scale_amp(self.coeff.b0);
        let input_by_b1 = input.scale_amp(self.coeff.b1);
        let input_by_b2 = input.scale_amp(self.coeff.b2);

        // This is the new filtered `Frame`.
        let output: F = self.t0.add_amp(input_by_b0);

        // Calculate scaled outputs.
        // NOTE: Negative signs on the scaling factors for these.
        let output_by_neg_a1 = output.scale_amp(-self.coeff.a1);
        let output_by_neg_a2 = output.scale_amp(-self.coeff.a2);

        // Update buffers.
        self.t0 = self.t1.add_amp(input_by_b1).add_amp(output_by_neg_a1);
        self.t1 = input_by_b2.add_amp(output_by_neg_a2);

        // Convert back into the original `Frame` format.
        output.map(FromSample::from_sample_)
    }
}

impl<F> From<Coefficients<F::Sample>> for Biquad<F>
where
    F: Frame,
    F::Sample: FloatSample,
{
    // Same as `new()`, but adding this for the blanket `Into` impl.
    fn from(coeff: Coefficients<F::Sample>) -> Self {
        Self::new(coeff)
    }
}
