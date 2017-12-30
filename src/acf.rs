use {ring_buffer, Frame, Sample};

/// Iteratively applies the ACF (auto-correlation function) over a signal of sample `Frame`s.
///
/// The `current` method returns a slice of float `Frame`s. Each frame in the acf represents how
/// strongly correlated the signal is with itself at each sample offset, over the given window size.
///
/// In order to properly calculate all of the auto-correlation values, the `next` method must be
/// called with at least `window_size() + max_t` frames. Prior to that, the output acf values are
/// undefined.
pub trait Acf<F> where F: Frame {

    /// Updates the calculated acf with `new_frame`. Returns the updated acf values. The `current`
    /// method will return this same slice after the call to `next` completes.
    fn next(&mut self, new_frame: F) -> &[F::Float];

    /// The current values calculated using the ACF. This is a slice of float frames, with a length
    /// equal to the value returned by the `max_t` method.
    fn current(&self) -> &[F::Float];

    /// The size of the window used to calculate the ACF.
    fn window_size(&self) -> usize;

    /// The number of frame offsets (where each offset is referred to as "t") for which to apply
    /// the ACF. The acf slice returned by the `current` and `next` methods will be max_t() + 1 in
    /// size.
    fn max_t(&self) -> usize;
}

/// An implementation of the Acf trait that uses the "diff squared" to calculate correlation.
///
/// The typical auto-correlation algorithm is used, but instead of multiplying samples together
/// and summing over the window at each T offset, the difference between the samples is squared
/// over the window for each T offset, and those are summed together.
#[derive(Clone)]
pub struct DiffSquaredAcf<F>
where
    F: Frame,
{
    window_size: usize,
    max_t: usize,
    frame_buffer: ring_buffer::Fixed<Vec<F::Float>>,
    acf: Vec<F::Float>,
}

impl<F> DiffSquaredAcf<F>
where
    F: Frame,
{
    /// Construct a new ***DiffSquaredAcf*** with the given window_size and max_t.
    ///
    /// ```
    /// extern crate sample;
    ///
    /// use sample::acf::{Acf, DiffSquaredAcf};
    ///
    /// fn main() {
    ///     let acf = DiffSquaredAcf::<[f32; 1]>::new(2, 3);
    ///     assert_eq!(acf.current(), [[0.0], [0.0], [0.0], [0.0]]);
    /// }
    /// ```
    pub fn new(window_size: usize, max_t: usize) -> Self {
        DiffSquaredAcf {
            window_size: window_size,
            max_t: max_t,
            frame_buffer: ring_buffer::Fixed::from(vec![
                F::Float::equilibrium();
                // Add room for 1 extra frame, to simplifity the logic in the `next` method.
                // The frame that gets pushed off the end of max_t's window will still be in the
                // frame_buffer, so it can be indexed there instead of having to have special logic
                // to handle that value being returned from frame_buffer.push().
                window_size + max_t + 1
            ]),
            acf: vec![F::Float::equilibrium(); max_t + 1],
        }
    }

    /// Returns the index of the given frame in frame_buffer.
    #[inline]
    fn frame_index(&self, index: usize) -> usize {
        // The ring buffer appends to the end, and logically shifts left. The most recent frame is
        // the last item in the ring buffer, and the signal is stored in reverse.
        self.frame_buffer.len() - 1 - index
    }

    #[inline]
    fn get_frame(&self, index: usize) -> &F::Float {
        self.frame_buffer.get(self.frame_index(index))
    }
}

