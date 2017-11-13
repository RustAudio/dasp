use {Frame, Sample};
use core;
use peak;
use ring_buffer;
use rms;

/// A type that can be used to detect the envelope of a signal.
#[derive(Clone, Debug)]
pub struct Detector<F, D>
where
    F: Frame,
    D: Detect<F>,
{
    last_env_frame: D::Output,
    attack_gain: f32,
    release_gain: f32,
    detect: D,
}

/// Types that may be used to detect an envelope over a signal.
pub trait Detect<F>
where
    F: Frame,
{
    /// The result of detection.
    type Output: Frame<NumChannels = F::NumChannels>;
    /// Given some frame, return the detected envelope over each channel.
    fn detect(&mut self, frame: F) -> Self::Output;
}

/// A `Peak` detector, generic over the `FullWave`, `PositiveHalfWave`, `NegativeHalfWave`
/// rectifiers.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Peak<R = peak::FullWave> {
    rectifier: R,
}

impl<R> From<R> for Peak<R> {
    fn from(rectifier: R) -> Self {
        Peak { rectifier: rectifier }
    }
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

impl<F, S> Detect<F> for rms::Rms<F, S>
where
    F: Frame,
    S: ring_buffer::Slice<Element = F::Float>
        + ring_buffer::SliceMut,
{
    type Output = F::Float;
    fn detect(&mut self, frame: F) -> Self::Output {
        self.next(frame)
    }
}

fn calc_gain(n_frames: f32) -> f32 {
    ::ops::f32::powf32(core::f32::consts::E, -1.0 / n_frames)
}

impl<F, S> Detector<F, rms::Rms<F, S>>
where
    F: Frame,
    S: ring_buffer::Slice<Element = F::Float> + ring_buffer::SliceMut,
{
/// Construct a new **Rms** **Detector**.
    pub fn rms(buffer: ring_buffer::Fixed<S>, attack_frames: f32, release_frames: f32) -> Self {
        let rms = rms::Rms::new(buffer);
        Self::new(rms, attack_frames, release_frames)
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

impl<F, D> Detector<F, D>
where
    F: Frame,
    D: Detect<F>,
{
    fn new(detect: D, attack_frames: f32, release_frames: f32) -> Self {
        Detector {
            last_env_frame: D::Output::equilibrium(),
            attack_gain: calc_gain(attack_frames),
            release_gain: calc_gain(release_frames),
            detect: detect,
        }
    }

    /// Set the **Detector**'s attack time as a number of frames.
    pub fn set_attack_frames(&mut self, frames: f32) {
        self.attack_gain = calc_gain(frames);
    }

    /// Set the **Detector**'s release time as a number of frames.
    pub fn set_release_frames(&mut self, frames: f32) {
        self.attack_gain = calc_gain(frames);
    }

    /// Given the next input signal frame, detect and return the next envelope frame.
    pub fn next(&mut self, frame: F) -> D::Output {
        let Detector {
            attack_gain,
            release_gain,
            ref mut detect,
            ref mut last_env_frame,
        } = *self;

        let detected_frame = detect.detect(frame);
        let new_env_frame = last_env_frame.zip_map(detected_frame, |l, d| {
            let gain = if l < d { attack_gain } else { release_gain };
            let diff = l.add_amp(-d.to_signed_sample());
            d.add_amp(diff.mul_amp(gain.to_sample()).to_sample())
        });
        *last_env_frame = new_env_frame;
        new_env_frame
    }
}
