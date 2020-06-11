# dasp [![Actions Status][dasp-actions-svg]][dasp-actions] [![docs.rs][dasp-docs-rs-svg]][dasp-docs-rs]

**Digital Audio Signal Processing in Rust.**

*Formerly the [`sample` crate](https://crates.io/crates/sample).*

A suite of crates providing the fundamentals for working with PCM (pulse-code
modulation) DSP (digital signal processing). In other words, `dasp` provides a
suite of low-level, high-performance tools including types, traits and functions
for working with digital audio signals.

The `dasp` libraries require **no dynamic allocations**<sup>1</sup> and have
**no dependencies**. The goal is to design a library akin to the **std, but for
audio DSP**; keeping the focus on portable and fast fundamentals.

<sup>1: Besides the feature-gated `SignalBus` trait, which is occasionally
useful when converting a `Signal` tree into a directed acyclic graph.</sup>

Find the [API documentation here][dasp-docs-rs].


## Crates

**dasp** is a modular collection of crates, allowing users to select the precise
set of tools required for their project. The following crates are included
within this repository:

| **Library** | **Links** | **Description** |
| --- | --- | --- |
| [**`dasp`**][dasp] | [![Crates.io][dasp-crates-io-svg]][dasp-crates-io] [![docs.rs][dasp-docs-rs-svg]][dasp-docs-rs] | Top-level API with features for all crates. |
| [**`dasp_sample`**][dasp_sample] | [![Crates.io][dasp_sample-crates-io-svg]][dasp_sample-crates-io] [![docs.rs][dasp_sample-docs-rs-svg]][dasp_sample-docs-rs] | Sample trait, types, conversions and operations. |
| [**`dasp_frame`**][dasp_frame] | [![Crates.io][dasp_frame-crates-io-svg]][dasp_frame-crates-io] [![docs.rs][dasp_frame-docs-rs-svg]][dasp_frame-docs-rs] | Frame trait, types, conversions and operations. |
| [**`dasp_slice`**][dasp_slice] | [![Crates.io][dasp_slice-crates-io-svg]][dasp_slice-crates-io] [![docs.rs][dasp_slice-docs-rs-svg]][dasp_slice-docs-rs] | Conversions and ops for slices of samples/frames. |
| [**`dasp_ring_buffer`**][dasp_ring_buffer] | [![Crates.io][dasp_ring_buffer-crates-io-svg]][dasp_ring_buffer-crates-io] [![docs.rs][dasp_ring_buffer-docs-rs-svg]][dasp_ring_buffer-docs-rs] | Simple fixed and bounded ring buffers. |
| [**`dasp_peak`**][dasp_peak] | [![Crates.io][dasp_peak-crates-io-svg]][dasp_peak-crates-io] [![docs.rs][dasp_peak-docs-rs-svg]][dasp_peak-docs-rs] | Peak detection with half/full pos/neg wave rectifiers. |
| [**`dasp_rms`**][dasp_rms] | [![Crates.io][dasp_rms-crates-io-svg]][dasp_rms-crates-io] [![docs.rs][dasp_rms-docs-rs-svg]][dasp_rms-docs-rs] | RMS detection with configurable window. |
| [**`dasp_envelope`**][dasp_envelope] | [![Crates.io][dasp_envelope-crates-io-svg]][dasp_envelope-crates-io] [![docs.rs][dasp_envelope-docs-rs-svg]][dasp_envelope-docs-rs] | Envelope detection with peak and RMS impls. |
| [**`dasp_interpolate`**][dasp_interpolate] | [![Crates.io][dasp_interpolate-crates-io-svg]][dasp_interpolate-crates-io] [![docs.rs][dasp_interpolate-docs-rs-svg]][dasp_interpolate-docs-rs] | Inter-frame rate interpolation (linear, sinc, etc). |
| [**`dasp_window`**][dasp_window] | [![Crates.io][dasp_window-crates-io-svg]][dasp_window-crates-io] [![docs.rs][dasp_window-docs-rs-svg]][dasp_window-docs-rs] | Windowing function abstraction (hann, rectangle). |
| [**`dasp_signal`**][dasp_signal] | [![Crates.io][dasp_signal-crates-io-svg]][dasp_signal-crates-io] [![docs.rs][dasp_signal-docs-rs-svg]][dasp_signal-docs-rs] | Iterator-like API for streams of audio frames. |

[![deps-graph][deps-graph]][deps-graph]

*Red dotted lines indicate optional dependencies, while black lines indicate
required dependencies.*


## Features

Use the **Sample** trait to convert between and remain generic over any
bit-depth in an optimal, performance-sensitive manner. Implementations are
provided for all signed integer, unsigned integer and floating point primitive
types along with some custom types including 11, 20, 24 and 48-bit signed and
unsigned unpacked integers. For example:

```rust
assert_eq!((-1.0).to_sample::<u8>(), 0);
assert_eq!(0.0.to_sample::<u8>(), 128);
assert_eq!(0i32.to_sample::<u32>(), 2_147_483_648);
assert_eq!(I24::new(0).unwrap(), Sample::from_sample(0.0));
assert_eq!(0.0, Sample::EQUILIBRIUM);
```

Use the **Frame** trait to remain generic over the number of channels at a
discrete moment in time. Implementations are provided for all fixed-size arrays
up to 32 elements in length.

```rust
let foo = [0.1, 0.2, -0.1, -0.2];
let bar = foo.scale_amp(2.0);
assert_eq!(bar, [0.2, 0.4, -0.2, -0.4]);

assert_eq!(Mono::<f32>::EQUILIBRIUM, [0.0]);
assert_eq!(Stereo::<f32>::EQUILIBRIUM, [0.0, 0.0]);
assert_eq!(<[f32; 3]>::EQUILIBRIUM, [0.0, 0.0, 0.0]);

let foo = [0i16, 0];
let bar: [u8; 2] = foo.map(Sample::to_sample);
assert_eq!(bar, [128u8, 128]);
```

Use the **Signal** trait (enabled by the "signal" feature) for working with
infinite-iterator-like types that yield `Frame`s. **Signal** provides methods
for adding, scaling, offsetting, multiplying, clipping, generating, monitoring
and buffering streams of `Frame`s.  Working with **Signal**s allows for easy,
readable creation of rich and complex DSP graphs with a simple and familiar API.

```rust
// Clip to an amplitude of 0.9.
let frames = [[1.2, 0.8], [-0.7, -1.4]];
let clipped: Vec<_> = signal::from_iter(frames.iter().cloned()).clip_amp(0.9).take(2).collect();
assert_eq!(clipped, vec![[0.9, 0.8], [-0.7, -0.9]]);

// Add `a` with `b` and yield the result.
let a = [0.2, -0.6, 0.5];
let b = [0.2, 0.1, -0.8];
let a_signal = signal::from_iter(a.iter().cloned());
let b_signal = signal::from_iter(b.iter().cloned());
let added: Vec<f32> = a_signal.add_amp(b_signal).take(3).collect();
assert_eq!(added, vec![0.4, -0.5, -0.3]);

// Scale the playback rate by `0.5`.
let foo = [0.0, 1.0, 0.0, -1.0];
let mut source = signal::from_iter(foo.iter().cloned());
let a = source.next();
let b = source.next();
let interp = Linear::new(a, b);
let frames: Vec<_> = source.scale_hz(interp, 0.5).take(8).collect();
assert_eq!(&frames[..], &[0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5][..]);

// Convert a signal to its RMS.
let signal = signal::rate(44_100.0).const_hz(440.0).sine();;
let ring_buffer = ring_buffer::Fixed::from([0.0; WINDOW_SIZE]);
let mut rms_signal = signal.rms(ring_buffer);
```

The **signal** module also provides a series of **Signal** source types,
including:

- `FromIterator`
- `FromInterleavedSamplesIterator`
- `Equilibrium` (silent signal)
- `Phase`
- `Sine`
- `Saw`
- `Square`
- `Noise`
- `NoiseSimplex`
- `Gen` (generate frames from a Fn() -> F)
- `GenMut` (generate frames from a FnMut() -> F)

Use the **slice** module functions (enabled via the "slice" feature) for
processing chunks of `Frame`s.  Conversion functions are provided for safely
converting between slices of interleaved `Sample`s and slices of `Frame`s
without requiring any allocation.  For example:

```rust
let frames = &[[0.0, 0.5], [0.0, -0.5]][..];
let samples = slice::to_sample_slice(frames);
assert_eq!(samples, &[0.0, 0.5, 0.0, -0.5][..]);

let samples = &[0.0, 0.5, 0.0, -0.5][..];
let frames = slice::to_frame_slice(samples);
assert_eq!(frames, Some(&[[0.0, 0.5], [0.0, -0.5]][..]));

let samples = &[0.0, 0.5, 0.0][..];
let frames = slice::to_frame_slice(samples);
assert_eq!(frames, None::<&[[f32; 2]]>);
```

The **signal::interpolate** module provides a **Converter** type, for converting
and interpolating the rate of **Signal**s. This can be useful for both sample
rate conversion and playback rate multiplication. **Converter**s can use a range
of interpolation methods, with Floor, Linear, and Sinc interpolation provided in
the library.

The **ring_buffer** module provides generic **Fixed** and **Bounded** ring
buffer types, both of which may be used with owned, borrowed, stack and
allocated buffers.

The **peak** module can be used for monitoring the peak of a signal. Provided
peak rectifiers include `full_wave`, `positive_half_wave` and
`negative_half_wave`.

The **rms** module provides a flexible **Rms** type that can be used for RMS
(root mean square) detection. Any **Fixed** ring buffer can be used as the
window for the RMS detection.

The **envelope** module provides a **Detector** type (also known as a
*Follower*) that allows for detecting the envelope of a signal. **Detector** is
generic over the type of **Detect**ion - **Rms** and **Peak** detection are
provided. For example:

```rust
let signal = signal::rate(4.0).const_hz(1.0).sine();
let attack = 1.0;
let release = 1.0;
let detector = envelope::Detector::peak(attack, release);
let mut envelope = signal.detect_envelope(detector);
assert_eq!(
    envelope.take(4).collect::<Vec<_>>(),
    vec![0.0, 0.6321205496788025, 0.23254416035257117, 0.7176687675647109]
);
```


## `no_std`

All crates may be compiled with and without the std library. The std library is
enabled by default, however it may be disabled via `--no-default-features`.

To enable all of a crate's features *without* the std library, you may use
`--no-default-features --features "all-no-std"`.

Please note that some of the crates require the `core_intrinsics` feature in
order to be able to perform operations like `sin`, `cos` and `powf32` in a
`no_std` context. This means that these crates require the nightly toolchain in
order to build in a `no_std` context.


## Contributing

If **dasp** is missing types, conversions or other fundamental functionality
that you wish it had, feel free to open an issue or pull request! The more
hands on deck, the merrier :)


## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

**Contributions**

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.


[dasp-actions]: https://github.com/nannou-org/dasp/actions
[dasp-actions-svg]: https://github.com/rustaudio/dasp/workflows/dasp/badge.svg
[deps-graph]: ./assets/deps-graph.png
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
