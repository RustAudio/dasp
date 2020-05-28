use crate::{Rc, Signal};

#[cfg(not(feature = "std"))]
type BTreeMap<K, V> = alloc::collections::btree_map::BTreeMap<K, V>;
#[cfg(feature = "std")]
type BTreeMap<K, V> = std::collections::BTreeMap<K, V>;

#[cfg(not(feature = "std"))]
type VecDeque<T> = alloc::collections::vec_deque::VecDeque<T>;
#[cfg(feature = "std")]
type VecDeque<T> = std::collections::vec_deque::VecDeque<T>;

pub trait SignalBus: Signal {
    /// Moves the `Signal` into a `Bus` from which its output may be divided into multiple other
    /// `Signal`s in the form of `Output`s.
    ///
    /// This method allows to create more complex directed acyclic graph structures that
    /// incorporate concepts like sends, side-chaining, etc, rather than being restricted to tree
    /// structures where signals can only ever be joined but never divided.
    ///
    /// Note: When using multiple `Output`s in this fashion, you will need to be sure to pull the
    /// frames from each `Output` in sync (whether per frame or per buffer). This is because when
    /// output A requests `Frame`s before output B, those frames must remain available for output
    /// B and in turn must be stored in an intermediary ring buffer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use dasp_signal::{self as signal, Signal};
    /// use dasp_signal::bus::SignalBus;
    ///
    /// fn main() {
    ///     let frames = [[0.1], [0.2], [0.3], [0.4], [0.5], [0.6]];
    ///     let signal = signal::from_iter(frames.iter().cloned());
    ///     let bus = signal.bus();
    ///     let mut a = bus.send();
    ///     let mut b = bus.send();
    ///     assert_eq!(a.by_ref().take(3).collect::<Vec<_>>(), vec![[0.1], [0.2], [0.3]]);
    ///     assert_eq!(b.by_ref().take(3).collect::<Vec<_>>(), vec![[0.1], [0.2], [0.3]]);
    ///
    ///     let c = bus.send();
    ///     assert_eq!(c.take(3).collect::<Vec<_>>(), vec![[0.4], [0.5], [0.6]]);
    ///     assert_eq!(b.take(3).collect::<Vec<_>>(), vec![[0.4], [0.5], [0.6]]);
    ///     assert_eq!(a.take(3).collect::<Vec<_>>(), vec![[0.4], [0.5], [0.6]]);
    /// }
    /// ```
    fn bus(self) -> Bus<Self>
    where
        Self: Sized,
    {
        Bus::new(self, BTreeMap::new())
    }
}

/// The data shared between each `Output`.
struct SharedNode<S>
where
    S: Signal,
{
    signal: S,
    // The buffer of frames that have not yet been consumed by all outputs.
    buffer: VecDeque<S::Frame>,
    // The number of frames in `buffer` that have already been read for each output.
    frames_read: BTreeMap<usize, usize>,
    // The next output key.
    next_key: usize,
}

/// A type which allows for `send`ing a single `Signal` to multiple outputs.
///
/// This type manages
pub struct Bus<S>
where
    S: Signal,
{
    node: Rc<core::cell::RefCell<SharedNode<S>>>,
}

/// An output node to which some signal `S` is `Output`ing its frames.
///
/// It may be more accurate to say that the `Output` "pull"s frames from the signal.
pub struct Output<S>
where
    S: Signal,
{
    key: usize,
    node: Rc<core::cell::RefCell<SharedNode<S>>>,
}

impl<S> Bus<S>
where
    S: Signal,
{
    fn new(signal: S, frames_read: BTreeMap<usize, usize>) -> Self {
        Bus {
            node: Rc::new(core::cell::RefCell::new(SharedNode {
                signal: signal,
                buffer: VecDeque::new(),
                frames_read: frames_read,
                next_key: 0,
            })),
        }
    }

    /// Produce a new Output node to which the signal `S` will output its frames.
    #[inline]
    pub fn send(&self) -> Output<S> {
        let mut node = self.node.borrow_mut();

        // Get the key and increment for the next output.
        let key = node.next_key;
        node.next_key = node.next_key.wrapping_add(1);

        // Insert the number of frames read by the new output.
        let num_frames = node.buffer.len();
        node.frames_read.insert(key, num_frames);

        Output {
            key: key,
            node: self.node.clone(),
        }
    }
}

