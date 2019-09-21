//! Pure functions and traits for converting between i8, i16, I24, i32, I48, i64, u8, u16, U24,
//! u32, U48, u64, f32 and f64.
//!
//! Each conversion function is performance focused, memory-sensitive and expects that the user has
//! validated their input prior to the function call.
//!
//! No conversion function will ever cast to a type with a size in bytes larger than the largest
//! between the source and target sample types.
//!
//! The conversion functions do *not* check the range of incoming values for floating point values
//! or any of the custom `I24`, `U24`, `I48` and `U48` types.
//!
//! Note that floating point conversions use the range -1.0 <= v < 1.0:
//! `(1.0 as f64).to_sample::<i16>()` will overflow!

use {Box, Sample};
#[cfg(feature = "frame")]
use Frame;
use core;
use types::{I24, U24, I48, U48};


macro_rules! conversion_fn {

    ($Rep:ty, $s:ident to_i8 { $body:expr }) => {
        #[inline]
        pub fn to_i8($s: $Rep) -> i8 {
            $body
        }
    };

    ($Rep:ty, $s:ident to_i16 { $body:expr }) => {
        #[inline]
        pub fn to_i16($s: $Rep) -> i16 {
            $body
        }
    };

    ($Rep:ty, $s:ident to_i24 { $body:expr }) => {
        #[inline]
        pub fn to_i24($s: $Rep) -> I24 {
            $body
        }
    };

    ($Rep:ty, $s:ident to_i32 { $body:expr }) => {
        #[inline]
        pub fn to_i32($s: $Rep) -> i32 {
            $body
        }
    };

    ($Rep:ty, $s:ident to_i48 { $body:expr }) => {
        #[inline]
        pub fn to_i48($s: $Rep) -> I48 {
            $body
        }
    };

    ($Rep:ty, $s:ident to_i64 { $body:expr }) => {
        #[inline]
        pub fn to_i64($s: $Rep) -> i64 {
            $body
        }
    };

    ($Rep:ty, $s:ident to_u8 { $body:expr }) => {
        #[inline]
        pub fn to_u8($s: $Rep) -> u8 {
            $body
        }
    };

    ($Rep:ty, $s:ident to_u16 { $body:expr }) => {
        #[inline]
        pub fn to_u16($s: $Rep) -> u16 {
            $body
        }
    };

    ($Rep:ty, $s:ident to_u24 { $body:expr }) => {
        #[inline]
        pub fn to_u24($s: $Rep) -> U24 {
            $body
        }
    };

    ($Rep:ty, $s:ident to_u32 { $body:expr }) => {
        #[inline]
        pub fn to_u32($s: $Rep) -> u32 {
            $body
        }
    };

    ($Rep:ty, $s:ident to_u48 { $body:expr }) => {
        #[inline]
        pub fn to_u48($s: $Rep) -> U48 {
            $body
        }
    };

    ($Rep:ty, $s:ident to_u64 { $body:expr }) => {
        #[inline]
        pub fn to_u64($s: $Rep) -> u64 {
            $body
        }
    };

    ($Rep:ty, $s:ident to_f32 { $body:expr }) => {
        #[inline]
        pub fn to_f32($s: $Rep) -> f32 {
            $body
        }
    };

    ($Rep:ty, $s:ident to_f64 { $body:expr }) => {
        #[inline]
        pub fn to_f64($s: $Rep) -> f64 {
            $body
        }
    };

}

macro_rules! conversion_fns {
    ($Rep:ty, $s:ident $fn_name:tt { $body:expr } $($rest:tt)*) => {
        conversion_fn!($Rep, $s $fn_name { $body });
        conversion_fns!($Rep, $($rest)*);
    };
    ($Rep:ty, ) => {};
}

macro_rules! conversions {
    ($T:ident, $mod_name:ident { $($rest:tt)* }) => {
        pub mod $mod_name {
            use $crate::types::{I24, U24, I48, U48};
            conversion_fns!($T, $($rest)*);
        }
    };
}


conversions!(i8, i8 {
    s to_i16 { (s as i16) << 8 }
    s to_i24 { I24::new_unchecked((s as i32) << 16) }
    s to_i32 { (s as i32) << 24 }
    s to_i48 { I48::new_unchecked((s as i64) << 40) }
    s to_i64 { (s as i64) << 56 }
    s to_u8 {
        if s < 0 {
            // 128i8 overflows, so we must use 127 + 1 instead.
            (s + 127 + 1) as u8
        } else {
            (s as u8) + 128
        }
    }
    s to_u16 {
        if s < 0 {
            ((s + 127 + 1) as u16) << 8
        } else {
            (s as u16 + 128) << 8
        }
    }
    s to_u24 {
        U24::new_unchecked((s as i32 + 128) << 16)
    }
    s to_u32 {
        if s < 0 {
            ((s + 127 + 1) as u32) << 24
        } else {
            (s as u32 + 128) << 24
        }
    }
    s to_u48 {
        U48::new_unchecked((s as i64 + 128) << 40)
    }
    s to_u64 {
        if s < 0 {
            ((s + 127 + 1) as u64) << 56
        } else {
            (s as u64 + 128) << 56
        }
    }
    s to_f32 {
        s as f32 / 128.0
    }
    s to_f64 {
        s as f64 / 128.0
    }
});

