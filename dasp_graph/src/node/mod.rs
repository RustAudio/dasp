use crate::buffer::Buffer;
use core::fmt;

#[cfg(feature = "node-boxed")]
pub use boxed::{BoxedNode, BoxedNodeSend};
#[cfg(feature = "node-delay")]
pub use delay::Delay;
#[cfg(feature = "node-graph")]
pub use graph::GraphNode;
#[cfg(feature = "node-pass")]
pub use pass::Pass;
#[cfg(feature = "node-sum")]
pub use sum::{Sum, SumBuffers};

#[cfg(feature = "node-boxed")]
mod boxed;
#[cfg(feature = "node-delay")]
mod delay;
#[cfg(feature = "node-graph")]
mod graph;
#[cfg(feature = "node-pass")]
mod pass;
#[cfg(feature = "node-signal")]
mod signal;
#[cfg(feature = "node-sum")]
mod sum;

/// The `Node` type used within a dasp graph must implement this trait.
///
/// The implementation describes how audio is processed from its inputs to outputs.
///
/// - Audio **sources** or **inputs** may simply ignore the `inputs` field and write their source
///   data directly to the `output` buffers.
/// - Audio **processors**, **effects** or **sinks** may read from their `inputs`, apply some
///   custom processing and write the result to their `output` buffers.
///
/// Multiple `Node` implementations are provided and can be enabled or disabled via [their
/// associated features](../index.html#optional-features).
///
/// # Example
///
/// The following demonstrates how to implement a simple node that sums each of its inputs onto the
/// output.
///
/// ```rust
/// use dasp_graph::{Buffer, Input, Node};
///
/// // Our new `Node` type.
/// pub struct Sum;
///
/// // Implement the `Node` trait for our new type.
/// # #[cfg(feature = "dasp_slice")]
/// impl Node for Sum {
///     fn process(&mut self, inputs: &[Input], output: &mut [Buffer]) {
///         // Fill the output with silence.
///         for out_buffer in output.iter_mut() {
///             out_buffer.silence();
///         }
///         // Sum the inputs onto the output.
///         for (channel, out_buffer) in output.iter_mut().enumerate() {
///             for input in inputs {
///                 let in_buffers = input.buffers();
///                 if let Some(in_buffer) = in_buffers.get(channel) {
///                     dasp_slice::add_in_place(out_buffer, in_buffer);
///                 }
///             }
///         }
///     }
/// }
/// ```
pub trait Node<W = ()> {
    /// Process some audio given a list of the node's `inputs` and write the result to the `output`
    /// buffers.
    ///
    /// `inputs` represents a list of all nodes with direct edges toward this node. Each
    /// [`Input`](./struct.Input.html) within the list can providee a reference to the output
    /// buffers of their corresponding node.
    ///
    /// The `inputs` may be ignored if the implementation is for a source node. Alternatively, if
    /// the `Node` only supports a specific number of `input`s, it is up to the user to decide how
    /// they wish to enforce this or provide feedback at the time of graph and edge creation.
    ///
    /// This `process` method is called by the [`Processor`](../struct.Processor.html) as it
    /// traverses the graph during audio rendering.
    fn process(&mut self, inputs: &[Input<W>], output: &mut [Buffer]);
}

/// A reference to another node that is an input to the current node.
///
/// *TODO: It may be useful to provide some information that can uniquely identify the input node.
/// This could be useful to allow to distinguish between side-chained and regular inputs for
/// example.*
pub struct Input<W = ()> {
    buffers_ptr: *const Buffer,
    buffers_len: usize,
    edge_weight: W,
}

impl<W> Input<W> {
    // Constructor solely for use within the graph `process` function.
    pub(crate) fn new(slice: &[Buffer], edge_weight: W) -> Input<W> {
        let buffers_ptr = slice.as_ptr();
        let buffers_len = slice.len();
        Input {
            buffers_ptr,
            buffers_len,
            edge_weight,
        }
    }

    /// A reference to the buffers of the input node.
    pub fn buffers(&self) -> &[Buffer] {
        // As we know that an `Input` can only be constructed during a call to the graph `process`
        // function, we can be sure that our slice is still valid as long as the input itself is
        // alive.
        unsafe { std::slice::from_raw_parts(self.buffers_ptr, self.buffers_len) }
    }

    /// A reference to the input's edge weight.
    pub fn edge_weight(&self) -> &W {
        &self.edge_weight
    }
}

// Inputs can only be created by the `dasp_graph::process` implementation and only ever live as
// long as the lifetime of the call to the function. Thus, it's safe to implement this so that
// `Send` closures can be stored within the graph and sent between threads.
unsafe impl<W> Send for Input<W> {}

impl<W> fmt::Debug for Input<W> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.buffers(), f)
    }
}

impl<'a, T, W> Node<W> for &'a mut T
where
    T: Node<W> + ?Sized,
{
    fn process(&mut self, inputs: &[Input<W>], output: &mut [Buffer]) {
        (**self).process(inputs, output)
    }
}

impl<T, W> Node<W> for Box<T>
where
    T: Node<W> + ?Sized,
{
    fn process(&mut self, inputs: &[Input<W>], output: &mut [Buffer]) {
        (**self).process(inputs, output)
    }
}

impl<W> Node<W> for dyn Fn(&[Input<W>], &mut [Buffer]) {
    fn process(&mut self, inputs: &[Input<W>], output: &mut [Buffer]) {
        (*self)(inputs, output)
    }
}

impl<W> Node<W> for dyn FnMut(&[Input<W>], &mut [Buffer]) {
    fn process(&mut self, inputs: &[Input<W>], output: &mut [Buffer]) {
        (*self)(inputs, output)
    }
}

impl<W> Node<W> for fn(&[Input<W>], &mut [Buffer]) {
    fn process(&mut self, inputs: &[Input<W>], output: &mut [Buffer]) {
        (*self)(inputs, output)
    }
}
