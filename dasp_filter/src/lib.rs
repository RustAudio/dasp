
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(core_intrinsics))]

use dasp_sample::FloatSample;

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