conversions!(i16, i16 {
    s to_i8 { (s >> 8) as i8 }
    s to_i24 { I24::new_unchecked((s as i32) << 8) }
    s to_i32 { (s as i32) << 16 }
    s to_i48 { I48::new_unchecked((s as i64) << 32) }
    s to_i64 { (s as i64) << 48 }
    s to_u8 {
        super::i8::to_u8(to_i8(s))
    }
    s to_u16 {
        if s < 0 {
            // 32_768i16 overflows, so we must use + 1 instead.
            (s + 32_767 + 1) as u16
        } else {
            s as u16 + 32_768
        }
    }
    s to_u24 {
        if s < 0 {
            U24::new_unchecked(((s + 32_767 + 1) as i32) << 8)
        } else {
            U24::new_unchecked((s as i32 + 32_768) << 8)
        }
    }
    s to_u32 {
        if s < 0 {
            ((s + 32_767 + 1) as u32) << 16
        } else {
            ((s as u32) + 32_768) << 16
        }
    }
    s to_u48 {
        if s < 0 {
            U48::new_unchecked(((s + 32_767 + 1) as i64) << 32)
        } else {
            U48::new_unchecked((s as i64 + 32_768) << 32)
        }
    }
    s to_u64 {
        if s < 0 {
            ((s + 32_767 + 1) as u64) << 48
        } else {
            ((s as u64) + 32_768) << 48
        }
    }
    s to_f32 {
        s as f32 / 32_768.0
    }
    s to_f64 {
        s as f64 / 32_768.0
    }
});

conversions!(I24, i24 {
    s to_i8 { (s.inner() >> 16) as i8 }
    s to_i16 { (s.inner() >> 8) as i16 }
    s to_i32 { s.inner() << 8 }
    s to_i48 { I48::new_unchecked((s.inner() as i64) << 24) }
    s to_i64 { (s.inner() as i64) << 40 }
    s to_u8 {
        super::i8::to_u8(to_i8(s))
    }
    s to_u16 {
        super::i16::to_u16(to_i16(s))
    }
    s to_u24 {
        U24::new_unchecked(s.inner() + 8_388_608)
    }
    s to_u32 {
        ((s.inner() + 8_388_608) as u32) << 8
    }
    s to_u48 {
        U48::new_unchecked((s.inner() as i64 + 8_388_608) << 24)
    }
    s to_u64 {
        ((s.inner() + 8_388_608) as u64) << 40
    }
    s to_f32 {
        s.inner() as f32 / 8_388_608.0
    }
    s to_f64 {
        s.inner() as f64 / 8_388_608.0
    }
});

conversions!(i32, i32 {
    s to_i8 { (s >> 24) as i8 }
    s to_i16 { (s >> 16) as i16 }
    s to_i24 { I24::new_unchecked(s >> 8) }
    s to_i48 { I48::new_unchecked((s as i64) << 16) }
    s to_i64 { (s as i64) << 32 }
    s to_u8 {
        super::i8::to_u8(to_i8(s))
    }
    s to_u16 {
        super::i16::to_u16(to_i16(s))
    }
    s to_u24 {
        super::i24::to_u24(to_i24(s))
    }
    s to_u32 {
        if s < 0 {
            ((s + 2_147_483_647 + 1) as u32)
        } else {
            s as u32 + 2_147_483_648
        }
    }
    s to_u48 {
        U48::new_unchecked((s as i64 + 2_147_483_648) << 16)
    }
    s to_u64 {
        if s < 0 {
            ((s + 2_147_483_647 + 1) as u64) << 32
        } else {
            (s as u64) + 2_147_483_648 << 32
        }
    }
    s to_f32 {
        s as f32 / 2_147_483_648.0
    }
    s to_f64 {
        s as f64 / 2_147_483_648.0
    }
});

