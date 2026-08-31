//! A collection of custom, non-std **Sample** types.
//!
//! Most types here are "padded", i.e. they store their value in the smallest primitive integer
//! that can contain it. `I24`, for example, is a newtype around an `i32` and is therefore four
//! bytes wide with an alignment of four.
//!
//! The `*LE3` and `*BE3` types are the "packed" counterparts: a 24-bit value stored in exactly
//! three bytes with an alignment of one, in the given byte order regardless of the byte order of
//! the host. These correspond to the `S24_3LE`, `S24_3BE`, `U24_3LE` and `U24_3BE` formats as
//! named by ALSA.
//!
//! # Packed types
//!
//! All four are `#[repr(transparent)]` over `[u8; 3]`, with `size_of` exactly 3, `align_of`
//! exactly 1, and every one of the 2^24 byte patterns a valid value. That is what they exist
//! for: a buffer of packed 24-bit PCM can be reinterpreted as a slice of them without a copy,
//! which a four-byte container cannot do. rustdoc does not render `repr(transparent)` for a type
//! with a private field, so the guarantee is stated here rather than being visible on each type.
//!
//! ```
//! use dasp_sample::I24LE3;
//!
//! let raw: [u8; 9] = [0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x7F];
//! // Sound because the alignment is 1, every byte pattern is valid, and the length is divided
//! // by the stride rather than assumed.
//! let samples: &[I24LE3] = unsafe {
//!     core::slice::from_raw_parts(raw.as_ptr() as *const I24LE3, raw.len() / 3)
//! };
//! assert_eq!(samples[0].inner(), -8_388_608);
//! assert_eq!(samples[2].inner(), 8_388_607);
//! ```
//!
//! Arithmetic mirrors the padded types — `Add`, `Sub`, `Mul`, `Div`, `Rem`, and `Neg` on the
//! signed pair, with the same debug-panic/release-wrap contract — so `I24LE3` and `I24BE3` are
//! `SignedSample` and `U24LE3` and `U24BE3` are not. The bitwise and shift operators are not
//! mirrored: `!x` on a padded type inverts the byte holding no sample data.
//!
//! Two behaviours differ from the padded types and are easy to trip over:
//!
//! - Out-of-range values wrap modulo 2^24, because there is no spare byte to hold them. See the
//!   `conv` module docs.
//! - `Default` is all-zero bytes, matching `I24`/`U24`. For the signed types that is equilibrium,
//!   but for the unsigned ones it is `MIN`, the negative rail. Fill silence with `EQUILIBRIUM`.

pub use self::i11::I11;
pub use self::i20::I20;
pub use self::i24::I24;
pub use self::i24_be3::I24BE3;
pub use self::i24_le3::I24LE3;
pub use self::i48::I48;
pub use self::u11::U11;
pub use self::u20::U20;
pub use self::u24::U24;
pub use self::u24_be3::U24BE3;
pub use self::u24_le3::U24LE3;
pub use self::u48::U48;

macro_rules! impl_from {
    ($T:ident: $Rep:ident from {$U:ident : $URep:ty}) => {
        impl From<$U> for $T {
            #[inline]
            fn from(other: $U) -> Self {
                $T(other.inner() as $Rep)
            }
        }
    };
    ($T:ident: $Rep:ident from $U:ident) => {
        impl From<$U> for $T {
            #[inline]
            fn from(other: $U) -> Self {
                $T(other as $Rep)
            }
        }
    };
}

macro_rules! impl_froms {
    ($T:ident: $Rep:ident, {$U:ident : $URep:ty}, $($rest:tt)*) => {
        impl_from!($T: $Rep from {$U: $URep});
        impl_froms!($T: $Rep, $($rest)*);
    };
    ($T:ident: $Rep:ident, {$U:ident : $URep:ty}) => {
        impl_from!($T: $Rep from {$U: $URep});
    };
    ($T:ident: $Rep:ident, $U:ident, $($rest:tt)*) => {
        impl_from!($T: $Rep from $U);
        impl_froms!($T: $Rep, $($rest)*);
    };
    ($T:ident: $Rep:ident, $U:ident) => {
        impl_from!($T: $Rep from $U);
    };
    ($T:ident: $Rep:ident,) => {};
}

macro_rules! impl_neg {
    ($T:ident) => {
        impl ::core::ops::Neg for $T {
            type Output = $T;
            #[inline]
            fn neg(self) -> $T {
                $T(-self.0)
            }
        }
    };
}

