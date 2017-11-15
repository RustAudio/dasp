//! The Interpolate module allows for conversion between various sample rates.

use {Duplex, Frame, Sample, Signal};
use core::f64::consts::PI;
use ring_buffer;
use ops::f64::{sin, cos};

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

/// Interpolator that just rounds off any values to the previous value from the source
pub struct Floor<F> {
    left: F,
}

/// Interpolator that interpolates linearly between the previous value and the next value
pub struct Linear<F> {
    left: F,
    right: F,
}

/// Interpolator for sinc interpolation.
///
/// Generally accepted as one of the better sample rate converters, although it uses significantly
/// more computation.
pub struct Sinc<S> {
    frames: ring_buffer::Fixed<S>,
    idx: usize,
}

/// Types that can interpolate between two values.
///
/// Implementations should keep track of the necessary data both before and after the current
/// frame.
pub trait Interpolator {
    type Frame: Frame;

    /// Given a distance between [0.0 and 1.0) to the following sample, return the interpolated
    /// value.
    fn interpolate(&self, x: f64) -> Self::Frame;

    /// Called whenever the Interpolator value steps passed 1.0.
    fn next_source_frame(&mut self, source_frame: Self::Frame);
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

impl<F> Floor<F> {
    /// Create a new Floor Interpolator.
    pub fn new(left: F) -> Floor<F> {
        Floor { left: left }
    }

    /// Consumes the first value from a given source in order to initialize itself. If the source
    /// has no values at all, this will return None.
    pub fn from_source<S>(source: &mut S) -> Floor<F>
    where
        F: Frame,
        S: Signal<Frame = F>,
    {
        let left = source.next();
        Floor { left: left }
    }
}

impl<F> Linear<F> {
    /// Create a new Linear Interpolator.
    pub fn new(left: F, right: F) -> Linear<F> {
        Linear {
            left: left,
            right: right,
        }
    }

    /// Consumes the first value from a given source to initialize itself. If the source has no
    /// values, this will return None.
    pub fn from_source<S>(source: &mut S) -> Linear<F>
    where
        F: Frame,
        S: Signal<Frame = F>,
    {
        let left = source.next();
        let right = source.next();
        Linear {
            left: left,
            right: right,
        }
    }
}

impl<S> Sinc<S> {
    /// Create a new **Sinc** interpolator with the given ring buffer.
    ///
    /// The given ring buffer should have a length twice that of the desired sinc interpolation
    /// `depth`.
    ///
    /// The initial contents of the ring_buffer will act as padding for the interpolated signal.
    ///
    /// **panic!**s if the given ring buffer's length is not a multiple of `2`.
    pub fn new(frames: ring_buffer::Fixed<S>) -> Self
    where
        S: ring_buffer::SliceMut,
        S::Element: Frame,
    {
        assert!(frames.len() % 2 == 0);
        Sinc {
            frames: frames,
            idx: 0,
        }
    }

    fn depth(&self) -> usize
    where
        S: ring_buffer::Slice,
    {
        self.frames.len() / 2
    }
}

impl<F> Interpolator for Floor<F>
where
    F: Frame,
    F::Sample: Duplex<f64>,
{
    type Frame = F;

    fn interpolate(&self, _x: f64) -> Self::Frame {
        self.left
    }

    fn next_source_frame(&mut self, source_frame: Self::Frame) {
        self.left = source_frame;
    }
}

impl<F> Interpolator for Linear<F>
where
    F: Frame,
    F::Sample: Duplex<f64>,
{
    type Frame = F;

    /// Converts linearly from the previous value, using the next value to interpolate. It is
    /// possible, although not advisable, to provide an x > 1.0 or < 0.0, but this will just
    /// continue to be a linear ramp in one direction or another.
    fn interpolate(&self, x: f64) -> Self::Frame {
        self.left.zip_map(self.right, |l, r| {
            let l_f = l.to_sample::<f64>();
            let r_f = r.to_sample::<f64>();
            let diff = r_f - l_f;
            ((diff * x) + l_f).to_sample::<<Self::Frame as Frame>::Sample>()
        })
    }

    fn next_source_frame(&mut self, source_frame: Self::Frame) {
        self.left = self.right;
        self.right = source_frame;
    }
}

impl<S> Interpolator for Sinc<S>
where
    S: ring_buffer::SliceMut,
    S::Element: Frame,
    <S::Element as Frame>::Sample: Duplex<f64>,
{
    type Frame = S::Element;

    /// Sinc interpolation
    fn interpolate(&self, x: f64) -> Self::Frame {
        let phil = x;
        let phir = 1.0 - x;
        let nl = self.idx;
        let nr = self.idx + 1;
        let depth = self.depth();

        let rightmost = nl + depth;
        let leftmost = nr as isize - depth as isize;
        let max_depth = if rightmost >= self.frames.len() {
            self.frames.len() - depth
        } else if leftmost < 0 {
            (depth as isize + leftmost) as usize
        } else {
            depth
        };

        (0..max_depth).fold(Self::Frame::equilibrium(), |mut v, n| {
            v = {
                let a = PI * (phil + n as f64);
                let first = sin(a) / a;
                let second = 0.5 + 0.5 * cos(a / (phil + max_depth as f64));
                v.zip_map(self.frames[nr - n], |vs, r_lag| {
                    vs.add_amp(
                        (first * second * r_lag.to_sample::<f64>())
                            .to_sample::<<Self::Frame as Frame>::Sample>()
                            .to_signed_sample(),
                    )
                })
            };

            let a = PI * (phir + n as f64);
            let first = sin(a) / a;
            let second = 0.5 + 0.5 * cos(a / (phir + max_depth as f64));
            v.zip_map(self.frames[nl + n], |vs, r_lag| {
                vs.add_amp(
                    (first * second * r_lag.to_sample::<f64>())
                        .to_sample::<<Self::Frame as Frame>::Sample>()
                        .to_signed_sample(),
                )
            })
        })
    }

    fn next_source_frame(&mut self, source_frame: Self::Frame) {
        let _old_frame = self.frames.push(source_frame);
        if self.idx < self.depth() {
            self.idx += 1;
        }
    }
}
