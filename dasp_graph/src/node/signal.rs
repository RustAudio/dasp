use crate::{Buffer, Input, Node};
use dasp_frame::Frame;
use dasp_signal::Signal;

impl<F> Node for dyn Signal<Frame = F>
where
    F: Frame<Sample = f32>,
{
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer]) {
        for ix in 0..Buffer::LEN {
            let frame = self.next();
            for (ch, out) in output.iter_mut().enumerate().take(F::CHANNELS) {
                // Safe, as ch never exceeds min(F::CHANNELS, output.len()).
                out[ix] = unsafe { *frame.channel_unchecked(ch) };
            }
        }
    }
}
