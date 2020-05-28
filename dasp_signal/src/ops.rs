pub mod f64 {
    #[allow(unused_imports)]
    use core;

    #[cfg(not(feature = "std"))]
    pub fn floor(x: f64) -> f64 {
        unsafe { core::intrinsics::floorf64(x) }
    }
    #[cfg(feature = "std")]
    pub fn floor(x: f64) -> f64 {
        x.floor()
    }

    #[cfg(not(feature = "std"))]
    #[allow(dead_code)]
    pub fn ceil(x: f64) -> f64 {
        unsafe { core::intrinsics::ceilf64(x) }
    }
    #[cfg(feature = "std")]
    #[allow(dead_code)]
    pub fn ceil(x: f64) -> f64 {
        x.ceil()
    }

    #[cfg(not(feature = "std"))]
    pub fn sin(x: f64) -> f64 {
        unsafe { core::intrinsics::sinf64(x) }
    }
    #[cfg(feature = "std")]
    pub fn sin(x: f64) -> f64 {
        x.sin()
    }
}
