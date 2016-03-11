//! sample
//!
//! A crate for simplifying generic audio sample processing. Use the **Sample** trait to remain
//! generic across any audio bit-depth.

#![recursion_limit="512"]

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

pub mod buffer;
pub mod conv;
pub mod frame;
pub mod signal;
pub mod types;

/// A trait for working generically across different sample types.
///
/// The trait may only be implemented for types that may be converted between any of the 
///
/// Provides methods for converting to and from any type that implements the
/// [`FromSample`](./trait.FromSample.html) trait.
pub trait Sample: Copy
    + Clone
    + std::fmt::Debug
    + PartialOrd
    + PartialEq
{
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
    /// lossless Multiplication format type for summing any two unique `Sample` types together.
    ///
    /// As a user of the `sample` crate, you will never need to be concerned with this type unless
    /// you are defining your own unique `Sample` type(s).
    type Float: FloatSample + Duplex<Self>;

    /// The equilibrium value for the wave that this `Sample` type represents. This is normally the
    /// value that is equal distance from both the min and max ranges of the sample.
    ///
    /// **NOTE:** This will likely be changed to an "associated const" if the feature lands.
    fn equilibrium() -> Self;

    /// Convert `self` to any type that implements `FromSample<Self>`.
    #[inline]
    fn to_sample<S>(self) -> S
        where Self: ToSample<S>,
    {
        self.to_sample_()
    }

    /// Create a `Self` from any type that implements `ToSample<Self>`.
    #[inline]
    fn from_sample<S>(s: S) -> Self
        where Self: FromSample<S>,
    {
        FromSample::from_sample_(s)
    }

    /// Sums the sample `other` from some signal onto the carrier signal `self`.
    #[inline]
    fn add_sample<S>(self, other: S) -> Self
        where S: Sample,
              Self: SelectSigned<S>,
    {
        let self_s: <Self as SelectSigned<S>>::Sample = self.to_sample();
        let other_s: <Self as SelectSigned<S>>::Sample = other.to_sample();
        (self_s + other_s).to_sample::<Self>()
    }

    /// Multiplies the `Sample` by the given amplitude (either `f32` or `f64`).
    ///
    /// - A > 1.0 amplifies the sample.
    /// - A < 1.0 attenuates the sample.
    /// - A == 1.0 yields the same sample.
    /// - A == 0.0 yields the `Sample::equilibrium`.
    #[inline]
    fn scale_amp<F>(self, amp: F) -> Self
        where F: FloatSample,
              Self: SelectFloat<F>,
    {
        let self_f: <Self as SelectFloat<F>>::Sample = self.to_sample();
        let amp_f: <Self as SelectFloat<F>>::Sample = amp.to_sample();
        (self_f * amp_f).to_sample::<Self>()
    }

}

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
                fn equilibrium() -> Self {
                    $equilibrium
                }
            }
        )*
    }
}

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


/// Sample format types whose equilibrium is at 0.
///
/// `Sample`s often need to be converted to some mutual `SignedSample` type for signal addition.
pub trait SignedSample: Sample + std::ops::Add<Output=Self> {}
macro_rules! impl_signed_sample { ($($T:ty)*) => { $( impl SignedSample for $T {} )* } }
impl_signed_sample!(i8 i16 I24 i32 I48 i64 f32 f64);

/// Sample format types represented as floating point numbers.
///
/// `Sample`s often need to be converted to some mutual `FloatSample` type for signal scaling and
/// modulation.
pub trait FloatSample: Sample + std::ops::Mul<Output=Self> {}
impl FloatSample for f32 {}
impl FloatSample for f64 {}


/// Allows for selecting the correct `Sample::Signed` between two sample types.
///
/// In this case, correct means optimal conversion to and from the type losslessly.
pub trait SelectSigned<S>: Sample where S: Sample {
    type Sample: SignedSample + Duplex<Self> + Duplex<S>;
}

/// Allows for selecting the correct `Sample::Float` for conversions between the two `Sample`s
/// `Self` and `S`.
///
/// In this case, correct means optimal conversion to and from the type losslessly.
pub trait SelectFloat<S>: Sample where S: Sample {
    type Sample: FloatSample + Duplex<Self> + Duplex<S>;
}


impl<S> SelectSigned<S> for S where S: Sample {
    type Sample = S::Signed;
}

impl<S> SelectFloat<S> for S where S: Sample {
    type Sample = S::Float;
}


/// Used to simplify implementation of `SelectSigned` and `SelectFloat` for all `Sample` pairs.
macro_rules! impl_select {

    ($Trait:ident: $A:ty ; $B:ty => $Sample:ty) => {
        impl $Trait<$B> for $A {
            type Sample = $Sample;
        }
    };

    // This branch implements $Trait<$B> for $A and $Trait<$A> for $B.
    ($Trait:ident: $A:ty , $B:ty => $Sample:ty, $($rest:tt)*) => {
        impl_select!($Trait: $A ; $B => $Sample);
        impl_select!($Trait: $B ; $A => $Sample);
        impl_select!($Trait: $($rest)*);
    };

    ($Trait:ident: ) => {};
}

