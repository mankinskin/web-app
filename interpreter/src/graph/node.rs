use petgraph::{
    *,
    graph::*,
    graphmap::*,
    dot::*,
    visit::*,
};

use crate::graph::edges::*;
use crate::graph::*;

#[derive(Debug, Clone)]
pub struct GraphNode<'a>  {
    graph: &'a TextGraph,
    index: NodeIndex,
}
impl<'a> std::ops::Deref for GraphNode<'a> {
    type Target = TextElement;
    fn deref(&self) -> &Self::Target {
        &self.graph[self.index]
    }
}
impl<'a> From<&'a GraphNode<'a>> for &'a TextElement {
    fn from(n: &'a GraphNode<'a>) -> Self {
        &n
    }
}
impl<'a> Into<NodeIndex> for GraphNode<'a> {
    fn into(self) -> NodeIndex {
        self.index
    }
}


impl<'a> GraphNode<'a> {
    pub fn new(graph: &'a TextGraph, index: NodeIndex) -> Self {
        GraphNode {
            graph,
            index,
        }
    }
    pub fn info(&'a self) {
        let mut nodes = GraphNodes::new(self.graph);
        nodes.add(self.index);

        let mut incoming_edges = self.incoming_edges();
        incoming_edges.sort_by_weight();
        let max_distance = incoming_edges.max_weight().unwrap_or(0);
        let node_weight = self.weight();
        println!("Node: {}", node_weight);
        println!("max_distance: {}", max_distance);

        let weight_groups = incoming_edges.group_by_weight();
        let node_groups: Vec<Vec<GraphNode<'a>>> = weight_groups.iter().map(|group|
            group.iter().map(|edge| self.graph.get_node(edge.source())).collect()
        ).collect();

        println!("---");
        for (distance, group) in node_groups.iter().enumerate() {
            // for each distance group 1..max

            println!("Distance {}, {} Nodes.", distance + 1, group.len());

            for node in group.iter() {
                // for each node at distance
                let mut nincoming_edges = node.incoming_edges();
                nincoming_edges.sort_by_weight();
                let nnode_weight = node.weight();

                let nweight_groups = nincoming_edges.group_by_weight();
                let mut nnode_groups: Vec<Vec<GraphNode<'a>>> = nweight_groups.iter().map(|group|
                    group.iter().map(|edge| self.graph.get_node(edge.source())).collect()
                ).collect();
            }

        }
        println!("---");
    }

    pub fn is_at_distance(&'a self, other: GraphNode<'a>, distance: usize) -> bool {
        self.graph
            .find_edge(self.index, other.into())
            .map(|e| self.graph.edge_weight(e).map(|w| w.contains(&distance)).unwrap())
            .unwrap_or(false)
    }

    pub fn weight(&'a self) -> &'a TextElement {
        <&TextElement>::from(self)
    }
    pub fn edges_directed(&self, d: Direction) -> GraphEdges<'a> {
        self.graph.edges_directed(self.index, d).into()
    }
    pub fn incoming_edges(&self) -> GraphEdges<'a> {
        self.edges_directed(Direction::Incoming)
    }
    pub fn outgoing_edges(&self) -> GraphEdges<'a> {
        self.edges_directed(Direction::Outgoing)
    }
}
