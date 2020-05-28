# Unreleased

*No unreleased changes as of yet.*

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

---

*CHANGELOG begins...*
