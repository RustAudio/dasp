use core::fmt;
use core::ops::{Deref, DerefMut};

/// The fixed-size buffer used for processing the graph.
#[derive(Clone)]
pub struct Buffer {
    data: [f32; Self::LEN],
}

impl Buffer {
    /// The fixed length of the **Buffer** type.
    pub const LEN: usize = 64;
    /// A silent **Buffer**.
    pub const SILENT: Self = Buffer {
        data: [0.0; Self::LEN],
    };

    /// Short-hand for writing silence to the whole buffer.
    pub fn silence(&mut self) {
        self.data.copy_from_slice(&Self::SILENT)
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::SILENT
    }
}

impl From<[f32; Self::LEN]> for Buffer {
    fn from(data: [f32; Self::LEN]) -> Self {
        Buffer { data }
    }
}

impl fmt::Debug for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.data[..], f)
    }
}

impl PartialEq for Buffer {
    fn eq(&self, other: &Self) -> bool {
        self[..] == other[..]
    }
}

impl Deref for Buffer {
    type Target = [f32];
    fn deref(&self) -> &Self::Target {
        &self.data[..]
    }
}

impl DerefMut for Buffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data[..]
    }
}
