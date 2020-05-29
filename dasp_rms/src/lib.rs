//! Root mean square calculation over a signal.
//!
//! The primary type of interest in this module is the [**Rms**](./struct.Rms).

#![cfg_attr(not(feature = "std"), no_std)]

use core::fmt;
use core::marker::PhantomData;
use dasp_frame::Frame;
use dasp_ring_buffer as ring_buffer;
use dasp_sample::{FloatSample, Sample};

/// Iteratively extracts the RMS (root mean square) envelope from a window over a signal of sample
/// `Frame`s.
#[derive(Clone)]
pub struct Rms<F, S>
where
    F: Frame,
    S: ring_buffer::Slice<Element = F::Float>,
{
    /// The type of `Frame`s for which the RMS will be calculated.
    frame: PhantomData<F>,
    /// The ringbuffer of frame sample squares (i.e. `sample * sample`) used to calculate the RMS
    /// per frame.
    ///
    /// When a new frame is received, the **Rms** pops the front sample_square and adds the new
    /// sample_square to the back.
    window: ring_buffer::Fixed<S>,
    /// The sum total of all sample_squares currently within the **Rms**'s `window` ring buffer.
    square_sum: F::Float,
}

impl<F, S> Rms<F, S>
where
    F: Frame,
    S: ring_buffer::Slice<Element = F::Float>,
{
    /// Construct a new **Rms** that uses the given ring buffer as its window.
    ///
    /// The window size of the **Rms** is equal to the length of the given ring buffer.
    ///
    /// ```
    /// use dasp_ring_buffer as ring_buffer;
    /// use dasp_rms::Rms;
    ///
    /// fn main() {
    ///     let window = ring_buffer::Fixed::from([[0.0]; 4]);
    ///     let mut rms = Rms::new(window);
    ///     rms.next([0.5]);
    /// }
    /// ```
    pub fn new(ring_buffer: ring_buffer::Fixed<S>) -> Self {
        Rms {
            frame: PhantomData,
            window: ring_buffer,
            square_sum: Frame::EQUILIBRIUM,
        }
    }

    /// Zeroes the square_sum and the buffer of the `window`.
    ///
    /// ```
    /// use dasp_ring_buffer as ring_buffer;
    /// use dasp_rms::Rms;
    ///
    /// fn main() {
    ///     let window = ring_buffer::Fixed::from([[0.0]; 4]);
    ///     let mut rms = Rms::new(window);
    ///     rms.next([0.6]);
    ///     rms.next([0.9]);
    ///     rms.reset();
    ///     assert_eq!(rms.current(), [0.0]);
    /// }
    /// ```
    pub fn reset(&mut self)
    where
        S: ring_buffer::SliceMut,
    {
        for sample_square in self.window.iter_mut() {
            *sample_square = Frame::EQUILIBRIUM;
        }
        self.square_sum = Frame::EQUILIBRIUM;
    }

    /// The length of the window as a number of frames.
    ///
    /// ```
    /// use dasp_ring_buffer as ring_buffer;
    /// use dasp_rms::Rms;
    ///
    /// fn main() {
    ///     let window = ring_buffer::Fixed::from([[0.0]; 4]);
    ///     let mut rms = Rms::new(window);
    ///     assert_eq!(rms.window_frames(), 4);
    ///     rms.next([0.5]);
    ///     assert_eq!(rms.window_frames(), 4);
    /// }
    /// ```
    #[inline]
    pub fn window_frames(&self) -> usize {
        self.window.len()
    }

    /// The next RMS given the new frame in the sequence.
    ///
    /// The **Rms** pops its front frame and adds the new frame to the back.
    ///
    /// The yielded RMS is the RMS of all frame squares in the `window` after the new frame is
    /// added.
    ///
    /// This method uses `Rms::next_squared` internally and then calculates the square root.
    ///
    /// ```
    /// use dasp_ring_buffer as ring_buffer;
    /// use dasp_rms::Rms;
    ///
    /// fn main() {
    ///     let window = ring_buffer::Fixed::from([[0.0]; 4]);
    ///     let mut rms = Rms::new(window);
    ///     assert_eq!(rms.next([1.0]), [0.5]);
    ///     assert_eq!(rms.next([-1.0]), [0.7071067811865476]);
    ///     assert_eq!(rms.next([1.0]), [0.8660254037844386]);
    ///     assert_eq!(rms.next([-1.0]), [1.0]);
    /// }
    /// ```
    #[inline]
    pub fn next(&mut self, new_frame: F) -> F::Float
    where
        S: ring_buffer::SliceMut,
    {
        self.next_squared(new_frame).map(|s| s.sample_sqrt())
    }

    /// The same as **Rms::next**, but does not calculate the final square root required to
    /// determine the RMS.
    #[inline]
    pub fn next_squared(&mut self, new_frame: F) -> F::Float
    where
        S: ring_buffer::SliceMut,
    {
        // Determine the square of the new frame.
        let new_frame_square = new_frame.to_float_frame().map(|s| s * s);
        // Push back the new frame_square.
        let removed_frame_square = self.window.push(new_frame_square);
        // Add the new frame square and subtract the removed frame square.
        self.square_sum =
            self.square_sum
                .add_amp(new_frame_square)
                .zip_map(removed_frame_square, |s, r| {
                    let diff = s - r;
                    // Don't let floating point rounding errors put us below 0.0.
                    if diff < Sample::EQUILIBRIUM {
                        Sample::EQUILIBRIUM
                    } else {
                        diff
                    }
                });
        self.calc_rms_squared()
    }

    /// Consumes the **Rms** and returns its inner ring buffer of squared frames along with a frame
    /// representing the sum of all frame squares contained within the ring buffer.
    pub fn into_parts(self) -> (ring_buffer::Fixed<S>, S::Element) {
        let Rms {
            window, square_sum, ..
        } = self;
        (window, square_sum)
    }

    /// Calculates the RMS of all frames currently stored within the inner window.
    ///
    /// ```
    /// use dasp_ring_buffer as ring_buffer;
    /// use dasp_rms::Rms;
    ///
    /// fn main() {
    ///     let window = ring_buffer::Fixed::from([[0.0]; 4]);
    ///     let mut rms = Rms::new(window);
    ///     assert_eq!(rms.current(), [0.0]);
    ///     rms.next([1.0]);
    ///     rms.next([1.0]);
    ///     rms.next([1.0]);
    ///     rms.next([1.0]);
    ///     assert_eq!(rms.current(), [1.0]);
    /// }
    /// ```
    pub fn current(&self) -> F::Float {
        self.calc_rms_squared().map(|s| s.sample_sqrt())
    }

    fn calc_rms_squared(&self) -> F::Float {
        let num_frames_f = Sample::from_sample(self.window.len() as f32);
        self.square_sum.map(|s| s / num_frames_f)
    }
}

impl<F, S> fmt::Debug for Rms<F, S>
where
    F: Frame,
    F::Float: fmt::Debug,
    S: fmt::Debug + ring_buffer::Slice<Element = F::Float>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("Rms")
            .field("frame", &self.frame)
            .field("window", &self.window)
            .field("square_sum", &self.square_sum)
            .finish()
    }
}
