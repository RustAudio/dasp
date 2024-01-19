//! Implementation of `Node` for a graph of nodes.
//!
//! Allows for nesting subgraphs within nodes of a graph.

use crate::{Buffer, Input, Node, NodeData, Processor};
use core::marker::PhantomData;
use petgraph::data::DataMapMut;
use petgraph::visit::{Data, GraphBase, IntoEdgesDirected, Visitable};

pub struct GraphNode<G, T>
where
    G: Visitable + Data,
{
    pub processor: Processor<G>,
    pub graph: G,
    pub input_nodes: Vec<G::NodeId>,
    pub output_node: G::NodeId,
    pub node_type: PhantomData<T>,
}

impl<G, T> Node for GraphNode<G, T>
where
    G: Data<NodeWeight = NodeData<T>> + DataMapMut + Visitable,
    for<'a> &'a G:
        GraphBase<NodeId = G::NodeId> + IntoEdgesDirected + Data<EdgeWeight = G::EdgeWeight>,
    G::EdgeWeight: Clone,
    T: Node<G::EdgeWeight>,
{
    fn process(&mut self, inputs: &[Input], output: &mut [Buffer]) {
        let GraphNode {
            ref mut processor,
            ref mut graph,
            ref input_nodes,
            output_node,
            ..
        } = *self;

        // Write the input buffers to the input nodes.
        for (input, &in_n) in inputs.iter().zip(input_nodes) {
            let in_node_bufs = &mut graph
                .node_weight_mut(in_n)
                .expect("no node for graph node's input node ID")
                .buffers;
            for (in_node_buf, in_buf) in in_node_bufs.iter_mut().zip(input.buffers()) {
                in_node_buf.copy_from_slice(in_buf);
            }
        }

        // Process the graph.
        processor.process(graph, output_node);

        // Write the output node buffers to the output buffers.
        let out_node_bufs = &mut graph
            .node_weight_mut(output_node)
            .expect("no node for graph node's output node ID")
            .buffers;
        for (out_buf, out_node_buf) in output.iter_mut().zip(out_node_bufs) {
            out_buf.copy_from_slice(out_node_buf);
        }
    }
}
