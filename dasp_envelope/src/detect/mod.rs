use dasp_frame::Frame;
use dasp_sample::Sample;
use ops::f32::powf32;

#[cfg(feature = "peak")]
pub use self::peak::Peak;

mod ops;
#[cfg(feature = "peak")]
mod peak;
#[cfg(feature = "rms")]
mod rms;

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

fn calc_gain(n_frames: f32) -> f32 {
    if n_frames == 0.0 {
        0.0
    } else {
        powf32(core::f32::consts::E, -1.0 / n_frames)
    }
}

impl<F, D> Detector<F, D>
where
    F: Frame,
    D: Detect<F>,
{
    /// Construct a **Detector** with the given **Detect** implementation.
    pub fn new(detect: D, attack_frames: f32, release_frames: f32) -> Self {
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
        self.release_gain = calc_gain(frames);
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