conversions!(I48, i48 {
    s to_i8 { (s.inner() >> 40) as i8 }
    s to_i16 { (s.inner() >> 32) as i16 }
    s to_i24 { I24::new_unchecked((s.inner() >> 24) as i32) }
    s to_i32 { (s.inner() >> 16) as i32 }
    s to_i64 { s.inner() << 16 }
    s to_u8 {
        super::i8::to_u8(to_i8(s))
    }
    s to_u16 {
        super::i16::to_u16(to_i16(s))
    }
    s to_u24 {
        super::i24::to_u24(to_i24(s))
    }
    s to_u32 {
        super::i32::to_u32(to_i32(s))
    }
    s to_u48 {
        U48::new_unchecked(s.inner() + 140_737_488_355_328)
    }
    s to_u64 {
        ((s.inner() + 140_737_488_355_328) as u64) << 16
    }
    s to_f32 {
        s.inner() as f32 / 140_737_488_355_328.0
    }
    s to_f64 {
        s.inner() as f64 / 140_737_488_355_328.0
    }
});

conversions!(i64, i64 {
    s to_i8 { (s >> 56) as i8 }
    s to_i16 { (s >> 48) as i16 }
    s to_i24 { I24::new_unchecked((s >> 40) as i32) }
    s to_i32 { (s >> 32) as i32 }
    s to_i48 { I48::new_unchecked(s >> 16) }
    s to_u8 {
        super::i8::to_u8(to_i8(s))
    }
    s to_u16 {
        super::i16::to_u16(to_i16(s))
    }
    s to_u24 {
        super::i24::to_u24(to_i24(s))
    }
    s to_u32 {
        super::i32::to_u32(to_i32(s))
    }
    s to_u48 {
        super::i48::to_u48(to_i48(s))
    }
    s to_u64 {
        if s < 0 {
            (s + 9_223_372_036_854_775_807 + 1) as u64
        } else {
            s as u64 + 9_223_372_036_854_775_808
        }
    }
    s to_f32 {
        s as f32 / 9_223_372_036_854_775_808.0
    }
    s to_f64 {
        s as f64 / 9_223_372_036_854_775_808.0
    }
});

conversions!(u8, u8 {
    s to_i8 {
        if s < 128 {
            s as i8 - 127 - 1
        } else {
            (s - 128) as i8
        }
    }
    s to_i16 {
        (s as i16 - 128) << 8
    }
    s to_i24 {
        I24::new_unchecked((s as i32 - 128) << 16)
    }
    s to_i32 {
        (s as i32 - 128) << 24
    }
    s to_i48 {
        I48::new_unchecked((s as i64 - 128) << 40)
    }
    s to_i64 {
        (s as i64 - 128) << 56
    }
    s to_u16 { (s as u16) << 8 }
    s to_u24 { U24::new_unchecked((s as i32) << 16) }
    s to_u32 { (s as u32) << 24 }
    s to_u48 { U48::new_unchecked((s as i64) << 40) }
    s to_u64 { (s as u64) << 56 }
    s to_f32 { super::i8::to_f32(to_i8(s)) }
    s to_f64 { super::i8::to_f64(to_i8(s)) }
});

conversions!(u16, u16 {
    s to_i8 { super::u8::to_i8(to_u8(s)) }
    s to_i16 {
        if s < 32_768 {
            s as i16 - 32_767 - 1
        } else {
            (s - 32_768) as i16
        }
    }
    s to_i24 {
        I24::new_unchecked((s as i32 - 32_768) << 8)
    }
    s to_i32 {
        (s as i32 - 32_768) << 16
    }
    s to_i48 {
        I48::new_unchecked((s as i64 - 32_768) << 32)
    }
    s to_i64 {
        (s as i64 - 32_768) << 48
    }
    s to_u8 { (s >> 8) as u8 }
    s to_u24 { U24::new_unchecked((s as i32) << 8) }
    s to_u32 { (s as u32) << 16 }
    s to_u48 { U48::new_unchecked((s as i64) << 32) }
    s to_u64 { (s as u64) << 48 }
    s to_f32 { super::i16::to_f32(to_i16(s)) }
    s to_f64 { super::i16::to_f64(to_i16(s)) }
});

conversions!(U24, u24 {
    s to_i8 { super::u8::to_i8(to_u8(s)) }
    s to_i16 { super::u16::to_i16(to_u16(s)) }
    s to_i24 {
        I24::new_unchecked(s.inner() - 8_388_608)
    }
    s to_i32 {
        (s.inner() - 8_388_608) << 8
    }
    s to_i48 {
        I48::new_unchecked(((s.inner() as i64) - 8_388_608) << 24)
    }
    s to_i64 {
        (s.inner() as i64 - 8_388_608) << 40
    }
    s to_u8 { (s.inner() >> 16) as u8 }
    s to_u16 { (s.inner() >> 8) as u16 }
    s to_u32 { (s.inner() as u32) << 8 }
    s to_u48 { U48::new_unchecked((s.inner() as i64) << 24) }
    s to_u64 { (s.inner() as u64) << 40 }
    s to_f32 { super::i24::to_f32(to_i24(s)) }
    s to_f64 { super::i24::to_f64(to_i24(s)) }
});

