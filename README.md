# sample [![Build Status](https://travis-ci.org/RustAudio/sample.svg?branch=master)](https://travis-ci.org/RustAudio/sample) [![Crates.io](https://img.shields.io/crates/v/sample.svg)](https://crates.io/crates/sample) [![Crates.io](https://img.shields.io/crates/l/sample.svg)](https://github.com/RustAudio/sample/blob/master/LICENSE-MIT)

A crate providing the fundamentals for working with PCM (pulse-code modulation)
DSP (digital signal processing). In other words, `sample` provides a suite of
low-level, high-performance tools including types, traits and functions for
working with digital audio signals.


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

Use the **buffer** module functions for processing chunks of `Frame`s.
Conversion functions are provided for safely converting between slices of
interleaved `Sample`s and slices of `Frame`s without requiring any allocation.
For example:

```rust
let foo = &[[0.0, 0.5], [0.0, -0.5]][..];
let bar = sample::buffer::to_sample_slice(foo);
assert_eq!(bar, &[0.0, 0.5, 0.0, -0.5][..]);

let foo = &[0.0, 0.5, 0.0, -0.5][..];
let bar = sample::buffer::to_frame_slice(foo);
assert_eq!(bar, Some(&[[0.0, 0.5], [0.0, -0.5]][..]));

let foo = &[0.0, 0.5, 0.0][..];
let bar = sample::buffer::to_frame_slice(foo);
assert_eq!(bar, None::<&[[f32; 2]]>);
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


Coming Soon
-----------

The **rate** module for handling sample rate conversion.

The **Signal** trait for working with `Frame` `Iterator`s.

If `sample` is missing types, conversions or other functionality that you wish
it had, feel free to open an issue or pull request!


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
