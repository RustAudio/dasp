# dasp

**Digital Audio Signal Processing in Rust.**

A suite of crates providing the fundamentals for working with PCM (pulse-code
modulation) DSP (digital signal processing). In other words, `dasp` provides a
suite of low-level, high-performance tools including types, traits and functions
for working with digital audio signals.


## Crates

**dasp** is a modular collection of crates, allowing users to select the precise
set of tools required for their project. The following crates are included
within this repository:

| **Library** | **Links** | **Description** |
| --- | --- | --- |
| [**`dasp`**][dasp] | [![Crates.io][dasp-crates-io-svg]][dasp-crates-io] [![docs.rs][dasp-docs-rs-svg]][dasp-docs-rs] | Top-level API with features for all crates. |
| [**`dasp_sample`**][dasp_sample] | [![Crates.io][dasp_sample-crates-io-svg]][dasp_sample-crates-io] [![docs.rs][dasp_sample-docs-rs-svg]][dasp_sample-docs-rs] | Sample trait, types, conversions and operations. |
| [**`dasp_frame`**][dasp_frame] | [![Crates.io][dasp_frame-crates-io-svg]][dasp_frame-crates-io] [![docs.rs][dasp_frame-docs-rs-svg]][dasp_frame-docs-rs] | Frame trait, types, conversions and operations. |
| [**`dasp_slice`**][dasp_slice] | [![Crates.io][dasp_slice-crates-io-svg]][dasp_slice-crates-io] [![docs.rs][dasp_slice-docs-rs-svg]][dasp_slice-docs-rs] | Conversions and operations for slices of samples or frames. |
| [**`dasp_ring_buffer`**][dasp_ring_buffer] | [![Crates.io][dasp_ring_buffer-crates-io-svg]][dasp_ring_buffer-crates-io] [![docs.rs][dasp_ring_buffer-docs-rs-svg]][dasp_ring_buffer-docs-rs] | Simple fixed and bounded ring buffers. |
| [**`dasp_peak`**][dasp_peak] | [![Crates.io][dasp_peak-crates-io-svg]][dasp_peak-crates-io] [![docs.rs][dasp_peak-docs-rs-svg]][dasp_peak-docs-rs] | Peak detection with half/full positive/negative wave rectifiers. |
| [**`dasp_rms`**][dasp_rms] | [![Crates.io][dasp_rms-crates-io-svg]][dasp_rms-crates-io] [![docs.rs][dasp_rms-docs-rs-svg]][dasp_rms-docs-rs] | RMS detection with configurable window. |
| [**`dasp_envelope`**][dasp_envelope] | [![Crates.io][dasp_envelope-crates-io-svg]][dasp_envelope-crates-io] [![docs.rs][dasp_envelope-docs-rs-svg]][dasp_envelope-docs-rs] | Envelope detection abstraction with peak and RMS implementations. |
| [**`dasp_interpolate`**][dasp_interpolate] | [![Crates.io][dasp_interpolate-crates-io-svg]][dasp_interpolate-crates-io] [![docs.rs][dasp_interpolate-docs-rs-svg]][dasp_interpolate-docs-rs] | Abstraction for frame interpolation (provides linear, sinc and more). |
| [**`dasp_window`**][dasp_window] | [![Crates.io][dasp_window-crates-io-svg]][dasp_window-crates-io] [![docs.rs][dasp_window-docs-rs-svg]][dasp_window-docs-rs] | Windowing abstraction with provided hanning and rectangle functions. |
| [**`dasp_signal`**][dasp_signal] | [![Crates.io][dasp_signal-crates-io-svg]][dasp_signal-crates-io] [![docs.rs][dasp_signal-docs-rs-svg]][dasp_signal-docs-rs] | An iterator-like API for working with streams of audio frames. |

[![deps-graph][deps-graph]][deps-graph]

*Red dotted lines indicate optional use, while black lines indicate required
dependencies.*


### Features

TODO


### `no_std`

All crates may be compiled with and without the std library. The std library is
enabled by default, however it may be disabled via `--no-default-features`.

To enable all of a crate's features *without* the std library, you may use
`--no-default-features --features "all-features-no-std"`.


## Contributing

If the **sample** crate is missing types, conversions or other fundamental
functionality that you wish it had, feel free to open an issue or pull request!
The more hands on deck, the merrier :)


## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

**Contributions**

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.


[deps-graph]: https://github.com/RustAudio/sample/blob/master/assets/deps-graph.png

