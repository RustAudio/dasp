# Unreleased

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
