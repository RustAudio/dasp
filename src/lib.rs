//! A crate of fundamentals for audio PCM DSP.
//!
//! - Use the [**Sample** trait](./trait.Sample.html) to remain generic across bit-depth.
//! - Use the [**Frame** trait](./frame/trait.Frame.html) to remain generic over channel layout.
//! - Use the [**Signal** trait](./signal/trait.Signal.html) for working with **Iterators** that yield **Frames**.
//! - Use the [**slice** module](./slice/index.html) for working with slices of **Samples** and **Frames**.
//! - See the [**conv** module](./conv/index.html) for fast conversions between slices, frames and samples.
//! - See the [**types** module](./types/index.html) for provided custom sample types.
//! - See the [**rate** module](./rate/index.html) for sample rate conversion and scaling.

#![recursion_limit="512"]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc, collections, core_intrinsics))]

#[cfg(feature = "std")]
extern crate core;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate collections;

#[cfg(not(feature = "std"))]
type Vec<T> = collections::vec::Vec<T>;
#[cfg(feature = "std")]
type Vec<T> = std::vec::Vec<T>;

#[cfg(not(feature = "std"))]
type VecDeque<T> = collections::vec_deque::VecDeque<T>;
#[cfg(feature = "std")]
type VecDeque<T> = std::collections::vec_deque::VecDeque<T>;

#[cfg(not(feature = "std"))]
type Rc<T> = alloc::rc::Rc<T>;
#[cfg(feature = "std")]
type Rc<T> = std::rc::Rc<T>;

pub use conv::{
    FromSample, ToSample, Duplex,
    FromSampleSlice, ToSampleSlice, DuplexSampleSlice,
    FromSampleSliceMut, ToSampleSliceMut, DuplexSampleSliceMut,
    FromFrameSlice, ToFrameSlice, DuplexFrameSlice,
    FromFrameSliceMut, ToFrameSliceMut, DuplexFrameSliceMut,
    DuplexSlice, DuplexSliceMut,
};
pub use frame::Frame;
pub use signal::Signal;
pub use types::{I24, U24, I48, U48};

pub mod slice;
pub mod conv;
pub mod frame;
pub mod signal;
pub mod rate;
pub mod types;

#[cfg(not(feature = "std"))]
fn floor(x: f64) -> f64 {
    unsafe { core::intrinsics::floorf64(x) }
}
#[cfg(feature = "std")]
fn floor(x: f64) -> f64 {
    x.floor()
}

#[cfg(not(feature = "std"))]
fn sin(x: f64) -> f64 {
    unsafe { core::intrinsics::sinf64(x) }
}
#[cfg(feature = "std")]
fn sin(x: f64) -> f64 {
    x.sin()
}

/// A trait for working generically across different **Sample** format types.
///
/// Provides methods for converting to and from any type that implements the
/// [`FromSample`](./trait.FromSample.html) trait and provides methods for performing signal
/// amplitude addition and multiplication.
///
/// # Example
///
/// ```rust
/// extern crate sample;
///
/// use sample::{I24, Sample};
///
/// fn main() {
///     assert_eq!((-1.0).to_sample::<u8>(), 0);
///     assert_eq!(0.0.to_sample::<u8>(), 128);
///     assert_eq!(0i32.to_sample::<u32>(), 2_147_483_648);
///     assert_eq!(I24::new(0).unwrap(), Sample::from_sample(0.0));
///     assert_eq!(0.0, Sample::equilibrium());
/// }
/// ```
pub trait Sample: Copy + Clone + PartialOrd + PartialEq {

    /// When summing two samples of a signal together, it is necessary for both samples to be
    /// represented in some signed format. This associated `Addition` type represents the format to
    /// which `Self` should be converted for optimal `Addition` performance.
    ///
    /// For example, u32's optimal `Addition` type would be i32, u8's would be i8, f32's would be
    /// f32, etc.
    ///
    /// Specifying this as an associated type allows us to automatically determine the optimal,
    /// lossless Addition format type for summing any two unique `Sample` types together.
    ///
    /// As a user of the `sample` crate, you will never need to be concerned with this type unless
    /// you are defining your own unique `Sample` type(s).
    type Signed: SignedSample + Duplex<Self>;