conversions!(u32, u32 {
    s to_i8 { super::u8::to_i8(to_u8(s)) }
    s to_i16 { super::u16::to_i16(to_u16(s)) }
    s to_i24 { super::u24::to_i24(to_u24(s)) }
    s to_i32 {
        if s < 2_147_483_648 {
            s as i32 - 2_147_483_647 - 1
        } else {
            (s - 2_147_483_648) as i32
        }
    }
    s to_i48 {
        I48::new_unchecked((s as i64 - 2_147_483_648) << 16)
    }
    s to_i64 {
        (s as i64 - 2_147_483_648) << 32
    }
    s to_u8 { (s >> 24) as u8 }
    s to_u16 { (s >> 16) as u16 }
    s to_u24 { U24::new_unchecked((s >> 8) as i32) }
    s to_u48 { U48::new_unchecked((s as i64) << 16) }
    s to_u64 { (s as u64) << 32 }
    s to_f32 { super::i32::to_f32(to_i32(s)) }
    s to_f64 { super::i32::to_f64(to_i32(s)) }
});

conversions!(U48, u48 {
    s to_i8 { super::u8::to_i8(to_u8(s)) }
    s to_i16 { super::u16::to_i16(to_u16(s)) }
    s to_i24 { super::u24::to_i24(to_u24(s)) }
    s to_i32 { super::u32::to_i32(to_u32(s)) }
    s to_i48 {
        I48::new_unchecked(s.inner() - 140_737_488_355_328)
    }
    s to_i64 {
        (s.inner() - 140_737_488_355_328) << 16
    }
    s to_u8 { (s.inner() >> 40) as u8 }
    s to_u16 { (s.inner() >> 32) as u16 }
    s to_u24 { U24::new_unchecked((s.inner() >> 24) as i32) }
    s to_u32 { (s.inner() >> 16) as u32 }
    s to_u64 { (s.inner() as u64) << 16 }
    s to_f32 { super::i48::to_f32(to_i48(s)) }
    s to_f64 { super::i48::to_f64(to_i48(s)) }
});

conversions!(u64, u64 {
    s to_i8 { super::u8::to_i8(to_u8(s)) }
    s to_i16 { super::u16::to_i16(to_u16(s)) }
    s to_i24 { super::u24::to_i24(to_u24(s)) }
    s to_i32 { super::u32::to_i32(to_u32(s)) }
    s to_i48 { super::u48::to_i48(to_u48(s)) }
    s to_i64 {
        if s < 9_223_372_036_854_775_808 {
            s as i64 - 9_223_372_036_854_775_807 - 1
        } else {
            (s - 9_223_372_036_854_775_808) as i64
        }
    }
    s to_u8 { (s >> 56) as u8 }
    s to_u16 { (s >> 48) as u16 }
    s to_u24 { U24::new_unchecked((s >> 40) as i32) }
    s to_u32 { (s >> 32) as u32 }
    s to_u48 { U48::new_unchecked((s >> 16) as i64) }
    s to_f32 { super::i64::to_f32(to_i64(s)) }
    s to_f64 { super::i64::to_f64(to_i64(s)) }
});

// The following conversions assume `-1.0 <= s < 1.0` (note that +1.0 is excluded) and will
// overflow otherwise.
conversions!(f32, f32 {
    s to_i8 { (s * 128.0) as i8 }
    s to_i16 { (s * 32_768.0) as i16 }
    s to_i24 { I24::new_unchecked((s * 8_388_608.0) as i32) }
    s to_i32 { (s * 2_147_483_648.0) as i32 }
    s to_i48 { I48::new_unchecked((s * 140_737_488_355_328.0) as i64) }
    s to_i64 { (s * 9_223_372_036_854_775_808.0) as i64 }
    s to_u8 { super::i8::to_u8(to_i8(s)) }
    s to_u16 { super::i16::to_u16(to_i16(s)) }
    s to_u24 { super::i24::to_u24(to_i24(s)) }
    s to_u32 { super::i32::to_u32(to_i32(s)) }
    s to_u48 { super::i48::to_u48(to_i48(s)) }
    s to_u64 { super::i64::to_u64(to_i64(s)) }
    s to_f64 { s as f64 }
});