// The packed counterparts of `impl_from!`/`impl_froms!`/`impl_neg!`. Separate because the value
// has to be encoded into three bytes rather than stored directly.
macro_rules! impl_packed_from {
    ($T:ident from {$U:ident}) => {
        impl From<$U> for $T {
            #[inline]
            fn from(other: $U) -> Self {
                $T::new_unchecked(other.inner() as i32)
            }
        }
    };
    ($T:ident from $U:ident) => {
        impl From<$U> for $T {
            #[inline]
            fn from(other: $U) -> Self {
                $T::new_unchecked(other as i32)
            }
        }
    };
}

macro_rules! impl_packed_froms {
    ($T:ident, {$U:ident}, $($rest:tt)*) => {
        impl_packed_from!($T from {$U});
        impl_packed_froms!($T, $($rest)*);
    };
    ($T:ident, {$U:ident}) => {
        impl_packed_from!($T from {$U});
    };
    ($T:ident, $U:ident, $($rest:tt)*) => {
        impl_packed_from!($T from $U);
        impl_packed_froms!($T, $($rest)*);
    };
    ($T:ident, $U:ident) => {
        impl_packed_from!($T from $U);
    };
    ($T:ident,) => {};
}

macro_rules! impl_packed_neg {
    ($T:ident) => {
        impl ::core::ops::Neg for $T {
            type Output = $T;
            /// Negation wraps for `MIN`, which has no positive counterpart in 24 bits, in
            /// keeping with the rest of these types' out-of-range behaviour.
            #[inline]
            fn neg(self) -> $T {
                $T::new_unchecked(-self.inner())
            }
        }
    };
}

// The arithmetic operators, mirroring what `new_sample_type!` gives the padded types: `Add`,
// `Sub` and `Mul` panic on overflow in debug and wrap in release, while `Div` and `Rem` are
// unchecked, exactly as the padded ones are. The release arms need no explicit wrap, because
// truncating to three bytes *is* reduction modulo 2^24 -- it lands on the same value the padded
// types reach through `wrap_overflow_once` (for `Add`/`Sub`) or `wrap_overflow` (for `Mul`).
//
// The bitwise and shift operators are deliberately not mirrored. They are bit manipulation
// rather than arithmetic, and their meaning does not carry over: `!x` on a padded type inverts
// the 32 bits of its container, including the byte that holds no sample data.
macro_rules! impl_packed_arithmetic {
    ($T:ident) => {
        impl ::core::ops::Add<$T> for $T {
            type Output = $T;
            #[inline]
            fn add(self, other: Self) -> Self {
                let sum = self.inner() + other.inner();
                if cfg!(debug_assertions) {
                    $T::new(sum).expect("arithmetic operation overflowed")
                } else {
                    $T::new_unchecked(sum)
                }
            }
        }

        impl ::core::ops::Sub<$T> for $T {
            type Output = $T;
            #[inline]
            fn sub(self, other: Self) -> Self {
                let difference = self.inner() - other.inner();
                if cfg!(debug_assertions) {
                    $T::new(difference).expect("arithmetic operation overflowed")
                } else {
                    $T::new_unchecked(difference)
                }
            }
        }

        impl ::core::ops::Mul<$T> for $T {
            type Output = $T;
            #[inline]
            fn mul(self, other: Self) -> Self {
                // As on the padded types, the product is formed in the same width the values
                // unpack to, so a large enough pair overflows that before the sample range is
                // ever considered.
                let product = self.inner() * other.inner();
                if cfg!(debug_assertions) {
                    $T::new(product).expect("arithmetic operation overflowed")
                } else {
                    $T::new_unchecked(product)
                }
            }
        }

        impl ::core::ops::Div<$T> for $T {
            type Output = $T;
            #[inline]
            fn div(self, other: Self) -> Self {
                $T::new_unchecked(self.inner() / other.inner())
            }
        }

        impl ::core::ops::Rem<$T> for $T {
            type Output = $T;
            #[inline]
            fn rem(self, other: Self) -> Self {
                $T::new_unchecked(self.inner() % other.inner())
            }
        }
    };
}

