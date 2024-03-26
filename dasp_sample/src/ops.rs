pub mod f32 {
    #[allow(unused_imports)]
    use core;

    #[cfg(not(feature = "std"))]
    pub fn sqrt(x: f32) -> f32 {
        if x >= 0.0 {
            f32::from_bits((x.to_bits() + 0x3f80_0000) >> 1)
        } else {
            f32::NAN
        }
    }
    #[cfg(feature = "std")]
    pub fn sqrt(x: f32) -> f32 {
        x.sqrt()
    }
}

pub mod f64 {
    #[allow(unused_imports)]
    use core;

    #[cfg(not(feature = "std"))]
    pub fn sqrt(x: f64) -> f64 {
        if x >= 0.0 {
            f64::from_bits((x.to_bits() + 0x3f80_0000) >> 1)
        } else {
            f64::NAN
        }
    }
    #[cfg(feature = "std")]
    pub fn sqrt(x: f64) -> f64 {
        x.sqrt()
    }
}
