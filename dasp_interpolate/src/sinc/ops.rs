#![allow(dead_code)]

pub mod f64 {
    #[cfg(not(feature = "std"))]
    pub fn sin(x: f64) -> f64 {
        core::intrinsics::sin(x)
    }
    #[cfg(feature = "std")]
    pub fn sin(x: f64) -> f64 {
        x.sin()
    }

    #[cfg(not(feature = "std"))]
    pub fn cos(x: f64) -> f64 {
        core::intrinsics::cos(x)
    }
    #[cfg(feature = "std")]
    pub fn cos(x: f64) -> f64 {
        x.cos()
    }
}