[dasp]: ./dasp
[dasp-crates-io]: https://crates.io/crates/dasp
[dasp-crates-io-svg]: https://img.shields.io/crates/v/dasp.svg
[dasp-docs-rs]: https://docs.rs/dasp/
[dasp-docs-rs-svg]: https://docs.rs/dasp/badge.svg
[dasp_envelope]: ./dasp_envelope
[dasp_envelope-crates-io]: https://crates.io/crates/dasp_envelope
[dasp_envelope-crates-io-svg]: https://img.shields.io/crates/v/dasp_envelope.svg
[dasp_envelope-docs-rs]: https://docs.rs/dasp_envelope/
[dasp_envelope-docs-rs-svg]: https://docs.rs/dasp_envelope/badge.svg
[dasp_frame]: ./dasp_frame
[dasp_frame-crates-io]: https://crates.io/crates/dasp_frame
[dasp_frame-crates-io-svg]: https://img.shields.io/crates/v/dasp_frame.svg
[dasp_frame-docs-rs]: https://docs.rs/dasp_frame/
[dasp_frame-docs-rs-svg]: https://docs.rs/dasp_frame/badge.svg
[dasp_interpolate]: ./dasp_interpolate
[dasp_interpolate-crates-io]: https://crates.io/crates/dasp_interpolate
[dasp_interpolate-crates-io-svg]: https://img.shields.io/crates/v/dasp_interpolate.svg
[dasp_interpolate-docs-rs]: https://docs.rs/dasp_interpolate/
[dasp_interpolate-docs-rs-svg]: https://docs.rs/dasp_interpolate/badge.svg
[dasp_peak]: ./dasp_peak
[dasp_peak-crates-io]: https://crates.io/crates/dasp_peak
[dasp_peak-crates-io-svg]: https://img.shields.io/crates/v/dasp_peak.svg
[dasp_peak-docs-rs]: https://docs.rs/dasp_peak/
[dasp_peak-docs-rs-svg]: https://docs.rs/dasp_peak/badge.svg
[dasp_ring_buffer]: ./dasp_ring_buffer
[dasp_ring_buffer-crates-io]: https://crates.io/crates/dasp_ring_buffer
[dasp_ring_buffer-crates-io-svg]: https://img.shields.io/crates/v/dasp_ring_buffer.svg
[dasp_ring_buffer-docs-rs]: https://docs.rs/dasp_ring_buffer/
[dasp_ring_buffer-docs-rs-svg]: https://docs.rs/dasp_ring_buffer/badge.svg
[dasp_rms]: ./dasp_rms
[dasp_rms-crates-io]: https://crates.io/crates/dasp_rms
[dasp_rms-crates-io-svg]: https://img.shields.io/crates/v/dasp_rms.svg
[dasp_rms-docs-rs]: https://docs.rs/dasp_rms/
[dasp_rms-docs-rs-svg]: https://docs.rs/dasp_rms/badge.svg
[dasp_sample]: ./dasp_sample
[dasp_sample-crates-io]: https://crates.io/crates/dasp_sample
[dasp_sample-crates-io-svg]: https://img.shields.io/crates/v/dasp_sample.svg
[dasp_sample-docs-rs]: https://docs.rs/dasp_sample/
[dasp_sample-docs-rs-svg]: https://docs.rs/dasp_sample/badge.svg
[dasp_signal]: ./dasp_signal
[dasp_signal-crates-io]: https://crates.io/crates/dasp_signal
[dasp_signal-crates-io-svg]: https://img.shields.io/crates/v/dasp_signal.svg
[dasp_signal-docs-rs]: https://docs.rs/dasp_signal/
[dasp_signal-docs-rs-svg]: https://docs.rs/dasp_signal/badge.svg
[dasp_slice]: ./dasp_slice
[dasp_slice-crates-io]: https://crates.io/crates/dasp_slice
[dasp_slice-crates-io-svg]: https://img.shields.io/crates/v/dasp_slice.svg
[dasp_slice-docs-rs]: https://docs.rs/dasp_slice/
[dasp_slice-docs-rs-svg]: https://docs.rs/dasp_slice/badge.svg
[dasp_window]: ./dasp_window
[dasp_window-crates-io]: https://crates.io/crates/dasp_window
[dasp_window-crates-io-svg]: https://img.shields.io/crates/v/dasp_window.svg
[dasp_window-docs-rs]: https://docs.rs/dasp_window/
[dasp_window-docs-rs-svg]: https://docs.rs/dasp_window/badge.svg
