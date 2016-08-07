use {Duplex, Frame, Sample};

use core::marker::PhantomData;

/// An iterator that converts the rate at which frames are yielded from some given frame
/// Interpolator into a new type.
///
/// Other names for `sample::rate::Converter` might include:
///
/// - Sample rate converter
/// - {Up/Down}sampler
/// - Sample interpolater
/// - Sample decimator
///
#[derive(Clone)]
pub struct Converter<T: Iterator, I: Interpolator<T>>
    where <T as Iterator>::Item: Frame
{
    interpolator: I,
    interpolation_value: f64,
    source_to_target_ratio: f64,
    phantom: PhantomData<T>
}

/// Interpolator that just rounds off any values to the previous value from the source
pub struct Floor<I>
    where I: Iterator,
          I::Item: Frame
{
    source: I,
    left: I::Item 
}

/// Interpolator that interpolates linearly between the previous value and the next value
pub struct Linear<I>
    where I: Iterator,
          I::Item: Frame
{
    source: I,
    left: I::Item,
    right: Option<I::Item>
}

/// Trait for all things that can interpolate between two values. Implementations should keep track
/// of the necessary data both before and after the current frame.
pub trait Interpolator<I>
    where I: Iterator,
          I::Item: Frame
{
    /// Creates a new Interpolator using the provided Iterator as the source
    fn from_source(source: I) -> Self;

    fn source(&self) -> &I;

    /// Given a distance between [0. and 1.) to the following sample, return the interpolated value
    fn value_at(&self, x: f64) -> I::Item;

    /// Called whenever the Interpolator value is over 1.
    fn increment_frame(&mut self);
}

impl<T, I> Converter<T, I> 
    where T: Iterator,
          <T as Iterator>::Item: Frame,
          I: Interpolator<T>
{
    /// Construct a new `Converter` from the source frames and the source and target sample rates
    /// (in Hz).
    #[inline]
    pub fn from_hz_to_hz(interpolator: I, source_hz: f64, target_hz: f64) -> Self {
        Self::scale_playback_hz(interpolator, source_hz / target_hz)
    }

    /// Construct a new `Converter` from the source frames and the amount by which the current
    /// ***playback*** **rate** (not sample rate) should be multiplied to reach the new playback
    /// rate.
    ///
    /// For example, if our `source_frames` is a sine wave oscillating at a frequency of 2hz and
    /// we wanted to convert it to a frequency of 3hz, the given `scale` should be `1.5`.
    #[inline]
    pub fn scale_playback_hz(interpolator: I, scale: f64) -> Self {
        assert!(scale > 0.0, "We can't yield any frames at 0 times a second!");
        Converter {
            interpolator: interpolator,
            interpolation_value: 0.0,
            source_to_target_ratio: scale,
            phantom: PhantomData
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
    pub fn scale_sample_hz(interpolator: I, scale: f64) -> Self {
        Self::scale_playback_hz(interpolator, 1.0 / scale)
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
    pub fn source(&self) -> &I {
        &self.interpolator
    }

    /// Mutably borrow the `source_frames` Iterator from the `Converter`.
    #[inline]
    pub fn source_mut(&mut self) -> &I {
        &mut self.interpolator
    }

    /// Drop `self` and return the internal `source_frames` Iterator.
    #[inline]
    pub fn into_source(self) -> I {
        self.interpolator
    }
}

impl<T, I> Iterator for Converter<T, I> 
    where T: Iterator,
          <T as Iterator>::Item: Frame,
          <<T as Iterator>::Item as Frame>::Sample: Duplex<f64>,
          I: Interpolator<T>
{
    type Item = <T as Iterator>::Item;

    #[allow(unused_variables)]
    fn next(&mut self) -> Option<<T as Iterator>::Item> {
        let Converter {
            ref mut interpolator,
            ref mut interpolation_value,
            source_to_target_ratio,
            phantom
        } = *self;
 
        while *interpolation_value >= 1.0 {
            interpolator.increment_frame();
            *interpolation_value -= 1.0;
        }

        let out = Some(interpolator.value_at(*interpolation_value));
        *interpolation_value += source_to_target_ratio;

        out
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len_multiplier = self.source_to_target_ratio / 1.0;
        let (source_lower, source_upper) = self.interpolator.source().size_hint();
        let lower = (source_lower as f64 * len_multiplier) as usize;
        let upper = source_upper.map(|upper| (upper as f64 * len_multiplier) as usize);
        (lower, upper)
    }
}

impl<I> Interpolator<I> for Floor<I>
    where I: Iterator,
          <I as Iterator>::Item: Frame,
          <<I as Iterator>::Item as Frame>::Sample: Duplex<f64>
{
    fn from_source(mut source: I) -> Floor<I> {
        let left = source.next().unwrap_or(I::Item::equilibrium());

        Floor {
            source: source,
            left: left
        }
    }

    fn source(&self) -> &I {
        &self.source
    }

    fn value_at(&self, _x: f64) -> I::Item {
        self.left
    }

    fn increment_frame(&mut self) {
        self.left = self.source.next().unwrap_or(self.left);
    }
}

impl<I> Interpolator<I> for Linear<I>
    where I: Iterator,
          <I as Iterator>::Item: Frame,
          <<I as Iterator>::Item as Frame>::Sample: Duplex<f64>
{
    fn from_source(mut source: I) -> Linear<I> {
        let left = source.next().unwrap_or(<<I as Iterator>::Item as Frame>::equilibrium());
        let right = source.next();

        Linear {
            source: source,
            left: left,
            right: right
        }
    }

    fn source(&self) -> &I {
        &self.source
    }

    /// Converts linearly from the previous value, using the next value to interpolate. It is
    /// possible, although not advisable, to provide an x > 1.0 or < 0.0, but this will just
    /// continue to be a linear ramp in one direction or another.
    fn value_at(&self, x: f64) -> I::Item {
        <<I as Iterator>::Item as Frame>::from_fn(|idx| {
            let left = self.left.channel(idx).unwrap_or(&<<I as Iterator>::Item as Frame>::Sample::equilibrium()).to_sample::<f64>();
            let right = self.right.unwrap_or(self.left).channel(idx).unwrap_or(&<<I as Iterator>::Item as Frame>::Sample::equilibrium()).to_sample::<f64>();
            let diff = right - left;
            ((diff * x) + left).to_sample::<<<I as Iterator>::Item as Frame>::Sample>()
        })
    }

    fn increment_frame(&mut self) {
        self.left = self.right.unwrap_or(self.left);
        self.right = self.source.next();
    }
}

