use crate::{Detect, Detector};
use dasp_frame::Frame;
use dasp_peak as peak;

/// A `Peak` detector, generic over the `FullWave`, `PositiveHalfWave`, `NegativeHalfWave`
/// rectifiers.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Peak<R = peak::FullWave> {
    rectifier: R,
}

impl Peak<peak::FullWave> {
    /// A signal rectifier that produces the absolute amplitude from samples.
    pub fn full_wave() -> Self {
        peak::FullWave.into()
    }
}

impl Peak<peak::PositiveHalfWave> {
    /// A signal rectifier that produces only the positive samples.
    pub fn positive_half_wave() -> Self {
        peak::PositiveHalfWave.into()
    }
}

impl Peak<peak::NegativeHalfWave> {
    /// A signal rectifier that produces only the negative samples.
    pub fn negative_half_wave() -> Self {
        peak::NegativeHalfWave.into()
    }
}

impl<F, R> Detector<F, Peak<R>>
where
    F: Frame,
    R: peak::Rectifier<F>,
{
    /// Construct a new **Peak** **Detector** that uses the given rectifier.
    pub fn peak_from_rectifier(rectifier: R, attack_frames: f32, release_frames: f32) -> Self {
        let peak = rectifier.into();
        Self::new(peak, attack_frames, release_frames)
    }
}

impl<F> Detector<F, Peak<peak::FullWave>>
where
    F: Frame,
{
    /// Construct a new full wave **Peak** **Detector**.
    pub fn peak(attack_frames: f32, release_frames: f32) -> Self {
        let peak = Peak::full_wave();
        Self::new(peak, attack_frames, release_frames)
    }
}

impl<F> Detector<F, Peak<peak::PositiveHalfWave>>
where
    F: Frame,
{
    /// Construct a new positive half wave **Peak** **Detector**.
    pub fn peak_positive_half_wave(attack_frames: f32, release_frames: f32) -> Self {
        let peak = Peak::positive_half_wave();
        Self::new(peak, attack_frames, release_frames)
    }
}

impl<F> Detector<F, Peak<peak::NegativeHalfWave>>
where
    F: Frame,
{
    /// Construct a new positive half wave **Peak** **Detector**.
    pub fn peak_negative_half_wave(attack_frames: f32, release_frames: f32) -> Self {
        let peak = Peak::negative_half_wave();
        Self::new(peak, attack_frames, release_frames)
    }
}

impl<F, R> Detect<F> for Peak<R>
where
    F: Frame,
    R: peak::Rectifier<F>,
{
    type Output = R::Output;
    fn detect(&mut self, frame: F) -> Self::Output {
        self.rectifier.rectify(frame)
    }
}

impl<R> From<R> for Peak<R> {
    fn from(rectifier: R) -> Self {
        Peak {
            rectifier: rectifier,
        }
    }
}
