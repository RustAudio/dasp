use crate::Signal;
use dasp_envelope as envelope;

pub trait SignalEnvelope: Signal {
    /// An adaptor that detects and yields the envelope of the signal.
    ///
    /// # Example
    ///
    /// ```
    /// use dasp_envelope as envelope;
    /// use dasp_signal::{self as signal, Signal};
    /// use dasp_signal::envelope::SignalEnvelope;
    ///
    /// fn main() {
    ///     let signal = signal::rate(4.0).const_hz(1.0).sine();
    ///     let attack = 1.0;
    ///     let release = 1.0;
    ///     let detector = envelope::Detector::peak(attack, release);
    ///     let mut envelope = signal.detect_envelope(detector);
    ///     assert_eq!(
    ///         envelope.take(4).collect::<Vec<_>>(),
    ///         vec![[0.0], [0.6321205496788025], [0.23254416035257117], [0.7176687675647109]]
    ///     );
    /// }
    /// ```
    fn detect_envelope<D>(
        self,
        detector: envelope::Detector<Self::Frame, D>,
    ) -> DetectEnvelope<Self, D>
    where
        Self: Sized,
        D: envelope::Detect<Self::Frame>,
    {
        DetectEnvelope {
            signal: self,
            detector: detector,
        }
    }
}

/// An adaptor that detects and yields the envelope of the signal.
#[derive(Clone)]
pub struct DetectEnvelope<S, D>
where
    S: Signal,
    D: envelope::Detect<S::Frame>,
{
    signal: S,
    detector: envelope::Detector<S::Frame, D>,
}

impl<S, D> DetectEnvelope<S, D>
where
    S: Signal,
    D: envelope::Detect<S::Frame>,
{
    /// Set the **Detector**'s attack time as a number of frames.
    pub fn set_attack_frames(&mut self, frames: f32) {
        self.detector.set_attack_frames(frames);
    }

    /// Set the **Detector**'s release time as a number of frames.
    pub fn set_release_frames(&mut self, frames: f32) {
        self.detector.set_release_frames(frames);
    }

    /// Consumes `Self` and returns the inner signal `S` and `Detector`.
    pub fn into_parts(self) -> (S, envelope::Detector<S::Frame, D>) {
        let DetectEnvelope { signal, detector } = self;
        (signal, detector)
    }
}

impl<S, D> Signal for DetectEnvelope<S, D>
where
    S: Signal,
    D: envelope::Detect<S::Frame>,
{
    type Frame = D::Output;
    fn next(&mut self) -> Self::Frame {
        self.detector.next(self.signal.next())
    }

    fn is_exhausted(&self) -> bool {
        self.signal.is_exhausted()
    }
}

impl<T> SignalEnvelope for T where T: Signal {}