    /// When multiplying two samples of a signal together, it is necessary for both samples to be
    /// represented in some signed, floating-point format. This associated `Multiplication` type
    /// represents the format to which `Self` should be converted for optimal `Multiplication`
    /// performance.
    ///
    /// For example, u32's optimal `Multiplication` type would be f32, u64's would be f64, i8's
    /// would be f32, etc.
    ///
    /// Specifying this as an associated type allows us to automatically determine the optimal,
    /// lossless Multiplication format type for multiplying any two unique `Sample` types together.
    ///
    /// As a user of the `sample` crate, you will never need to be concerned with this type unless
    /// you are defining your own unique `Sample` type(s).
    type Float: FloatSample + Duplex<Self>;

    /// The equilibrium value for the wave that this `Sample` type represents. This is normally the
    /// value that is equal distance from both the min and max ranges of the sample.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Sample;
    ///
    /// fn main() {
    ///     assert_eq!(0.0, f32::equilibrium());
    ///     assert_eq!(0, i32::equilibrium());
    ///     assert_eq!(128, u8::equilibrium());
    ///     assert_eq!(32_768_u16, Sample::equilibrium());
    /// }
    /// ```
    ///
    /// **Note:** This will likely be changed to an "associated const" if the feature lands.
    fn equilibrium() -> Self;

    /// The multiplicative identity of the signal.
    ///
    /// In other words: A value which when used to scale/multiply the amplitude or frequency of a
    /// signal, returns the same signal.
    ///
    /// This is useful as a default, non-affecting amplitude or frequency multiplier.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::{Sample, U48};
    ///
    /// fn main() {
    ///     assert_eq!(1.0, f32::identity());
    ///     assert_eq!(1.0, i8::identity());
    ///     assert_eq!(1.0, u8::identity());
    ///     assert_eq!(1.0, U48::identity());
    /// }
    /// ```
    #[inline]
    fn identity() -> Self::Float {
        <Self::Float as FloatSample>::identity()
    }

    /// Convert `self` to any type that implements `FromSample<Self>`.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Sample;
    ///
    /// fn main() {
    ///     assert_eq!(0.0.to_sample::<i32>(), 0);
    ///     assert_eq!(0.0.to_sample::<u8>(), 128);
    ///     assert_eq!((-1.0).to_sample::<u8>(), 0);
    /// }
    /// ```
    #[inline]
    fn to_sample<S>(self) -> S
        where Self: ToSample<S>,
    {
        self.to_sample_()
    }

    /// Create a `Self` from any type that implements `ToSample<Self>`.
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::{Sample, I24};
    ///
    /// fn main() {
    ///     assert_eq!(f32::from_sample(128_u8), 0.0);
    ///     assert_eq!(i8::from_sample(-1.0), -128);
    ///     assert_eq!(I24::from_sample(0.0), I24::new(0).unwrap());
    /// }
    /// ```
    #[inline]
    fn from_sample<S>(s: S) -> Self
        where Self: FromSample<S>,
    {
        FromSample::from_sample_(s)
    }

    /// Adds (or "offsets") the amplitude of the `Sample` by the given signed amplitude.
    ///
    /// `Self` will be converted to `Self::Signed`, the addition will occur and then the result
    /// will be converted back to `Self`. These conversions allow us to correctly handle the
    /// addition of unsigned signal formats.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Sample;
    ///
    /// fn main() {
    ///     assert_eq!(0.25.add_amp(0.5), 0.75);
    ///     assert_eq!(192u8.add_amp(-128), 64);
    /// }
    /// ```
    #[inline]
    fn add_amp(self, amp: Self::Signed) -> Self {
        let self_s: Self::Signed = self.to_sample();
        (self_s + amp).to_sample()
    }