// The following conversions assume `-1.0 <= s < 1.0` (note that +1.0 is excluded) and will
// overflow otherwise.
conversions!(f64, f64 {
    s to_i8 { (s * 128.0) as i8 }
    s to_i16 { (s * 32_768.0) as i16 }
    s to_i24 { I24::new_unchecked((s * 8_388_608.0) as i32) }
    s to_i32 { (s * 2_147_483_648.0) as i32 }
    s to_i48 { I48::new_unchecked((s * 140_737_488_355_328.0) as i64) }
    s to_i64 { (s * 9_223_372_036_854_775_808.0) as i64 }
    s to_u8 { super::i8::to_u8(to_i8(s)) }
    s to_u16 { super::i16::to_u16(to_i16(s)) }
    s to_u24 { super::i24::to_u24(to_i24(s)) }
    s to_u32 { super::i32::to_u32(to_i32(s)) }
    s to_u48 { super::i48::to_u48(to_i48(s)) }
    s to_u64 { super::i64::to_u64(to_i64(s)) }
    s to_f32 { s as f32 }
});


/// Similar to the std `From` trait, but specifically for converting between sample types.
///
/// We use this trait to be generic over the `Sample::to_sample` and `Sample::from_sample` methods.
pub trait FromSample<S> {
    fn from_sample_(s: S) -> Self;
}

impl<S> FromSample<S> for S {
    #[inline]
    fn from_sample_(s: S) -> Self {
        s
    }
}

