#![allow(dead_code)]

pub mod f32 {
    #[cfg(feature = "std")]
    pub fn powf32(a: f32, b: f32) -> f32 {
        a.powf(b)
    }
    #[cfg(not(feature = "std"))]
    pub fn powf32(a: f32, b: f32) -> f32 {
        unsafe { core::intrinsics::powf32(a, b) }
    }
}