impl<S> SharedNode<S>
where
    S: Signal,
{
    // Requests the next frame for the `Output` at the given key.
    //
    // If there are no frames pending for the output, a new frame will be requested from the
    // signal and appended to the ring buffer to be received by the other outputs.
    fn next_frame(&mut self, key: usize) -> S::Frame {
        let num_frames = self.buffer.len();
        let frames_read = self.frames_read.remove(&key).expect(
            "no frames_read for Output",
        );

        let frame = if frames_read < num_frames {
            self.buffer[frames_read]
        } else {
            let frame = self.signal.next();
            self.buffer.push_back(frame);
            frame
        };

        // If the number of frames read by this output is the lowest, then we can pop the frame
        // from the front.
        let least_frames_read = !self.frames_read.values().any(|&other_frames_read| {
            other_frames_read <= frames_read
        });

        // If this output had read the least number of frames, pop the front frame and decrement
        // the frames read counters for each of the other outputs.
        let new_frames_read = if least_frames_read {
            self.buffer.pop_front();
            for other_frames_read in self.frames_read.values_mut() {
                *other_frames_read -= 1;
            }
            frames_read
        } else {
            frames_read + 1
        };

        self.frames_read.insert(key, new_frames_read);

        frame
    }

    #[inline]
    fn pending_frames(&self, key: usize) -> usize {
        self.buffer.len() - self.frames_read[&key]
    }

    // Drop the given output from the `Bus`.
    //
    // Called by the `Output::drop` implementation.
    fn drop_output(&mut self, key: usize) {
        self.frames_read.remove(&key);
        let least_frames_read = self.frames_read.values().fold(self.buffer.len(), |a, &b| {
            core::cmp::min(a, b)
        });
        if least_frames_read > 0 {
            for frames_read in self.frames_read.values_mut() {
                *frames_read -= least_frames_read;
            }
            for _ in 0..least_frames_read {
                self.buffer.pop_front();
            }
        }
    }
}

impl<S> Output<S>
where
    S: Signal,
{
    /// The number of frames that have been requested from the `Signal` `S` by some other `Output`
    /// that have not yet been requested by this `Output`.
    ///
    /// This is useful when using an `Output` to "monitor" some signal, allowing the user to drain
    /// only frames that have already been requested by some other `Output`.
    ///
    /// # Example
    ///
    /// ```
    /// use dasp_signal::{self as signal, Signal};
    /// use dasp_signal::bus::SignalBus;
    ///
    /// fn main() {
    ///     let frames = [[0.1], [0.2], [0.3]];
    ///     let bus = signal::from_iter(frames.iter().cloned()).bus();
    ///     let signal = bus.send();
    ///     let mut monitor = bus.send();
    ///     assert_eq!(signal.take(3).collect::<Vec<_>>(), vec![[0.1], [0.2], [0.3]]);
    ///     assert_eq!(monitor.pending_frames(), 3);
    ///     assert_eq!(monitor.next(), [0.1]);
    ///     assert_eq!(monitor.pending_frames(), 2);
    /// }
    /// ```
    #[inline]
    pub fn pending_frames(&self) -> usize {
        self.node.borrow().pending_frames(self.key)
    }
}

impl<T> SignalBus for T where T: Signal {}

impl<S> Signal for Output<S>
where
    S: Signal,
{
    type Frame = S::Frame;

    #[inline]
    fn next(&mut self) -> Self::Frame {
        self.node.borrow_mut().next_frame(self.key)
    }

    #[inline]
    fn is_exhausted(&self) -> bool {
        let node = self.node.borrow();
        node.pending_frames(self.key) == 0 && node.signal.is_exhausted()
    }
}

impl<S> Drop for Output<S>
where
    S: Signal,
{
    fn drop(&mut self) {
        self.node.borrow_mut().drop_output(self.key)
    }
}