macro_rules! new_sample_type {
    ($T:ident: $Rep:ident, eq: $EQ:expr, min: $MIN:expr, max: $MAX:expr, total: $TOTAL:expr, from: $($rest:tt)*) => {
        pub const MIN: $T = $T($MIN);
        pub const MAX: $T = $T($MAX);
        pub const EQUILIBRIUM: $T = $T($EQ);
        const MIN_REP: $Rep = $MIN;
        const MAX_REP: $Rep = $MAX;
        const TOTAL: $Rep = $TOTAL;

        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
        pub struct $T($Rep);

        impl From<$Rep> for $T {
            #[inline]
            fn from(val: $Rep) -> Self {
                $T(val).wrap_overflow()
            }
        }

        impl $T {
            /// Construct a new sample if the given value is within range.
            ///
            /// Returns `None` if `val` is out of range.
            #[inline]
            pub fn new(val: $Rep) -> Option<Self> {
                if val > MAX_REP || val < MIN_REP {
                    None
                } else {
                    Some($T(val))
                }
            }

            /// Constructs a new sample without checking for overflowing.
            ///
            /// This should *only* be used if the user can guarantee the sample will be within
            /// range and they require the extra performance.
            ///
            /// If this function is used, the sample crate can't guarantee that the returned sample
            /// or any interacting samples will remain within their MIN and MAX bounds.
            pub fn new_unchecked(s: $Rep) -> Self {
                $T(s)
            }

            /// Return the internal value used to represent the sample type.
            #[inline]
            pub fn inner(self) -> $Rep {
                self.0
            }

            /// Wraps self once in the case that self has overflowed.
            #[inline]
            fn wrap_overflow_once(self) -> Self {
                if      self.0 > MAX_REP { $T(self.0 - TOTAL) }
                else if self.0 < MIN_REP { $T(self.0 + TOTAL) }
                else                     { self }
            }

            /// Wraps self in the case that self has overflowed.
            #[inline]
            fn wrap_overflow(mut self) -> Self {
                while self.0 > MAX_REP {
                    self.0 -= TOTAL;
                }
                while self.0 < MIN_REP {
                    self.0 += TOTAL;
                }
                self
            }
        }

        impl ::core::ops::Add<$T> for $T {
            type Output = $T;
            #[inline]
            fn add(self, other: Self) -> Self {
                if cfg!(debug_assertions) {
                    $T::new(self.0 + other.0).expect("arithmetic operation overflowed")
                } else {
                    $T(self.0 + other.0).wrap_overflow_once()
                }
            }
        }

        impl ::core::ops::Sub<$T> for $T {
            type Output = $T;
            #[inline]
            fn sub(self, other: Self) -> Self {
                if cfg!(debug_assertions) {
                    $T::new(self.0 - other.0).expect("arithmetic operation overflowed")
                } else {
                    $T(self.0 - other.0).wrap_overflow_once()
                }
            }
        }

        impl ::core::ops::Mul<$T> for $T {
            type Output = $T;
            #[inline]
            fn mul(self, other: Self) -> Self {
                if cfg!(debug_assertions) {
                    $T::new(self.0 * other.0).expect("arithmetic operation overflowed")
                } else {
                    $T::from(self.0 * other.0)
                }
            }
        }

        impl ::core::ops::Div<$T> for $T {
            type Output = $T;
            #[inline]
            fn div(self, other: Self) -> Self {
                $T(self.0 / other.0)
            }
        }

        impl ::core::ops::Not for $T {
            type Output = $T;
            #[inline]
            fn not(self) -> $T {
                $T(!self.0)
            }
        }

        impl ::core::ops::Rem<$T> for $T {
            type Output = $T;
            #[inline]
            fn rem(self, other: Self) -> Self {
                $T(self.0 % other.0)
            }
        }

        impl ::core::ops::Shl<$T> for $T {
            type Output = $T;
            #[inline]
            fn shl(self, other: Self) -> Self {
                // TODO: Needs review
                $T(self.0 << other.0)
            }
        }

        impl ::core::ops::Shr<$T> for $T {
            type Output = $T;
            #[inline]
            fn shr(self, other: Self) -> Self {
                // TODO: Needs review
                $T(self.0 >> other.0)
            }
        }

        impl ::core::ops::BitAnd<$T> for $T {
            type Output = $T;
            #[inline]
            fn bitand(self, other: Self) -> Self {
                $T(self.0 & other.0)
            }
        }

        impl ::core::ops::BitOr<$T> for $T {
            type Output = $T;
            #[inline]
            fn bitor(self, other: Self) -> Self {
                $T(self.0 | other.0)
            }
        }

        impl ::core::ops::BitXor<$T> for $T {
            type Output = $T;
            #[inline]
            fn bitxor(self, other: Self) -> Self {
                $T(self.0 ^ other.0)
            }
        }

        impl_froms!($T: $Rep, $($rest)*);
    };
}

