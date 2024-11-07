use crate::{Buffer, Input, Node};
use dasp_frame::Frame;
use dasp_signal::Signal;

impl<F, W> Node<W> for dyn Signal<Frame = F>
where
    F: Frame<Sample = f32>,
{
    fn process(&mut self, _inputs: &[Input<W>], output: &mut [Buffer]) {
        let channels = std::cmp::min(F::CHANNELS, output.len());
        for ix in 0..Buffer::LEN {
            let frame = self.next();
            for ch in 0..channels {
                // Safe, as we verify the number of channels at the beginning of the function.
                output[ch][ix] = unsafe { *frame.channel_unchecked(ch) };
            }
        }
    }
}
