use crate::Signal;
use dasp_interpolate::Interpolator;

/// An iterator that converts the rate at which frames are yielded from some given frame
/// Interpolator into a new type.
///
/// Other names for `sample::interpolate::Converter` might include:
///
/// - Sample rate converter
/// - {Up/Down}sampler
/// - Sample interpolater
/// - Sample decimator
///
#[derive(Clone)]
pub struct Converter<S, I>
where
    S: Signal,
    I: Interpolator,
{
    source: S,
    interpolator: I,
    interpolation_value: f64,
    source_to_target_ratio: f64,
}

impl<S, I> Converter<S, I>
where
    S: Signal,
    I: Interpolator,
{
    /// Construct a new `Converter` from the source frames and the source and target sample rates
    /// (in Hz).
    #[inline]
    pub fn from_hz_to_hz(source: S, interpolator: I, source_hz: f64, target_hz: f64) -> Self {
        Self::scale_playback_hz(source, interpolator, source_hz / target_hz)
    }

    /// Construct a new `Converter` from the source frames and the amount by which the current
    /// ***playback*** **rate** (not sample rate) should be multiplied to reach the new playback
    /// rate.
    ///
    /// For example, if our `source_frames` is a sine wave oscillating at a frequency of 2hz and
    /// we wanted to convert it to a frequency of 3hz, the given `scale` should be `1.5`.
    #[inline]
    pub fn scale_playback_hz(source: S, interpolator: I, scale: f64) -> Self {
        assert!(
            scale > 0.0,
            "We can't yield any frames at 0 times a second!"
        );
        Converter {
            source: source,
            interpolator: interpolator,
            interpolation_value: 0.0,
            source_to_target_ratio: scale,
        }
    }

    /// Construct a new `Converter` from the source frames and the amount by which the current
    /// ***sample*** **rate** (not playback rate) should be multiplied to reach the new sample
    /// rate.
    ///
    /// If our `source_frames` are being sampled at a rate of 44_100hz and we want to
    /// convert to a sample rate of 96_000hz, the given `scale` should be `96_000.0 / 44_100.0`.
    ///
    /// This is the same as calling `Converter::scale_playback_hz(source_frames, 1.0 / scale)`.
    #[inline]
    pub fn scale_sample_hz(source: S, interpolator: I, scale: f64) -> Self {
        Self::scale_playback_hz(source, interpolator, 1.0 / scale)
    }

    /// Update the `source_to_target_ratio` internally given the source and target hz.
    ///
    /// This method might be useful for changing the sample rate during playback.
    #[inline]
    pub fn set_hz_to_hz(&mut self, source_hz: f64, target_hz: f64) {
        self.set_playback_hz_scale(source_hz / target_hz)
    }

    /// Update the `source_to_target_ratio` internally given a new **playback rate** multiplier.
    ///
    /// This method is useful for dynamically changing rates.
    #[inline]
    pub fn set_playback_hz_scale(&mut self, scale: f64) {
        self.source_to_target_ratio = scale;
    }

    /// Update the `source_to_target_ratio` internally given a new **sample rate** multiplier.
    ///
    /// This method is useful for dynamically changing rates.
    #[inline]
    pub fn set_sample_hz_scale(&mut self, scale: f64) {
        self.set_playback_hz_scale(1.0 / scale);
    }

    /// Borrow the `source_frames` Interpolator from the `Converter`.
    #[inline]
    pub fn source(&self) -> &S {
        &self.source
    }

    /// Mutably borrow the `source_frames` Iterator from the `Converter`.
    #[inline]
    pub fn source_mut(&mut self) -> &mut S {
        &mut self.source
    }

    /// Drop `self` and return the internal `source_frames` Iterator.
    #[inline]
    pub fn into_source(self) -> S {
        self.source
    }
}

impl<S, I> Signal for Converter<S, I>
where
    S: Signal,
    I: Interpolator<Frame = S::Frame>,
{
    type Frame = S::Frame;

    fn next(&mut self) -> Self::Frame {
        let Converter {
            ref mut source,
            ref mut interpolator,
            ref mut interpolation_value,
            source_to_target_ratio,
        } = *self;

        // Advance frames
        while *interpolation_value >= 1.0 {
            interpolator.next_source_frame(source.next());
            *interpolation_value -= 1.0;
        }

        let out = interpolator.interpolate(*interpolation_value);
        *interpolation_value += source_to_target_ratio;
        out
    }

    fn is_exhausted(&self) -> bool {
        self.source.is_exhausted() && self.interpolation_value >= 1.0
    }
}