/// Implement the `FromSample` trait for the given types.
macro_rules! impl_from_sample {
    ($T:ty, $fn_name:ident from $({$U:ident: $Umod:ident})*) => {
        $(
            impl FromSample<$U> for $T {
                #[inline]
                fn from_sample_(s: $U) -> Self {
                    self::$Umod::$fn_name(s)
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
    fn to_sample_(self) -> S;
}

impl<T, U> ToSample<U> for T
where
    U: FromSample<T>,
{
    #[inline]
    fn to_sample_(self) -> U {
        U::from_sample_(self)
    }
}

/// Sample types which may be converted to and from some type `S`.
pub trait Duplex<S>: FromSample<S> + ToSample<S> {}
impl<S, T> Duplex<S> for T
where
    T: FromSample<S> + ToSample<S>,
{
}


///// DSP Slice Conversion Traits


/// For converting from a slice of `Sample`s to a slice of `Frame`s.
pub trait FromSampleSlice<'a, S>: Sized
where
    S: Sample,
{
    fn from_sample_slice(slice: &'a [S]) -> Option<Self>;
}

/// For converting from a mutable slice of `Sample`s to a mutable slice of `Frame`s.
pub trait FromSampleSliceMut<'a, S>: Sized
where
    S: Sample,
{
    fn from_sample_slice_mut(slice: &'a mut [S]) -> Option<Self>;
}

/// For converting a boxed slice of `Sample`s to a boxed slice of `Frame`s.
pub trait FromBoxedSampleSlice<S>: Sized
where
    S: Sample,
{
    fn from_boxed_sample_slice(slice: Box<[S]>) -> Option<Self>;
}

#[cfg(feature = "frame")]
/// For converting from a slice of `Frame`s to a slice of `Sample`s.
pub trait FromFrameSlice<'a, F>
where
    F: Frame,
{
    fn from_frame_slice(slice: &'a [F]) -> Self;
}

#[cfg(feature = "frame")]
/// For converting from a slice of `Frame`s to a slice of `Sample`s.
pub trait FromFrameSliceMut<'a, F>
where
    F: Frame,
{
    fn from_frame_slice_mut(slice: &'a mut [F]) -> Self;
}

#[cfg(feature = "frame")]
/// For converting from a boxed slice of `Frame`s to a boxed slice of `Sample`s.
pub trait FromBoxedFrameSlice<F>
where
    F: Frame,
{
    fn from_boxed_frame_slice(slice: Box<[F]>) -> Self;
}

/// For converting from a slice of `Frame`s to a slice of `Sample`s.
pub trait ToSampleSlice<'a, S>
where
    S: Sample,
{
    fn to_sample_slice(self) -> &'a [S];
}

/// For converting from a mutable slice of `Frame`s to a mutable slice of `Sample`s.
pub trait ToSampleSliceMut<'a, S>
where
    S: Sample,
{
    fn to_sample_slice_mut(self) -> &'a mut [S];
}

/// For converting from a boxed slice of `Frame`s to a boxed slice of `Sample`s.
pub trait ToBoxedSampleSlice<S>
where
    S: Sample,
{
    fn to_boxed_sample_slice(self) -> Box<[S]>;
}

#[cfg(feature = "frame")]
/// For converting from a slice of `Sample`s to a slice of `Frame`s.
pub trait ToFrameSlice<'a, F>
where
    F: Frame,
{
    fn to_frame_slice(self) -> Option<&'a [F]>;
}

#[cfg(feature = "frame")]
/// For converting from a mutable slice of `Sample`s to a mutable slice of `Frame`s.
pub trait ToFrameSliceMut<'a, F>
where
    F: Frame,
{
    fn to_frame_slice_mut(self) -> Option<&'a mut [F]>;
}

#[cfg(feature = "frame")]
/// For converting from a boxed slice of `Sample`s to a boxed slice of `Frame`s.
pub trait ToBoxedFrameSlice<F>
where
    F: Frame,
{
    fn to_boxed_frame_slice(self) -> Option<Box<[F]>>;
}


///// DSP Slice Conversion Trait Implementations


impl<'a, S> FromSampleSlice<'a, S> for &'a [S]
where
    S: Sample,
{
    #[inline]
    fn from_sample_slice(slice: &'a [S]) -> Option<Self> {
        Some(slice)
    }
}

impl<'a, S> FromSampleSliceMut<'a, S> for &'a mut [S]
where
    S: Sample,
{
    #[inline]
    fn from_sample_slice_mut(slice: &'a mut [S]) -> Option<Self> {
        Some(slice)
    }
}

impl<S> FromBoxedSampleSlice<S> for Box<[S]>
where
    S: Sample,
{
    #[inline]
    fn from_boxed_sample_slice(slice: Box<[S]>) -> Option<Self> {
        Some(slice)
    }
}

#[cfg(feature = "frame")]
impl<'a, F> FromFrameSlice<'a, F> for &'a [F]
where
    F: Frame,
{
    #[inline]
    fn from_frame_slice(slice: &'a [F]) -> Self {
        slice
    }
}

#[cfg(feature = "frame")]
impl<'a, F> FromFrameSliceMut<'a, F> for &'a mut [F]
where
    F: Frame,
{
    #[inline]
    fn from_frame_slice_mut(slice: &'a mut [F]) -> Self {
        slice
    }
}

#[cfg(feature = "frame")]
impl<F> FromBoxedFrameSlice<F> for Box<[F]>
where
    F: Frame,
{
    #[inline]
    fn from_boxed_frame_slice(slice: Box<[F]>) -> Self {
        slice
    }
}

impl<'a, S> ToSampleSlice<'a, S> for &'a [S]
where
    S: Sample,
{
    #[inline]
    fn to_sample_slice(self) -> &'a [S] {
        self
    }
}

impl<'a, S> ToSampleSliceMut<'a, S> for &'a mut [S]
where
    S: Sample,
{
    #[inline]
    fn to_sample_slice_mut(self) -> &'a mut [S] {
        self
    }
}

impl<S> ToBoxedSampleSlice<S> for Box<[S]>
where
    S: Sample,
{
    #[inline]
    fn to_boxed_sample_slice(self) -> Box<[S]> {
        self
    }
}

#[cfg(feature = "frame")]
impl<'a, F> ToFrameSlice<'a, F> for &'a [F]
where
    F: Frame,
{
    #[inline]
    fn to_frame_slice(self) -> Option<&'a [F]> {
        Some(self)
    }
}

#[cfg(feature = "frame")]
impl<'a, F> ToFrameSliceMut<'a, F> for &'a mut [F]
where
    F: Frame,
{
    #[inline]
    fn to_frame_slice_mut(self) -> Option<&'a mut [F]> {
        Some(self)
    }
}

#[cfg(feature = "frame")]
impl<F> ToBoxedFrameSlice<F> for Box<[F]>
where
    F: Frame,
{
    #[inline]
    fn to_boxed_frame_slice(self) -> Option<Box<[F]>> {
        Some(self)
    }
}

#[cfg(feature = "frame")]
/// A macro for implementing all audio slice conversion traits for each fixed-size array.
macro_rules! impl_from_slice_conversions {
    ($($N:expr)*) => {
        $(

            impl<'a, S> FromSampleSlice<'a, S> for &'a [[S; $N]]
                where S: Sample,
                      [S; $N]: Frame,
            {
                #[inline]
                fn from_sample_slice(slice: &'a [S]) -> Option<Self> {
                    let len = slice.len();
                    if len % $N == 0 {
                        let new_len = len / $N;
                        let ptr = slice.as_ptr() as *const _;
                        let new_slice = unsafe {
                            core::slice::from_raw_parts(ptr, new_len)
                        };
                        Some(new_slice)
                    } else {
                        None
                    }
                }
            }

            impl<'a, S> FromSampleSliceMut<'a, S> for &'a mut [[S; $N]]
                where S: Sample,
                      [S; $N]: Frame,
            {
                #[inline]
                fn from_sample_slice_mut(slice: &'a mut [S]) -> Option<Self> {
                    let len = slice.len();
                    if len % $N == 0 {
                        let new_len = len / $N;
                        let ptr = slice.as_ptr() as *mut _;
                        let new_slice = unsafe {
                            core::slice::from_raw_parts_mut(ptr, new_len)
                        };
                        Some(new_slice)
                    } else {
                        None
                    }
                }
            }

            impl<S> FromBoxedSampleSlice<S> for Box<[[S; $N]]>
                where S: Sample,
                      [S; $N]: Frame,
            {
                #[inline]
                fn from_boxed_sample_slice(mut slice: Box<[S]>) -> Option<Self> {

                    // First, we need a raw pointer to the slice and to make sure that the `Box` is
                    // forgotten so that our slice does not get deallocated.
                    let len = slice.len();
                    let slice_ptr = &mut slice as &mut [S] as *mut [S];
                    core::mem::forget(slice);
                    let sample_slice = unsafe {
                        core::slice::from_raw_parts_mut((*slice_ptr).as_mut_ptr(), len)
                    };

                    // Convert to our frame slice if possible.
                    let frame_slice = match <&mut [[S; $N]]>::from_sample_slice_mut(sample_slice) {
                        Some(slice) => slice,
                        None => return None,
                    };
                    let ptr = frame_slice as *mut [[S; $N]];

                    // Take ownership over the slice again before returning it.
                    let new_slice = unsafe {
                        Box::from_raw(ptr)
                    };

                    Some(new_slice)
                }
            }

            impl<'a, S> FromFrameSlice<'a, [S; $N]> for &'a [S]
                where [S; $N]: Frame,
            {
                #[inline]
                fn from_frame_slice(slice: &'a [[S; $N]]) -> Self {
                    let new_len = slice.len() * $N;
                    let ptr = slice.as_ptr() as *const _;
                    unsafe {
                        core::slice::from_raw_parts(ptr, new_len)
                    }
                }
            }

            impl<'a, S> FromFrameSliceMut<'a, [S; $N]> for &'a mut [S]
                where [S; $N]: Frame,
            {
                #[inline]
                fn from_frame_slice_mut(slice: &'a mut [[S; $N]]) -> Self {
                    let new_len = slice.len() * $N;
                    let ptr = slice.as_ptr() as *mut _;
                    unsafe {
                        core::slice::from_raw_parts_mut(ptr, new_len)
                    }
                }
            }

            impl<S> FromBoxedFrameSlice<[S; $N]> for Box<[S]>
                where [S; $N]: Frame,
            {
                #[inline]
                fn from_boxed_frame_slice(mut slice: Box<[[S; $N]]>) -> Self {
                    let new_len = slice.len() * $N;
                    let frame_slice_ptr = &mut slice as &mut [[S; $N]] as *mut [[S; $N]];
                    core::mem::forget(slice);
                    let sample_slice_ptr = frame_slice_ptr as *mut [S];
                    unsafe {
                        let ptr = (*sample_slice_ptr).as_mut_ptr();
                        let sample_slice = core::slice::from_raw_parts_mut(ptr, new_len);
                        Box::from_raw(sample_slice as *mut _)
                    }
                }
            }

            impl<'a, S> ToSampleSlice<'a, S> for &'a [[S; $N]]
                where S: Sample,
            {
                #[inline]
                fn to_sample_slice(self) -> &'a [S] {
                    FromFrameSlice::from_frame_slice(self)
                }
            }

            impl<'a, S> ToSampleSliceMut<'a, S> for &'a mut [[S; $N]]
                where S: Sample,
            {
                #[inline]
                fn to_sample_slice_mut(self) -> &'a mut [S] {
                    FromFrameSliceMut::from_frame_slice_mut(self)
                }
            }

            impl<S> ToBoxedSampleSlice<S> for Box<[[S; $N]]>
                where S: Sample,
            {
                #[inline]
                fn to_boxed_sample_slice(self) -> Box<[S]> {
                    FromBoxedFrameSlice::from_boxed_frame_slice(self)
                }
            }

            impl<'a, S> ToFrameSlice<'a, [S; $N]> for &'a [S]
                where S: Sample,
                      [S; $N]: Frame,
            {
                #[inline]
                fn to_frame_slice(self) -> Option<&'a [[S; $N]]> {
                    FromSampleSlice::from_sample_slice(self)
                }
            }

            impl<'a, S> ToFrameSliceMut<'a, [S; $N]> for &'a mut [S]
                where S: Sample,
                      [S; $N]: Frame,
            {
                #[inline]
                fn to_frame_slice_mut(self) -> Option<&'a mut [[S; $N]]> {
                    FromSampleSliceMut::from_sample_slice_mut(self)
                }
            }

            impl<S> ToBoxedFrameSlice<[S; $N]> for Box<[S]>
                where S: Sample,
                      [S; $N]: Frame,
            {
                #[inline]
                fn to_boxed_frame_slice(self) -> Option<Box<[[S; $N]]>> {
                    FromBoxedSampleSlice::from_boxed_sample_slice(self)
                }
            }

        )*
    };
}

