use super::{Window, Windower};
use dasp_frame::Frame;
use dasp_window::Hanning;

impl<'a, F> Windower<'a, F, Hanning>
where
    F: 'a + Frame,
{
    /// Constructor for a `Windower` using the `Hanning` window function.
    ///
    /// ### Required Features
    ///
    /// - When using `dasp_signal`, this item requires the **window-hanning** feature to be enabled.
    /// - When using `dasp`, this item requires the **signal-window-hanning** feature to be enabled.
    pub fn hanning(frames: &'a [F], bin: usize, hop: usize) -> Self {
        Windower::new(frames, bin, hop)
    }
}

/// A helper function for constructing a `Window` that uses a `Hanning` `Type` function.
///
/// ### Required Features
///
/// - When using `dasp_signal`, this item requires the **window-hanning** feature to be enabled.
/// - When using `dasp`, this item requires the **signal-window-hanning** feature to be enabled.
pub fn hanning<F>(num_frames: usize) -> Window<F, Hanning>
where
    F: Frame,
{
    Window::new(num_frames)
}
