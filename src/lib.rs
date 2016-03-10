//! sample
//!
//! A crate for simplifying generic audio sample processing. Use the **Sample** trait to remain
//! generic across any audio bit-depth.

pub use conv::{FromSample, ToSample, Duplex};
pub use frame::Frame;
pub use types::{I24, U24, I48, U48};

pub mod buffer;
pub mod conv;
pub mod frame;
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
    + std::ops::Add<Output=Self>
    + std::ops::Sub<Output=Self>
    + std::ops::Mul<Output=Self>
    + std::ops::Div<Output=Self>
    + std::ops::Rem<Output=Self>
{

    /// Convert `self` to any type that implements `FromSample`.
    #[inline]
    fn to_sample<S>(self) -> S
        where Self: ToSample<S>,
    {
        self.to_sample_()
    }

    /// Create a `Self` from any type that implements `ToSample`.
    #[inline]
    fn from_sample<S>(s: S) -> Self
        where Self: FromSample<S>,
    {
        FromSample::from_sample_(s)
    }

    /// The equilibrium value for the wave that this `Sample` type represents. This is normally the
    /// value that is equal distance from both the min and max ranges of the sample.
    ///
    /// **NOTE:** This will likely be changed to an "associated const" if the feature lands.
    fn equilibrium() -> Self;

    /// Multiplies the `Sample` by the given amplitude (either `f32` or `f64`).
    ///
    /// - A > 1.0 amplifies the sample.
    /// - A < 1.0 attenuates the sample.
    /// - A == 1.0 yields the same sample.
    /// - A == 0.0 yields the `Sample::equilibrium`.
    #[inline]
    fn scale_amplitude<A>(self, amplitude: A) -> Self
        where Self: Duplex<A>,
              A: Amplitude,
    {
        (self.to_sample::<A>() * amplitude).to_sample()
    }

}

/// This trait exists in order for us to make `Amplitude` public while restricting it from being
/// implemented for other types by providing a blanket implementation.
trait PrivateAmplitude: Sample {}
impl PrivateAmplitude for f32 {}
impl PrivateAmplitude for f64 {}

/// Types that may be used as an amplitude multiplier for a signal.
///
/// This trait is only implemented for `f32` and `f64` in order to avoid strange behaviour with
/// user-defined `Amplitude`s.
pub trait Amplitude: Sample {}
impl<T> Amplitude for T where T: PrivateAmplitude {}


macro_rules! impl_sample {
    ($($T:ty: $equilibrium:expr),*) => {
        $(
            impl Sample for $T {
                fn equilibrium() -> Self {
                    $equilibrium
                }
            }
        )*
    }
}

impl_sample!{
    i8: 0,
    i16: 0,
    I24: types::i24::EQUILIBRIUM,
    i32: 0,
    I48: types::i48::EQUILIBRIUM,
    i64: 0,
    u8: 128,
    u16: 32_768,
    U24: types::u24::EQUILIBRIUM,
    u32: 2_147_483_648,
    U48: types::u48::EQUILIBRIUM,
    u64: 9_223_372_036_854_775_808,
    f32: 0.0,
    f64: 0.0
}
