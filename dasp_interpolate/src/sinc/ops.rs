#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(core_intrinsics))]
#![allow(dead_code)]

pub mod f64 {
    #[cfg(not(feature = "std"))]
    pub fn sin(x: f64) -> f64 {
        unsafe { core::intrinsics::sinf64(x) }
    }
    #[cfg(feature = "std")]
    pub fn sin(x: f64) -> f64 {
        x.sin()
    }

    #[cfg(not(feature = "std"))]
    pub fn cos(x: f64) -> f64 {
        unsafe { core::intrinsics::cosf64(x) }
    }
    #[cfg(feature = "std")]
    pub fn cos(x: f64) -> f64 {
        x.cos()
    }
}
