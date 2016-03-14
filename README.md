# sample [![Build Status](https://travis-ci.org/RustAudio/sample.svg?branch=master)](https://travis-ci.org/RustAudio/sample) [![Crates.io](https://img.shields.io/crates/v/sample.svg)](https://crates.io/crates/sample) [![Crates.io](https://img.shields.io/crates/l/sample.svg)](https://github.com/RustAudio/sample/blob/master/LICENSE-MIT)

A crate providing the fundamentals for working with PCM (pulse-code modulation)
DSP (digital signal processing). In other words, `sample` provides a suite of
low-level, high-performance tools including types, traits and functions for
working with digital audio signals.

The `sample` crate requires **no dynamic allocations** and has **no
dependencies**. The goal is to design a library akin to the **std, but for audio
DSP**; keeping the focus on portable and fast fundamentals.

Find the [API documentation here](http://rustaudio.github.io/sample/sample/).


Features
--------

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
assert_eq!(0.0, Sample::equilibrium());
```

Use the **Frame** trait to remain generic over the number of channels at a
discrete moment in time. Implementations are provided for all fixed-size arrays
up to 32 elements in length.

```rust
let foo = [0.1, 0.2, -0.1, -0.2];
let bar = foo.scale_amplitude(2.0);
assert_eq!(bar, [0.2, 0.4, -0.2, -0.4]);

assert_eq!(Mono::<f32>::equilibrium(), [0.0]);
assert_eq!(Stereo::<f32>::equilibrium(), [0.0, 0.0]);
assert_eq!(<[f32; 3]>::equilibrium(), [0.0, 0.0, 0.0]);

let foo = [0i16, 0];
let bar: [u8; 2] = foo.map(Sample::to_sample);
assert_eq!(bar, [128u8, 128]);
```

Use the **Signal** trait for working with `Iterator`s that yield `Frame`s.
To complement the `Iterator` trait, **Signal** provides methods for adding,
scaling, offsetting, multiplying, clipping and generating frame iterators and
more. Working with **Signal**s allows for easy, readable creation of rich and
complex DSP graphs with a simple and familiar API.

```rust
// Clip to an amplitude of 0.9.
let frames = [[1.2, 0.8], [-0.7, -1.4]];
let clipped: Vec<_> = frames.iter().cloned().clip_amp(0.9).collect();
assert_eq!(clipped, vec![[0.9, 0.8], [-0.7, -0.9]]);

// Add `a` with `b` and yield the result.
let a = [[0.2], [-0.6], [0.5]];
let b = [[0.2], [0.1], [-0.8]];
let a_signal = a.iter().cloned();
let b_signal = b.iter().cloned();
let added: Vec<[f32; 1]> = a_signal.add_amp(b_signal).collect();
assert_eq!(added, vec![[0.4], [-0.5], [-0.3]]);

// Scale the playback rate by `0.5`.
let foo = [[0.0], [1.0], [0.0], [-1.0]];
let frames: Vec<_> = foo.iter().cloned().scale_hz(0.5).collect();
assert_eq!(&frames[..], &[[0.0], [0.5], [1.0], [0.5], [0.0], [-0.5], [-1.0]][..]);
```

Use the **slice** module functions for processing chunks of `Frame`s.
Conversion functions are provided for safely converting between slices of
interleaved `Sample`s and slices of `Frame`s without requiring any allocation.
For example:

```rust
let frames = &[[0.0, 0.5], [0.0, -0.5]][..];
let samples = sample::slice::to_sample_slice(frames);
assert_eq!(samples, &[0.0, 0.5, 0.0, -0.5][..]);

let samples = &[0.0, 0.5, 0.0, -0.5][..];
let frames = sample::slice::to_frame_slice(samples);
assert_eq!(frames, Some(&[[0.0, 0.5], [0.0, -0.5]][..]));

let samples = &[0.0, 0.5, 0.0][..];
let frames = sample::slice::to_frame_slice(samples);
assert_eq!(frames, None::<&[[f32; 2]]>);
```

The **conv** module provides pure functions and traits for more specific
conversions. A function is provided for converting between every possible pair
of sample types. Traits include:

- `FromSample`, `ToSample`, `Duplex`,
- `FromSampleSlice`, `ToSampleSlice`, `DuplexSampleSlice`,
- `FromSampleSliceMut`, `ToSampleSliceMut`, `DuplexSampleSliceMut`,
- `FromFrameSlice`, `ToFrameSlice`, `DuplexFrameSlice`,
- `FromFrameSliceMut`, `ToFrameSliceMut`, `DuplexFrameSliceMut`,
- `DuplexSlice`, `DuplexSliceMut`,

The **rate** module provides a **Converter** type, for converting and
interpolating the rate of **Signal**s. This can be useful for both sample rate
conversion and playback rate multiplication.


Contributions
-------------

If the **sample** crate is missing types, conversions or other fundamental
functionality that you wish it had, feel free to open an issue or pull request!
The more hands on deck, the merrier :)


License
-------

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.


**Contributions**

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
