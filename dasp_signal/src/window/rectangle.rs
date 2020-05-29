use super::{Window, Windower};
use dasp_frame::Frame;
use dasp_window::Rectangle;

impl<'a, F> Windower<'a, F, Rectangle>
where
    F: 'a + Frame,
{
    /// Constructor for a `Windower` using the `Rectangle` window function.
    ///
    /// ### Required Features
    ///
    /// - When using `dasp_signal`, this item requires the **window-rectangle** feature to be enabled.
    /// - When using `dasp`, this item requires the **signal-window-rectangle** feature to be enabled.
    pub fn rectangle(frames: &'a [F], bin: usize, hop: usize) -> Self {
        Windower::new(frames, bin, hop)
    }
}

/// A helper function for constructing a `Window` that uses a `Rectangle` `Type` function.
///
/// ### Required Features
///
/// - When using `dasp_signal`, this item requires the **window-rectangle** feature to be enabled.
/// - When using `dasp`, this item requires the **signal-window-rectangle** feature to be enabled.
pub fn rectangle<F>(num_frames: usize) -> Window<F, Rectangle>
where
    F: Frame,
{
    Window::new(num_frames)
}
