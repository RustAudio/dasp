//! Peak envelope detection over a signal.

#![cfg_attr(not(feature = "std"), no_std)]

use dasp_frame::Frame;
use dasp_sample::Sample;

/// Types that may be used to rectify a signal of frames `F` for a `Peak` detector.
pub trait Rectifier<F>
where
    F: Frame,
{
    /// Frames that can be detected.
    type Output: Frame<NumChannels = F::NumChannels>;
    /// Rectify the given frame.
    fn rectify(&mut self, frame: F) -> Self::Output;
}

/// A signal rectifier that produces the absolute amplitude from samples.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct FullWave;
/// A signal rectifier that produces only the positive samples.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct PositiveHalfWave;
/// A signal rectifier that produces only the negative samples.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct NegativeHalfWave;

impl<F> Rectifier<F> for FullWave
where
    F: Frame,
{
    type Output = F::Signed;
    fn rectify(&mut self, frame: F) -> Self::Output {
        full_wave(frame)
    }
}

impl<F> Rectifier<F> for PositiveHalfWave
where
    F: Frame,
{
    type Output = F;
    fn rectify(&mut self, frame: F) -> Self::Output {
        positive_half_wave(frame)
    }
}

impl<F> Rectifier<F> for NegativeHalfWave
where
    F: Frame,
{
    type Output = F;
    fn rectify(&mut self, frame: F) -> Self::Output {
        negative_half_wave(frame)
    }
}

/// A signal rectifier that produces the absolute amplitude from samples.
pub fn full_wave<F>(frame: F) -> F::Signed
where
    F: Frame,
{
    frame.map(|s| {
        let signed = s.to_signed_sample();
        if signed < Sample::equilibrium() {
            -signed
        } else {
            signed
        }
    })
}

/// A signal rectifier that produces only the positive samples.
pub fn positive_half_wave<F>(frame: F) -> F
where
    F: Frame,
{
    frame.map(|s| if s < Sample::equilibrium() {
        Sample::equilibrium()
    } else {
        s
    })
}

/// A signal rectifier that produces only the negative samples.
pub fn negative_half_wave<F>(frame: F) -> F
where
    F: Frame,
{
    frame.map(|s| if s > Sample::equilibrium() {
        Sample::equilibrium()
    } else {
        s
    })
}