// Expands to a packed 24-bit sample type; see the module docs. `$decode` and `$encode` carry the
// byte order and signedness, and are the only things that differ between the four types.
macro_rules! new_packed_sample_type {
    ($T:ident: $Unpacked:ident, eq: $EQ:expr, min: $MIN:expr, max: $MAX:expr,
     endian: $ENDIAN:expr, alsa: $ALSA:expr,
     decode: |$b:ident| $decode:expr, encode: |$v:ident| $encode:expr,
     from: $($rest:tt)*) => {
        pub const MIN: $T = $T::new_unchecked($MIN);
        pub const MAX: $T = $T::new_unchecked($MAX);
        pub const EQUILIBRIUM: $T = $T::new_unchecked($EQ);
        const MIN_VALUE: i32 = $MIN;
        const MAX_VALUE: i32 = $MAX;

        #[doc = concat!("A 24-bit sample packed into exactly three ", $ENDIAN, " bytes.")]
        ///
        #[doc = concat!(
            "Valid range `", stringify!($MIN), "..=", stringify!($MAX),
            "`, with equilibrium at `", stringify!($EQ), "`. Known to ALSA as `", $ALSA, "`."
        )]
        ///
        #[doc = concat!(
            "This type is ", $ENDIAN, " on every target; see the [`types`](crate::types) module docs for the \
             layout guarantees, which are what make the zero-copy reinterpretation possible."
        )]
        #[derive(Copy, Clone, PartialEq, Eq, Default)]
        #[repr(transparent)]
        pub struct $T([u8; 3]);

        impl $T {
        #[doc = concat!(
            "Construct a sample from its three packed bytes, read as ", $ENDIAN, ". Every byte \
             triple is a valid sample, so this cannot fail."
        )]
            #[inline]
            pub const fn from_bytes(bytes: [u8; 3]) -> Self {
                $T(bytes)
            }

        #[doc = concat!("The three packed bytes representing the sample, in ", $ENDIAN, " order.")]
            #[inline]
            pub const fn to_bytes(self) -> [u8; 3] {
                self.0
            }

            /// Construct a new sample if the given value is within range.
            ///
            /// Returns `None` if `val` is out of range.
            #[inline]
            pub fn new(val: i32) -> Option<Self> {
                if (MIN_VALUE..=MAX_VALUE).contains(&val) {
                    Some($T::new_unchecked(val))
                } else {
                    None
                }
            }

            /// Constructs a new sample without checking for overflowing.
            ///
            /// Unlike `I24::new_unchecked`, which stores an out-of-range value verbatim, there is
            /// nowhere to put the overshoot here, so `val` wraps modulo 2^24. Use [`Self::new`]
            /// where the value is not known to be in range.
            ///
            /// ```
            /// use dasp_sample::{I24, I24LE3};
            ///
            /// assert_eq!(I24::new_unchecked(8_388_608).inner(), 8_388_608); // stored verbatim
            /// assert_eq!(I24LE3::new_unchecked(8_388_608).inner(), -8_388_608); // wrapped
            /// ```
            #[inline]
            pub const fn new_unchecked($v: i32) -> Self {
                $T($encode)
            }

            /// The value represented by the sample, unpacked into an `i32`.
            #[inline]
            pub const fn inner(self) -> i32 {
                let $b = self.0;
                $decode
            }
        }

        impl From<$Unpacked> for $T {
            #[inline]
            fn from(other: $Unpacked) -> Self {
                $T::new_unchecked(other.inner())
            }
        }

        impl From<$T> for $Unpacked {
            #[inline]
            fn from(other: $T) -> Self {
                $Unpacked::new_unchecked(other.inner())
            }
        }

        // Derived comparison would compare the bytes in storage order, which is neither the
        // numeric order nor even consistent between the `LE3` and `BE3` types, so we decode.
        impl ::core::cmp::Ord for $T {
            #[inline]
            fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
                self.inner().cmp(&other.inner())
            }
        }

        impl ::core::cmp::PartialOrd for $T {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<::core::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        // Show the value rather than the storage, to match the padded sample types.
        impl ::core::fmt::Debug for $T {
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                f.debug_tuple(stringify!($T)).field(&self.inner()).finish()
            }
        }

        impl_packed_froms!($T, $($rest)*);
    };
}

pub mod i11 {
    new_sample_type!(I11: i16, eq: 0, min: -1024, max: 1023, total: 2048,
                     from: i8, u8);
    impl_neg!(I11);
}

