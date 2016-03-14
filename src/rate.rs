use {Duplex, Frame, Sample};

/// An iterator that converts the rate at which frames are yielded from some given frame Iterator
/// via some given ratio.
///
/// Other names for `sample::rate::Converter` might include:
///
/// - Sample rate converter
/// - {Up/Down}sampler
/// - Sample interpolater
/// - Sample decimator
///
/// At the moment, `Converter` only supports basic linear amplitude interpolation between
/// frames and is far from the most precise algorithm available. The current form of interpolation
/// also requires that samples are either in `f64` format or can be converted to and from `f64`
/// format. In terms of audio quality, it is not recommended for use in pro-audio applications or
/// professional sound design. However if you are simply reading audio files and need to do a
/// single conversion from their sample rate to your own domain for basic playback, `Converter`
/// might be sufficient and fast at the very least.
///
/// That said, the aim is to provide higher quality interpolation types soon and change
/// `Converter`s interface to a generic API compatible with a range of interpolation types.
#[derive(Clone)]
pub struct Converter<I>
    where I: Iterator,
          I::Item: Frame,
{
    /// The frames at the old rate which we need to convert.
    source_frames: I,
    /// The ratio between the target and source sample rates.
    ///
    /// This value is equal to `source_sample_rate / target_sample_rate`.
    source_to_target_ratio: f64,
    /// The "left" side of the source frame window that is used for interpolation when calculating
    /// new target frames.
    source_window_left: Option<I::Item>,
    /// The "right" side of the source frame window that is used for interpolation when calculating
    /// new target frames.
    source_window_right: Option<I::Item>,
    /// Represents the interpolation between the `source_window_left` and `source_window_right`.
    ///
    /// This can be thought of as the "playhead" over the source frames.
    ///
    /// This is stepped forward by the `source_to_target_ratio` each time a new target sample is
    /// yielded.
    ///
    /// Whenever `source_interpolation` surpasses `1.0`, the "source window" is stepped forward and
    /// the `source_interpolation` reduced by `1.0` accordingly until the "source window" falls
    /// under the `source_interpolation`. The resulting `source_interpolation` is then used to
    /// interpolate the window.
    source_interpolation: f64,
}

impl<I> Converter<I>
    where I: Iterator,
          I::Item: Frame,
{

    /// Construct a new `Converter` from the source frames and the source and target sample rates
    /// (in Hz).
    #[inline]
    pub fn from_hz_to_hz(source_frames: I, source_hz: f64, target_hz: f64) -> Self {
        Self::scale_hz(source_frames, source_hz / target_hz)
    }

    /// Construct a new `Converter` from the source frames and the amount by which the current
    /// playback rate (not sample rate) should be multiplied to reach the new playback rate.
    ///
    /// For example, if our `source_frames` is a sine wave oscillating at a frequency of 2hz and
    /// we wanted to convert it to a frequency of 3hz, the given `scale` should be `1.5`.
    ///
    /// However, if our `source_frames` are being sampled at a rate of 44_100hz and we want to
    /// convert to a sample rate of 96_000hz, the given `scale` should be `44_100.0 / 96_000.0`.
    #[inline]
    pub fn scale_hz(source_frames: I, scale: f64) -> Self {
        assert!(scale > 0.0, "We can't yield any frames at 0 times a second!");
        Converter {
            source_frames: source_frames,
            source_to_target_ratio: scale,
            source_interpolation: 0.0,
            source_window_left: None,
            source_window_right: None,
        }
    }

    /// Update the `source_to_target_ratio` internally given the source and target hz.
    ///
    /// This method might be useful for changing the sample rate during playback.
    #[inline]
    pub fn set_source_and_target_hz(&mut self, source_hz: f64, target_hz: f64) {
        self.set_source_to_target_ratio(source_hz / target_hz)
    }

    /// Update the `source_to_target_ratio` internally given a new rate multiplier.
    ///
    /// This method is useful for dynamically changing rates.
    #[inline]
    pub fn set_rate_multiplier(&mut self, rate_multiplier: f64) {
        self.set_source_to_target_ratio(1.0 / rate_multiplier)
    }

    /// Update the `source_to_target_ratio`.
    ///
    /// For constant rate conversions, this method is unnecessary.
    #[inline]
    pub fn set_source_to_target_ratio(&mut self, source_to_target_ratio: f64) {
        self.source_to_target_ratio = source_to_target_ratio;
    }

    /// Yields the next interpolated target frame.
    #[inline]
    pub fn next_frame(&mut self) -> Option<I::Item>
        where <I::Item as Frame>::Sample: Duplex<f64>,
    {
        let Converter {
            ref mut source_frames,
            source_to_target_ratio,
            ref mut source_interpolation,
            ref mut source_window_left,
            ref mut source_window_right,
        } = *self;

        // Retrieve the source_window_left.
        //
        // If we have no source_window_left, we can assume this is the first iteration and
        // simply assign and yield the first source_frame.
        let mut left_frame = match *source_window_left {
            Some(frame) => frame,
            None => match source_frames.next() {
                Some(frame) => {
                    *source_window_left = Some(frame);
                    *source_interpolation += source_to_target_ratio;
                    return *source_window_left;
                },
                None => return None,
            },
        };

        // Retrieve the source_window_right.
        let mut right_frame = match *source_window_right {
            Some(frame) => frame,
            None => match source_frames.next() {
                Some(frame) => frame,
                None => return None,
            },
        };

        // Step forward in our source_frames until our `source_interpolation` is 0.0...1.0.
        while *source_interpolation > 1.0 {
            left_frame = right_frame;
            right_frame = match source_frames.next() {
                Some(frame) => frame,
                // If there are no more frames we have finished our conversion.
                None => return None,
            };
            *source_interpolation -= 1.0;
        }

        // Calculate the target frame by interpolating between `left_frame` and `right_frame` by
        // the `source_interpolation`.
        let target_frame = left_frame.zip_map(right_frame, |current, next| {
            let current_f = current.to_sample::<f64>();
            let next_f = next.to_sample::<f64>();
            let diff = next_f - current_f;
            let amp = current_f + diff * *source_interpolation;
            amp.to_sample::<<I::Item as Frame>::Sample>()
        });

        *source_window_left = Some(left_frame);
        *source_window_right = Some(right_frame);
        *source_interpolation += source_to_target_ratio;

        Some(target_frame)
    }

}

impl<I> Iterator for Converter<I>
    where I: Iterator,
          I::Item: Frame,
          <I::Item as Frame>::Sample: Duplex<f64>,
{
    type Item = I::Item;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.next_frame()
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len_multiplier = self.source_to_target_ratio / 1.0;
        let (source_lower, source_upper) = self.source_frames.size_hint();
        let lower = (source_lower as f64 * len_multiplier) as usize;
        let upper = source_upper.map(|upper| (upper as f64 * len_multiplier) as usize);
        (lower, upper)
    }
}