#[cfg(feature = "frame")]
impl_from_slice_conversions! {
    1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32
}


///// Bi-Directional DSP Slice Conversion Traits


pub trait DuplexSampleSlice<'a, S>
    : FromSampleSlice<'a, S> + ToSampleSlice<'a, S>
where
    S: Sample
{
}

#[cfg(feature = "frame")]
pub trait DuplexFrameSlice<'a, F>: FromFrameSlice<'a, F> + ToFrameSlice<'a, F>
where
    F: Frame
{
}

#[cfg(feature = "frame")]
pub trait DuplexSlice<'a, S, F>
    : DuplexSampleSlice<'a, S> + DuplexFrameSlice<'a, F>
where
    S: Sample,
    F: Frame<Sample = S>
{
}

pub trait DuplexSampleSliceMut<'a, S>
    : FromSampleSliceMut<'a, S> + ToSampleSliceMut<'a, S>
where
    S: Sample
{
}

#[cfg(feature = "frame")]
pub trait DuplexFrameSliceMut<'a, F>
    : FromFrameSliceMut<'a, F> + ToFrameSliceMut<'a, F>
where
    F: Frame
{
}

#[cfg(feature = "frame")]
pub trait DuplexSliceMut<'a, S, F>
    : DuplexSampleSliceMut<'a, S> + DuplexFrameSliceMut<'a, F>