    /// Multiplies (or "scales") the amplitude of the `Sample` by the given float amplitude.
    ///
    /// - `amp` > 1.0 amplifies the sample.
    /// - `amp` < 1.0 attenuates the sample.
    /// - `amp` == 1.0 yields the same sample.
    /// - `amp` == 0.0 yields the `Sample::equilibrium`.
    ///
    /// `Self` will be converted to `Self::Float`, the multiplication will occur and then the
    /// result will be converted back to `Self`. These conversions allow us to correctly handle the
    /// multiplication of integral signal formats.
    ///
    /// # Example
    ///
    /// ```rust
    /// extern crate sample;
    ///
    /// use sample::Sample;
    ///
    /// fn main() {
    ///     assert_eq!(64_i8.mul_amp(0.5), 32);
    ///     assert_eq!(0.5.mul_amp(-2.0), -1.0);
    ///     assert_eq!(64_u8.mul_amp(0.0), 128);
    /// }
    /// ```
    #[inline]
    fn mul_amp(self, amp: Self::Float) -> Self {
        let self_f: Self::Float = self.to_sample();
        (self_f * amp).to_sample()
    }

}

/// A macro used to simplify the implementation of `Sample`.
macro_rules! impl_sample {
    ($($T:ty:
       Signed: $Addition:ty,
       Float: $Modulation:ty,
       equilibrium: $equilibrium:expr),*) =>
    {
        $(
            impl Sample for $T {
                type Signed = $Addition;
                type Float = $Modulation;
                #[inline]
                fn equilibrium() -> Self {
                    $equilibrium
                }
            }
        )*
    }
}

// Expands to `Sample` implementations for all of the following types.
impl_sample!{
    i8:  Signed: i8,  Float: f32, equilibrium: 0,
    i16: Signed: i16, Float: f32, equilibrium: 0,
    I24: Signed: I24, Float: f32, equilibrium: types::i24::EQUILIBRIUM,
    i32: Signed: i32, Float: f32, equilibrium: 0,
    I48: Signed: I48, Float: f64, equilibrium: types::i48::EQUILIBRIUM,
    i64: Signed: i64, Float: f64, equilibrium: 0,
    u8:  Signed: i8,  Float: f32, equilibrium: 128,
    u16: Signed: i16, Float: f32, equilibrium: 32_768,
    U24: Signed: i32, Float: f32, equilibrium: types::u24::EQUILIBRIUM,
    u32: Signed: i32, Float: f32, equilibrium: 2_147_483_648,
    U48: Signed: i64, Float: f64, equilibrium: types::u48::EQUILIBRIUM,
    u64: Signed: i64, Float: f64, equilibrium: 9_223_372_036_854_775_808,
    f32: Signed: f32, Float: f32, equilibrium: 0.0,
    f64: Signed: f64, Float: f64, equilibrium: 0.0
}


/// Integral and floating-point **Sample** format types whose equilibrium is at 0.
///
/// **Sample**s often need to be converted to some mutual **SignedSample** type for signal
/// addition.
pub trait SignedSample: Sample
    + core::ops::Add<Output=Self>
    + core::ops::Sub<Output=Self>
    + core::ops::Neg<Output=Self> {}
macro_rules! impl_signed_sample { ($($T:ty)*) => { $( impl SignedSample for $T {} )* } }
impl_signed_sample!(i8 i16 I24 i32 I48 i64 f32 f64);

/// Sample format types represented as floating point numbers.
///
/// **Sample**s often need to be converted to some mutual **FloatSample** type for signal scaling
/// and modulation.
pub trait FloatSample: SignedSample
    + core::ops::Mul<Output=Self>
    + core::ops::Div<Output=Self>
{
    /// Represents the multiplicative identity of the floating point signal.
    fn identity() -> Self;
}

impl FloatSample for f32 {
    #[inline]
    fn identity() -> Self { 1.0 }
}

impl FloatSample for f64 {
    #[inline]
    fn identity() -> Self { 1.0 }
}
