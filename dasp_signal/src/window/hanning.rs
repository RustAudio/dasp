use super::{Window, Windower};
use dasp_frame::Frame;
use dasp_window::hanning::Hanning;

impl<'a, F> Windower<'a, F, Hanning>
where
    F: 'a + Frame,
{
    /// Constructor for a `Windower` using the `Hanning` window function.
    pub fn hanning(frames: &'a [F], bin: usize, hop: usize) -> Self {
        Windower::new(frames, bin, hop)
    }
}

/// A helper function for constructing a `Window` that uses a `Hanning` `Type` function.
pub fn hanning<F>(num_frames: usize) -> Window<F, Hanning>
where
    F: Frame,
{
    Window::new(num_frames)
}
