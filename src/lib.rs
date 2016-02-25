//! sample
//!
//! A crate for simplifying generic audio sample processing. Use the **Sample** trait to remain
//! generic across any audio bit-depth.

pub use types::{I24, U24, I48, U48};

pub mod buffer;
pub mod conv;
pub mod types;


/// Similar to the std `From` trait, but specifically for converting between sample types.
///
/// We use this trait to be generic over the `Sample::to_sample` and `Sample::from_sample` methods.
pub trait FromSample<S> {
    fn from_sample(s: S) -> Self;
}

impl<S> FromSample<S> for S {
    #[inline]
    fn from_sample(s: S) -> Self {
        s
    }
}

/// Implement the `FromSample` trait for the given types.
macro_rules! impl_from_sample {
    ($T:ty, $fn_name:ident from $({$U:ident: $Umod:ident})*) => {
        $(
            impl FromSample<$U> for $T {
                #[inline]
                fn from_sample(s: $U) -> Self {
                    conv::$Umod::$fn_name(s)
                }
            }
        )*
    };
}

impl_from_sample!{i8, to_i8 from
    {i16:i16} {I24:i24} {i32:i32} {I48:i48} {i64:i64}
    {u8:u8} {u16:u16} {U24:u24} {u32:u32} {U48:u48} {u64:u64}
    {f32:f32} {f64:f64}
}

impl_from_sample!{i16, to_i16 from
    {i8:i8} {I24:i24} {i32:i32} {I48:i48} {i64:i64}
    {u8:u8} {u16:u16} {U24:u24} {u32:u32} {U48:u48} {u64:u64}
    {f32:f32} {f64:f64}
}

impl_from_sample!{I24, to_i24 from
    {i8:i8} {i16:i16} {i32:i32} {I48:i48} {i64:i64}
    {u8:u8} {u16:u16} {U24:u24} {u32:u32} {U48:u48} {u64:u64}
    {f32:f32} {f64:f64}
}

impl_from_sample!{i32, to_i32 from
    {i8:i8} {i16:i16} {I24:i24} {I48:i48} {i64:i64}
    {u8:u8} {u16:u16} {U24:u24} {u32:u32} {U48:u48} {u64:u64}
    {f32:f32} {f64:f64}
}

impl_from_sample!{I48, to_i48 from
    {i8:i8} {i16:i16} {I24:i24} {i32:i32} {i64:i64}
    {u8:u8} {u16:u16} {U24:u24} {u32:u32} {U48:u48} {u64:u64}
    {f32:f32} {f64:f64}
}

impl_from_sample!{i64, to_i64 from
    {i8:i8} {i16:i16} {I24:i24} {i32:i32} {I48:i48}
    {u8:u8} {u16:u16} {U24:u24} {u32:u32} {U48:u48} {u64:u64}
    {f32:f32} {f64:f64}
}

impl_from_sample!{u8, to_u8 from
    {i8:i8} {i16:i16} {I24:i24} {i32:i32} {I48:i48} {i64:i64}
    {u16:u16} {U24:u24} {u32:u32} {U48:u48} {u64:u64}
    {f32:f32} {f64:f64}
}

impl_from_sample!{u16, to_u16 from
    {i8:i8} {i16:i16} {I24:i24} {i32:i32} {I48:i48} {i64:i64}
    {u8:u8} {U24:u24} {u32:u32} {U48:u48} {u64:u64}
    {f32:f32} {f64:f64}
}

impl_from_sample!{U24, to_u24 from
    {i8:i8} {i16:i16} {I24:i24} {i32:i32} {I48:i48} {i64:i64}
    {u8:u8} {u16:u16} {u32:u32} {U48:u48} {u64:u64}
    {f32:f32} {f64:f64}
}

impl_from_sample!{u32, to_u32 from
    {i8:i8} {i16:i16} {I24:i24} {i32:i32} {I48:i48} {i64:i64}
    {u8:u8} {u16:u16} {U24:u24} {U48:u48} {u64:u64}
    {f32:f32} {f64:f64}
}

impl_from_sample!{U48, to_u48 from
    {i8:i8} {i16:i16} {I24:i24} {i32:i32} {I48:i48} {i64:i64}
    {u8:u8} {u16:u16} {U24:u24} {u32:u32} {u64:u64}
    {f32:f32} {f64:f64}
}

impl_from_sample!{u64, to_u64 from
    {i8:i8} {i16:i16} {I24:i24} {i32:i32} {I48:i48} {i64:i64}
    {u8:u8} {u16:u16} {U24:u24} {u32:u32} {U48:u48}
    {f32:f32} {f64:f64}
}

impl_from_sample!{f32, to_f32 from
    {i8:i8} {i16:i16} {I24:i24} {i32:i32} {I48:i48} {i64:i64}
    {u8:u8} {u16:u16} {U24:u24} {u32:u32} {U48:u48} {u64:u64}
    {f64:f64}
}

impl_from_sample!{f64, to_f64 from
    {i8:i8} {i16:i16} {I24:i24} {i32:i32} {I48:i48} {i64:i64}
    {u8:u8} {u16:u16} {U24:u24} {u32:u32} {U48:u48} {u64:u64}
    {f32:f32}
}

/// Similar to the std `Into` trait, but specifically for converting between sample types.
///
/// This trait has a blanket implementation for all types that implement
/// [`FromSample`](./trait.FromSample.html).
pub trait ToSample<S> {
    fn to_sample(self) -> S;
}

impl<T, U> ToSample<U> for T
    where U: FromSample<T>
{
    #[inline]
    fn to_sample(self) -> U {
        U::from_sample(self)
    }
}


/// A trait for working generically across different sample types.
///
/// Provides methods for converting to and from any type that implements the
/// [`FromSample`](./trait.FromSample.html) trait.
pub trait Sample:
    Copy +
    Clone +
    ::std::default::Default +
    ::std::fmt::Debug +
    PartialOrd +
    PartialEq +
    ::std::ops::Add<Output=Self> +
    ::std::ops::Sub<Output=Self> +
    ::std::ops::Mul<Output=Self> +
    ::std::ops::Div<Output=Self> +
    ::std::ops::Rem<Output=Self> +
{
    /// Convert `self` to any type that implements `FromSample`.
    #[inline]
    fn to_sample<S>(self) -> S
        where S: FromSample<Self>,
    {
        FromSample::from_sample(self)
    }

    /// Create a `Self` from any type that implements `ToSample`.
    #[inline]
    fn from_sample<S>(s: S) -> Self
        where Self: FromSample<S>,
    {
        FromSample::from_sample(s)
    }

    /// The equilibrium value for the wave that this `Sample` type represents. This is normally the
    /// value that is equal distance from both the min and max ranges of the sample.
    ///
    /// **NOTE:** This will likely be changed to an "associated const" if the feature lands.
    fn equilibrium() -> Self;
}

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