pub mod i20 {
    use super::{I11, U11};
    new_sample_type!(I20: i32, eq: 0, min: -524_288, max: 524_287, total: 1_048_576,
                     from: i8, {I11:i16}, i16, u8, {U11:i16}, u16);
}

pub mod i24 {
    use super::{I20, U20};
    new_sample_type!(I24: i32, eq: 0, min: -8_388_608, max: 8_388_607, total: 16_777_216,
                     from: i8, i16, {I20:i32}, u8, u16, {U20:i32});
    impl_neg!(I24);
}

pub mod i24_le3 {
    use super::{I20, I24, U20};
    new_packed_sample_type!(I24LE3: I24, eq: 0, min: -8_388_608, max: 8_388_607,
                            endian: "little-endian", alsa: "S24_3LE",
                            // The most significant byte is cast via `i8` to sign extend.
                            decode: |b| (b[0] as i32) | ((b[1] as i32) << 8) | ((b[2] as i8 as i32) << 16),
                            encode: |v| [v as u8, (v >> 8) as u8, (v >> 16) as u8],
                            from: i8, i16, {I20}, i32, u8, u16, {U20});
    impl_packed_neg!(I24LE3);
    impl_packed_arithmetic!(I24LE3);
}

pub mod i24_be3 {
    use super::{I20, I24, U20};
    new_packed_sample_type!(I24BE3: I24, eq: 0, min: -8_388_608, max: 8_388_607,
                            endian: "big-endian", alsa: "S24_3BE",
                            decode: |b| (b[2] as i32) | ((b[1] as i32) << 8) | ((b[0] as i8 as i32) << 16),
                            encode: |v| [(v >> 16) as u8, (v >> 8) as u8, v as u8],
                            from: i8, i16, {I20}, i32, u8, u16, {U20});
    impl_packed_neg!(I24BE3);
    impl_packed_arithmetic!(I24BE3);
}

pub mod i48 {
    use super::{I20, I24, I24BE3, I24LE3, U20, U24, U24BE3, U24LE3};
    new_sample_type!(I48: i64, eq: 0, min: -140_737_488_355_328, max: 140_737_488_355_327, total: 281_474_976_710_656,
                     from: i8, i16, {I20:i32}, {I24:i32}, {I24LE3:i32}, {I24BE3:i32}, i32,
                     u8, u16, {U20:i32}, {U24:i32}, {U24LE3:i32}, {U24BE3:i32}, u32);
    impl_neg!(I48);
}

pub mod u11 {
    new_sample_type!(U11: i16, eq: 1024, min: 0, max: 2047, total: 2048,
                     from: u8);
    impl_neg!(U11);
}

pub mod u20 {
    new_sample_type!(U20: i32, eq: 524_288, min: 0, max: 1_048_575, total: 1_048_576,
                     from: u8, u16);
}

pub mod u24 {
    use super::U20;
    new_sample_type!(U24: i32, eq: 8_388_608, min: 0, max: 16_777_215, total: 16_777_216,
                     from: u8, u16, {U20:i32});
}

pub mod u24_le3 {
    use super::{U20, U24};
    new_packed_sample_type!(U24LE3: U24, eq: 8_388_608, min: 0, max: 16_777_215,
                            endian: "little-endian", alsa: "U24_3LE",
                            decode: |b| (b[0] as i32) | ((b[1] as i32) << 8) | ((b[2] as i32) << 16),
                            encode: |v| [v as u8, (v >> 8) as u8, (v >> 16) as u8],
                            from: i32, u8, u16, {U20});
    impl_packed_arithmetic!(U24LE3);
}

pub mod u24_be3 {
    use super::{U20, U24};
    new_packed_sample_type!(U24BE3: U24, eq: 8_388_608, min: 0, max: 16_777_215,
                            endian: "big-endian", alsa: "U24_3BE",
                            decode: |b| (b[2] as i32) | ((b[1] as i32) << 8) | ((b[0] as i32) << 16),
                            encode: |v| [(v >> 16) as u8, (v >> 8) as u8, v as u8],
                            from: i32, u8, u16, {U20});
    impl_packed_arithmetic!(U24BE3);
}

pub mod u48 {
    use super::{U20, U24, U24BE3, U24LE3};
    new_sample_type!(U48: i64, eq: 140_737_488_355_328, min: 0, max: 281_474_976_710_655, total: 281_474_976_710_656,
                     from: u8, u16, {U20:i32}, {U24:i32}, {U24LE3:i32}, {U24BE3:i32}, u32);
}