where
    S: Sample,
    F: Frame<Sample = S>
{
}

pub trait DuplexBoxedSampleSlice<S>
    : FromBoxedSampleSlice<S> + ToBoxedSampleSlice<S>
where
    S: Sample
{
}

#[cfg(feature = "frame")]
pub trait DuplexBoxedFrameSlice<F>
    : FromBoxedFrameSlice<F> + ToBoxedFrameSlice<F>
where
    F: Frame
{
}

#[cfg(feature = "frame")]
pub trait DuplexBoxedSlice<S, F>
    : DuplexBoxedSampleSlice<S> + DuplexBoxedFrameSlice<F>
where
    S: Sample,
    F: Frame<Sample = S>
{
}

///// Bi-Directional DSP Slice Conversion Trait Implementations


impl<'a, S, T> DuplexSampleSlice<'a, S> for T
where
    S: Sample,
    T: FromSampleSlice<'a, S> + ToSampleSlice<'a, S>,
{
}

#[cfg(feature = "frame")]
impl<'a, F, T> DuplexFrameSlice<'a, F> for T
where
    F: Frame,
    T: FromFrameSlice<'a, F> + ToFrameSlice<'a, F>,
{
}

#[cfg(feature = "frame")]
impl<'a, S, F, T> DuplexSlice<'a, S, F> for T
where
    S: Sample,
    F: Frame<Sample = S>,
    T: DuplexSampleSlice<'a, S> + DuplexFrameSlice<'a, F>,
{
}

impl<'a, S, T> DuplexSampleSliceMut<'a, S> for T
where
    S: Sample,
    T: FromSampleSliceMut<'a, S> + ToSampleSliceMut<'a, S> {}

#[cfg(feature = "frame")]
impl<'a, F, T> DuplexFrameSliceMut<'a, F> for T
where
    F: Frame,
    T: FromFrameSliceMut<'a, F> + ToFrameSliceMut<'a, F>,
{
}

#[cfg(feature = "frame")]
impl<'a, S, F, T> DuplexSliceMut<'a, S, F> for T
where
    S: Sample,
    F: Frame<Sample = S>,
    T: DuplexSampleSliceMut<'a, S>
        + DuplexFrameSliceMut<'a, F>,
{
}

impl<S, T> DuplexBoxedSampleSlice<S> for T
where
    S: Sample,
    T: FromBoxedSampleSlice<S> + ToBoxedSampleSlice<S>,
{
}

#[cfg(feature = "frame")]
impl<F, T> DuplexBoxedFrameSlice<F> for T
where
    F: Frame,
    T: FromBoxedFrameSlice<F> + ToBoxedFrameSlice<F>,
{
}

#[cfg(feature = "frame")]
impl<S, F, T> DuplexBoxedSlice<S, F> for T
where
    S: Sample,
    F: Frame<Sample = S>,
    T: DuplexBoxedSampleSlice<S> + DuplexBoxedFrameSlice<F>,
{
}