impl<F> Acf<F> for DiffSquaredAcf<F>
where
    F: Frame,
{
    /// Integrates `new_frame` into the acf.
    ///
    /// Returns a borrowed reference to the current ACF after the frame is applied.
    ///
    /// ```
    /// extern crate sample;
    ///
    /// use sample::acf::{Acf, DiffSquaredAcf};
    ///
    /// fn main() {
    ///     let mut acf = DiffSquaredAcf::<[f32; 1]>::new(2, 3);
    ///     acf.next([1.0]);
    ///     acf.next([2.0]);
    ///     acf.next([3.0]);
    ///     acf.next([4.0]);
    ///     assert_eq!(acf.next([5.0]),  [[0.0], [2.0], [8.0], [18.0]]);
    ///     assert_eq!(acf.next([4.0]),  [[0.0], [2.0], [4.0], [10.0]]);
    ///     assert_eq!(acf.next([3.0]),  [[0.0], [2.0], [4.0], [2.0]]);
    ///     assert_eq!(acf.next([2.0]),  [[0.0], [2.0], [8.0], [10.0]]);
    ///     assert_eq!(acf.next([1.0]),  [[0.0], [2.0], [8.0], [18.0]]);
    /// }
    /// ```
    fn next(&mut self, new_frame: F) -> &[F::Float] {
        let diff_squared_fn = &diff_squared::<<F::Float as Frame>::Sample>;
        self.frame_buffer.push(new_frame.to_float_frame());
        for t in 1..(self.max_t + 1) {
            let new_diff_squared: F::Float = self.get_frame(t)
                .zip_map(*self.get_frame(0), diff_squared_fn);
            let old_diff_squared: F::Float = self.get_frame(t + self.window_size)
                .zip_map(*self.get_frame(self.window_size), diff_squared_fn);
            self.acf[t] = self.acf[t].add_amp(new_diff_squared)
                .zip_map(old_diff_squared, |acf_sample, old_diff_squared_sample| {
                    let diff = acf_sample - old_diff_squared_sample;
                    // Don't let floating point rounding errors put us below 0.0.
                    if diff < <F::Sample as Sample>::Float::equilibrium() {
                        <F::Sample as Sample>::Float::equilibrium()
                    } else {
                        diff
                    }
                });
        }
        &self.acf
    }

    /// Returns a borrowed reference to the current acf, without modifying it.
    ///
    /// ```
    /// extern crate sample;
    ///
    /// use sample::acf::{Acf, DiffSquaredAcf};
    ///
    /// fn main() {
    ///     let mut acf = DiffSquaredAcf::<[f32; 2]>::new(2, 3);
    ///     acf.next([1.0, 2.0]);
    ///     acf.next([2.0, 3.0]);
    ///     acf.next([3.0, 4.0]);
    ///     acf.next([4.0, 5.0]);
    ///     acf.next([5.0, 4.0]);
    ///     assert_eq!(acf.current(),  [[0.0, 0.0], [2.0, 2.0], [8.0, 4.0], [18.0, 10.0]]);
    ///     acf.next([4.0, 3.0]);
    ///     assert_eq!(acf.current(),  [[0.0, 0.0], [2.0, 2.0], [4.0, 4.0], [10.0, 2.0]]);
    ///     acf.next([3.0, 2.0]);
    ///     assert_eq!(acf.current(),  [[0.0, 0.0], [2.0, 2.0], [4.0, 8.0], [2.0, 10.0]]);
    ///     acf.next([2.0, 1.0]);
    ///     assert_eq!(acf.current(),  [[0.0, 0.0], [2.0, 2.0], [8.0, 8.0], [10.0, 18.0]]);
    ///     acf.next([1.0, 2.0]);
    ///     assert_eq!(acf.current(),  [[0.0, 0.0], [2.0, 2.0], [8.0, 4.0], [18.0, 10.0]]);
    /// }
    /// ```
    #[inline]
    fn current(&self) -> &[F::Float] {
        &self.acf
    }

    /// Returns the window_size used for the ACF calculation.
    ///
    /// ```
    /// extern crate sample;
    ///
    /// use sample::acf::{Acf, DiffSquaredAcf};
    ///
    /// fn main() {
    ///     let mut acf = DiffSquaredAcf::<[f32; 1]>::new(2, 3);
    ///     assert_eq!(acf.window_size(), 2);
    /// }
    /// ```
    #[inline]
    fn window_size(&self) -> usize {
        self.window_size
    }

    /// Returns the max sample offset (referred to as "T") for which to calculate the ACF.
    /// The acf slice returned by the `current` and `next` methods will be max_t() + 1 in size.
    ///
    /// ```
    /// extern crate sample;
    ///
    /// use sample::acf::{Acf, DiffSquaredAcf};
    ///
    /// fn main() {
    ///     let mut acf = DiffSquaredAcf::<[f32; 1]>::new(2, 3);
    ///     assert_eq!(acf.max_t(), 3);
    /// }
    /// ```
    #[inline]
    fn max_t(&self) -> usize {
        self.max_t
    }
}

/// Returns the difference between left and right, squared.
fn diff_squared<S>(left: S::Float, right: S::Float) -> S::Float
where
    S: Sample,
{
    let diff = left - right;
    diff * diff
}
