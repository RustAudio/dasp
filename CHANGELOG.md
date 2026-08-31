# Unreleased

- Add packed 24-bit sample types `I24LE3`, `I24BE3`, `U24LE3` and `U24BE3` to
  `dasp_sample::types`, corresponding to the ALSA `S24_3LE`, `S24_3BE`,
  `U24_3LE` and `U24_3BE` formats. Unlike `I24`/`U24` these occupy exactly three
  bytes with an alignment of one, so packed 24-bit PCM can be reinterpreted as a
  slice of them without a copy. Out-of-range values wrap modulo 2^24.
- Bump `dasp_sample` and `dasp_frame` to 0.11.1, and raise `dasp_frame`'s
  requirement on `dasp_sample` to `0.11.1`, as it now references the packed
  types by name and would not build against 0.11.0.
- Renamed `window-hanning` to `window-hann`
- Made `IntoInterleavedSamples` and `IntoInterleavedSamplesIterator` stop
  yielding samples when the underlying signal gets exhausted. This is a breaking
  change. The return type of the `IntoInterleavedSamples#next_sample` method was
  modified.

---

# 0.11.0 (2020-05-29)

- Refactor the `sample` crate into a modular collection of crates under `dasp`.
- Rename repository from `sample` to `dasp`, where `dasp` stands for digital
  audio signal processing.
- Add a suite of useful feature gates:
    - Add `std` to all crates. Can be disabled in order to use `no_std`.
    - Add a `all-features-no-std` feature to `dasp`, `dasp_envelope`,
      `dasp_interpolate`, `dasp_signal`, `dasp_slice` and `dasp_window`. Enables
      all features within a `no_std` context.
    - `dasp_envelope` crate:
        - `peak` - enables peak detector implementation.
        - `rms` - enables RMS detector implementation.
    - `dasp_interpolate` crate:
        - `floor` - enables `Floor` `Interpolate` implementation.
        - `linear` - enables `Linear` `Interpolate` implementation.
        - `sinc` - enables `Sinc` `Interpolate` implementation.
    - `dasp_signal` crate:
        - `boxed` - enables `Signal` implementation for `Box<dyn Signal>`.
        - `bus` - enables `SignalBus` trait.
        - `envelope` - enables `SignalEnvelope` trait.
        - `rms` - enables `SignalRms` trait.
        - `window` - enables `signal::window` module.
        - `window-hanning` - enables *hanning* window constructor.
        - `window-rectangle` - enables *rectangle* window constructor.
    - `dasp_slice` crate:
        - `boxed` - enables conversions between boxed slices.
    - The `dasp` crate has a feature for each of the above.
- Make **Window** trait generic over its phase and amplitude type. Update the
  `dasp_signal::window` module items accordingly.
- Remove unsafe uninitialized ring buffer constructors.
- Remove `equilibrium()` and `identity()` constructors from `Sample` and `Frame`
  traitsin favour of `EQUILIBRIUM` and `IDENTITY` associated consts.
- Remove `Frame::n_channels` function in favour of `Frame::CHANNELS` associated
  const.
- Add implementation of `Frame` for all primitive `Sample` types where each are
  assumed to represent a frame of a monophonic signal. This greatly simplifies
  working with monophonic signal sources as demonstrated in the updated
  `dasp_signal` crate.

---

*CHANGELOG begins...*