// For every possible combination of `SignedSample` types, return the `SelectSigned::Sample`
// between them.
impl_select! { SelectSigned:
    u8  , u16 => i16,
    u8  , U24 => i32,
    u8  , u32 => i32,
    u8  , U48 => i64,
    u8  , u64 => i64,
    u8  , i8  => i8,
    u8  , i16 => i16,
    u8  , I24 => i32,
    u8  , i32 => i32,
    u8  , I48 => i64,
    u8  , i64 => i64,
    u8  , f32 => f32,
    u8  , f64 => f64,

    u16 , U24 => i32,
    u16 , u32 => i32,
    u16 , U48 => i64,
    u16 , u64 => i64,
    u16 , i8  => i16,
    u16 , i16 => i16,
    u16 , I24 => i32,
    u16 , i32 => i32,
    u16 , I48 => i64,
    u16 , i64 => i64,
    u16 , f32 => f32,
    u16 , f64 => f64,

    U24 , u32 => i32,
    U24 , U48 => i64,
    U24 , u64 => i64,
    U24 , i8  => i32,
    U24 , i16 => i32,
    U24 , I24 => i32,
    U24 , i32 => i32,
    U24 , I48 => i64,
    U24 , i64 => i64,
    U24 , f32 => f32,
    U24 , f64 => f64,

    u32 , U48 => i64,
    u32 , u64 => i64,
    u32 , i8  => i32,
    u32 , i16 => i32,
    u32 , I24 => i32,
    u32 , i32 => i32,
    u32 , I48 => i64,
    u32 , i64 => i64,
    u32 , f32 => f32,
    u32 , f64 => f64,

    U48 , u64 => i64,
    U48 , i8  => i64,
    U48 , i16 => i64,
    U48 , I24 => i64,
    U48 , i32 => i64,
    U48 , I48 => i64,
    U48 , i64 => i64,
    U48 , f32 => f64,
    U48 , f64 => f64,

    u64 , i8  => i64,
    u64 , i16 => i64,
    u64 , I24 => i64,
    u64 , i32 => i64,
    u64 , I48 => i64,
    u64 , i64 => i64,
    u64 , f32 => f64,
    u64 , f64 => f64,

    i8  , i16 => i16,
    i8  , I24 => i32,
    i8  , i32 => i32,
    i8  , I48 => i64,
    i8  , i64 => i64,
    i8  , f32 => f32,
    i8  , f64 => f64,

    i16 , I24 => i32,
    i16 , i32 => i32,
    i16 , I48 => i64,
    i16 , i64 => i64,
    i16 , f32 => f32,
    i16 , f64 => f64,

    I24 , i32 => i32,
    I24 , I48 => i64,
    I24 , i64 => i64,
    I24 , f32 => f32,
    I24 , f64 => f64,

    i32 , I48 => i64,
    i32 , i64 => i64,
    i32 , f32 => f32,
    i32 , f64 => f64,

    I48 , i64 => i64,
    I48 , f32 => f64,
    I48 , f64 => f64,

    i64 , f32 => f64,
    i64 , f64 => f64,

    f32 , f64 => f64,
}

// For every possible combination of `FloatSample` types, return the `SelectFloat::Sample` between
// them.
impl_select! { SelectFloat:
    u8  , u16 => f32,
    u8  , U24 => f32,
    u8  , u32 => f32,
    u8  , U48 => f64,
    u8  , u64 => f64,
    u8  , i8  => f32,
    u8  , i16 => f32,
    u8  , I24 => f32,
    u8  , i32 => f32,
    u8  , I48 => f64,
    u8  , i64 => f64,
    u8  , f32 => f32,
    u8  , f64 => f64,

    u16 , U24 => f32,
    u16 , u32 => f32,
    u16 , U48 => f64,
    u16 , u64 => f64,
    u16 , i8  => f32,
    u16 , i16 => f32,
    u16 , I24 => f32,
    u16 , i32 => f32,
    u16 , I48 => f64,
    u16 , i64 => f64,
    u16 , f32 => f32,
    u16 , f64 => f64,

    U24 , u32 => f32,
    U24 , U48 => f64,
    U24 , u64 => f64,
    U24 , i8  => f32,
    U24 , i16 => f32,
    U24 , I24 => f32,
    U24 , i32 => f32,
    U24 , I48 => f64,
    U24 , i64 => f64,
    U24 , f32 => f32,
    U24 , f64 => f64,

    u32 , U48 => f64,
    u32 , u64 => f64,
    u32 , i8  => f32,
    u32 , i16 => f32,
    u32 , I24 => f32,
    u32 , i32 => f32,
    u32 , I48 => f64,
    u32 , i64 => f64,
    u32 , f32 => f32,
    u32 , f64 => f64,

    U48 , u64 => f64,
    U48 , i8  => f64,
    U48 , i16 => f64,
    U48 , I24 => f64,
    U48 , i32 => f64,
    U48 , I48 => f64,
    U48 , i64 => f64,
    U48 , f32 => f64,
    U48 , f64 => f64,

    u64 , i8  => f64,
    u64 , i16 => f64,
    u64 , I24 => f64,
    u64 , i32 => f64,
    u64 , I48 => f64,
    u64 , i64 => f64,
    u64 , f32 => f64,
    u64 , f64 => f64,

    i8  , i16 => f32,
    i8  , I24 => f32,
    i8  , i32 => f32,
    i8  , I48 => f64,
    i8  , i64 => f64,
    i8  , f32 => f32,
    i8  , f64 => f64,

    i16 , I24 => f32,
    i16 , i32 => f32,
    i16 , I48 => f64,
    i16 , i64 => f64,
    i16 , f32 => f32,
    i16 , f64 => f64,

    I24 , i32 => f32,
    I24 , I48 => f64,
    I24 , i64 => f64,
    I24 , f32 => f32,
    I24 , f64 => f64,

    i32 , I48 => f64,
    i32 , i64 => f64,
    i32 , f32 => f32,
    i32 , f64 => f64,

    I48 , i64 => f64,
    I48 , f32 => f64,
    I48 , f64 => f64,

    i64 , f32 => f64,
    i64 , f64 => f64,

    f32 , f64 => f64,
}
