pub mod f32 {
    /// Newton-Raphson square root implementation for f32.
    /// Uses bit manipulation for initial guess, then 3 iterations for ~6-7 decimal places.
    /// Accuracy: ~6-7 decimal places
    #[cfg(not(feature = "std"))]
    #[inline]
    pub fn sqrt(x: f32) -> f32 {
        if x < 0.0 {
            return f32::NAN;
        }
        if x == 0.0 {
            return x; // preserves +0.0 and -0.0
        }

        // Initial guess from bit manipulation: halve exponent, shift mantissa
        let bits = x.to_bits();
        let exp = (bits >> 23) & 0xff;
        let mant = bits & 0x7fffff;

        let unbiased = exp as i32 - 127;
        let sqrt_exp = (unbiased / 2 + 127) as u32;
        let guess_bits = (sqrt_exp << 23) | (mant >> 1);
        let mut guess = f32::from_bits(guess_bits);

        for _ in 0..3 {
            guess = 0.5 * (guess + x / guess);
        }
        guess
    }
    #[cfg(feature = "std")]
    #[inline]
    pub fn sqrt(x: f32) -> f32 {
        x.sqrt()
    }

    #[cfg(not(feature = "std"))]
    #[inline]
    pub fn round(x: f32) -> f32 {
        // Branchless rounding: copysign gives +0.5 for positive x, -0.5 for negative x
        // This shifts the value toward zero before truncation, achieving proper rounding
        (x + 0.5_f32.copysign(x)) as i64 as f32
    }
    #[cfg(feature = "std")]
    #[inline]
    pub fn round(x: f32) -> f32 {
        x.round()
    }
}

pub mod f64 {
    /// Newton-Raphson square root implementation for f64.
    /// Uses bit manipulation for initial guess, then 4 iterations for ~14-15 decimal places.
    /// Accuracy: ~14-15 decimal places
    #[cfg(not(feature = "std"))]
    #[inline]
    pub fn sqrt(x: f64) -> f64 {
        if x < 0.0 {
            return f64::NAN;
        }
        if x == 0.0 {
            return x; // preserves +0.0 and -0.0
        }

        // Initial guess from bit manipulation: halve exponent, shift mantissa
        let bits = x.to_bits();
        let exp = (bits >> 52) & 0x7ff;
        let mant = bits & 0x000f_ffff_ffff_ffff;

        let unbiased = exp as i32 - 1023;
        let sqrt_exp = (unbiased / 2 + 1023) as u64;
        let guess_bits = (sqrt_exp << 52) | (mant >> 1);
        let mut guess = f64::from_bits(guess_bits);

        for _ in 0..4 {
            guess = 0.5 * (guess + x / guess);
        }
        guess
    }
    #[cfg(feature = "std")]
    #[inline]
    pub fn sqrt(x: f64) -> f64 {
        x.sqrt()
    }

    #[cfg(not(feature = "std"))]
    #[inline]
    pub fn round(x: f64) -> f64 {
        // Branchless rounding: copysign gives +0.5 for positive x, -0.5 for negative x
        // This shifts the value toward zero before truncation, achieving proper rounding
        (x + 0.5_f64.copysign(x)) as i64 as f64
    }
    #[cfg(feature = "std")]
    #[inline]
    pub fn round(x: f64) -> f64 {
        x.round()
    }
}
