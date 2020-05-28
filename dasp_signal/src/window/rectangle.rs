use super::{Window, Windower};
use dasp_frame::Frame;
use dasp_window::rectangle::Rectangle;

impl<'a, F> Windower<'a, F, Rectangle>
where
    F: 'a + Frame,
{
    /// Constructor for a `Windower` using the `Rectangle` window function.
    pub fn rectangle(frames: &'a [F], bin: usize, hop: usize) -> Self {
        Windower::new(frames, bin, hop)
    }
}

/// A helper function for constructing a `Window` that uses a `Rectangle` `Type` function.
pub fn rectangle<F>(num_frames: usize) -> Window<F, Rectangle>
where
    F: Frame,
{
    Window::new(num_frames)
}
