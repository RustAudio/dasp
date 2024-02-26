use crate::{Buffer, Input, Node};
use core::fmt;
use core::ops::{Deref, DerefMut};

/// A wrapper around a `Box<dyn Node>`.
///
/// Provides the necessary `Sized` implementation to allow for compatibility with the graph process
/// function.
pub struct BoxedNode<W = ()>(pub Box<dyn Node<W>>);

/// A wrapper around a `Box<dyn Node>`.
///
/// Provides the necessary `Sized` implementation to allow for compatibility with the graph process
/// function.
///
/// Useful when the ability to send nodes from one thread to another is required. E.g. this is
/// common when initialising nodes or the audio graph itself on one thread before sending them to
/// the audio thread.
pub struct BoxedNodeSend<W = ()>(pub Box<dyn Node<W> + Send>);

impl<W> BoxedNode<W> {
    /// Create a new `BoxedNode` around the given `node`.
    ///
    /// This is short-hand for `BoxedNode::from(Box::new(node))`.
    pub fn new<T>(node: T) -> Self
    where
        T: 'static + Node<W>,
    {
        Self::from(Box::new(node))
    }
}

impl<W> BoxedNodeSend<W> {
    /// Create a new `BoxedNode` around the given `node`.
    ///
    /// This is short-hand for `BoxedNode::from(Box::new(node))`.
    pub fn new<T>(node: T) -> Self
    where
        T: 'static + Node<W> + Send,
    {
        Self::from(Box::new(node))
    }
}

impl<W> Node<W> for BoxedNode<W> {
    fn process(&mut self, inputs: &[Input<W>], output: &mut [Buffer]) {
        self.0.process(inputs, output)
    }
}

impl<W> Node<W> for BoxedNodeSend<W> {
    fn process(&mut self, inputs: &[Input<W>], output: &mut [Buffer]) {
        self.0.process(inputs, output)
    }
}

impl<T, W> From<Box<T>> for BoxedNode<W>
where
    T: 'static + Node<W>,
{
    fn from(n: Box<T>) -> Self {
        BoxedNode(n as Box<dyn Node<W>>)
    }
}

impl<T, W> From<Box<T>> for BoxedNodeSend<W>
where
    T: 'static + Node<W> + Send,
{
    fn from(n: Box<T>) -> Self {
        BoxedNodeSend(n as Box<dyn Node<W> + Send>)
    }
}

impl<W> Into<Box<dyn Node<W>>> for BoxedNode<W> {
    fn into(self) -> Box<dyn Node<W>> {
        self.0
    }
}

impl<W> Into<Box<dyn Node<W> + Send>> for BoxedNodeSend<W> {
    fn into(self) -> Box<dyn Node<W> + Send> {
        self.0
    }
}

impl<W> fmt::Debug for BoxedNode<W> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BoxedNode").finish()
    }
}

impl<W> fmt::Debug for BoxedNodeSend<W> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BoxedNodeSend").finish()
    }
}

impl<W> Deref for BoxedNode<W> {
    type Target = Box<dyn Node<W>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<W> Deref for BoxedNodeSend<W> {
    type Target = Box<dyn Node<W> + Send>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<W> DerefMut for BoxedNode<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<W> DerefMut for BoxedNodeSend<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
