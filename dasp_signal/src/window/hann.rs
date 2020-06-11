use super::{Window, Windower};
use dasp_frame::Frame;
use dasp_window::Hann;

impl<'a, F> Windower<'a, F, Hann>
where
    F: 'a + Frame,
{
    /// Constructor for a `Windower` using the `Hann` window function.
    ///
    /// ### Required Features
    ///
    /// - When using `dasp_signal`, this item requires the **window-hann** feature to be enabled.
    /// - When using `dasp`, this item requires the **signal-window-hann** feature to be enabled.
    pub fn hann(frames: &'a [F], bin: usize, hop: usize) -> Self {
        Windower::new(frames, bin, hop)
    }
}

/// A helper function for constructing a `Window` that uses a `Hann` `Type` function.
///
/// ### Required Features
///
/// - When using `dasp_signal`, this item requires the **window-hann** feature to be enabled.
/// - When using `dasp`, this item requires the **signal-window-hann** feature to be enabled.
pub fn hann<F>(num_frames: usize) -> Window<F, Hann>
where
    F: Frame,
{
    Window::new(num_frames)
}
