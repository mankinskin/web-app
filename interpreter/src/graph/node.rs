use petgraph::{
    *,
    graph::*,
    graphmap::*,
    dot::*,
    visit::*,
};

use crate::graph::edges::*;
use crate::graph::*;

#[derive(Clone)]
pub struct GraphNode<'a>  {
    graph: &'a TextGraph,
    index: NodeIndex,
}
impl<'a> PartialEq for GraphNode<'a> {
    fn eq(&self, other: &GraphNode<'a>) -> bool {
        self.index == other.index &&
            self.graph as *const _ == other.graph
    }
}
impl<'a> Eq for GraphNode<'a> {}

impl<'a> std::hash::Hash for GraphNode<'a> {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        self.index.hash(h);
        (self.graph as *const TextGraph).hash(h);
    }
}

impl<'a> std::ops::Deref for GraphNode<'a> {
    type Target = TextGraphNodeWeight;
    fn deref(&self) -> &Self::Target {
        &self.graph[self.index]
    }
}
impl<'a> From<&'a GraphNode<'a>> for &'a TextElement {
    fn from(n: &'a GraphNode<'a>) -> Self {
        n.element()
    }
}
impl<'a> Into<NodeIndex> for GraphNode<'a> {
    fn into(self) -> NodeIndex {
        self.index
    }
}
use std::fmt::{self, Debug, Display, Formatter};
impl<'a> Debug for GraphNode<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "GraphNode {{ {:?} }}", self.index)
    }
}
impl<'a> Display for GraphNode<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let node_weight = self.weight();
        writeln!(f, "## {}", node_weight.element());

        /// Incoming
        let incoming_edges = self.edges_incoming();
        let max_incoming_distance = incoming_edges.max_weight().unwrap_or(0);
        let incoming_weight_groups = incoming_edges.clone().group_by_weight();
        let incoming_groups_counts: Vec<usize> = incoming_weight_groups.iter().map(|group|
            group.iter().count()
        ).collect();
        let incoming_node_groups: Vec<Vec<_>> = incoming_weight_groups.iter().map(|group|
            group.iter().map(|edge| self.graph.get_node(edge.source())).collect::<Vec<_>>()
        ).collect();
        writeln!(f, "Incoming edges:\n\
            \tcount: {},\n\
            \tmax distance: {},\n\
            \tgroup counts: {:?}",
            incoming_edges.into_iter().count(),
            max_incoming_distance,
            incoming_groups_counts);


        /// Outgoing
        let outgoing_edges = self.edges_outgoing();
        let max_outgoing_distance = outgoing_edges.max_weight().unwrap_or(0);

        let outgoing_weight_groups = outgoing_edges.clone().group_by_weight();
        let outgoing_groups_counts: Vec<usize> = outgoing_weight_groups.iter().map(|group|
            group.iter().count()
        ).collect();
        let outgoing_node_groups: Vec<Vec<_>> = outgoing_weight_groups.iter().map(|group|
            group.iter().map(|edge| self.graph.get_node(edge.target())).collect()
        ).collect();
        writeln!(f, "Outgoing edges:\n\
            \tcount: {},\n\
            \tmax distance: {},\n\
            \tgroup counts: {:?}",
            outgoing_edges.into_iter().count(),
            max_outgoing_distance,
            outgoing_groups_counts)
    }
}

impl<'a> GraphNode<'a> {
    pub fn new(graph: &'a TextGraph, index: NodeIndex) -> Self {
        GraphNode {
            graph,
            index,
        }
    }
    pub fn weight(&'a self) -> &'a TextGraphNodeWeight {
        <&TextGraphNodeWeight>::from(self)
    }
    pub fn index(&'a self) -> NodeIndex {
        self.index
    }
    pub fn is_at_distance(&'a self, other: &GraphNode<'a>, distance: usize) -> bool {
        self.graph
            .find_edge(self, other)
            .map(|e| self.graph.edge_weight(*e.index()).map(|w| w.contains(&distance)).unwrap())
            .unwrap_or(false)
    }

    pub fn edges(&'a self) -> GraphEdges<'a> {
        self.graph.get_edges(self.index)
    }
    pub fn edges_directed(&'a self, d: Direction) -> GraphEdges<'a> {
        self.graph.get_edges_directed(self.index, d)
    }
    pub fn edges_incoming(&'a self) -> GraphEdges<'a> {
        self.edges_directed(Direction::Incoming)
    }
    pub fn edges_outgoing(&'a self) -> GraphEdges<'a> {
        self.edges_directed(Direction::Outgoing)
    }
    pub fn edges_with_distance(&'a self, distance: &'a usize) -> impl Iterator<Item=GraphEdge<'a>> + 'a {
        self.edges().into_iter().filter(move |e| e.weight().contains(distance))
    }
    pub fn edges_with_distance_directed(&'a self, distance: usize, direction: Direction)
        -> impl Iterator<Item=GraphEdge<'a>> + 'a {
        self.edges_directed(direction)
            .into_iter()
            .filter(move |e| e.weight().contains(&distance))
    }
    pub fn edges_incoming_with_distance(&'a self, distance: usize) -> impl Iterator<Item=GraphEdge<'a>> + 'a {
        self.edges_with_distance_directed(distance, Direction::Incoming)
    }
    pub fn edges_outgoing_with_distance(&'a self, distance: usize) -> impl Iterator<Item=GraphEdge<'a>> + 'a {
        self.edges_with_distance_directed(distance, Direction::Outgoing)
    }

    pub fn neighbors(&'a self) -> Vec<NodeIndex> {
        self.graph.neighbors(self.index).collect()
    }
    pub fn neighbors_directed(&'a self, direction: Direction) -> impl Iterator<Item=GraphNode<'a>> + 'a {
        self.graph.neighbors_directed(self.index, direction)
            .map(move |i| GraphNode::new(self.graph, i.clone()))
    }
    pub fn neighbors_incoming(&'a self) -> impl Iterator<Item=GraphNode<'a>> + 'a {
        self.neighbors_directed(Direction::Incoming)
    }
    pub fn neighbors_outgoing(&'a self) -> impl Iterator<Item=GraphNode<'a>> + 'a {
        self.neighbors_directed(Direction::Outgoing)
    }
    pub fn neighbors_with_distance(&'a self, distance: usize) -> impl Iterator<Item=GraphNode<'a>> + 'a {
        self.neighbors_incoming_with_distance(distance)
            .chain(self.neighbors_outgoing_with_distance(distance))
    }
    pub fn neighbors_incoming_with_distance(&'a self, distance: usize) -> impl Iterator<Item=GraphNode<'a>> + 'a {
        self.edges_incoming_with_distance(distance)
            .map(|e| e.source())
            .map(move |i| GraphNode::new(self.graph, i.clone()))
    }
    pub fn neighbors_outgoing_with_distance(&'a self, distance: usize) -> impl Iterator<Item=GraphNode<'a>> + 'a {
        self.edges_outgoing_with_distance(distance)
            .map(|e| e.target())
            .map(move |i| GraphNode::new(self.graph, i.clone()))
    }
    pub fn predecessors(&'a self) -> impl Iterator<Item=GraphNode<'a>> + 'a {
        self.neighbors_incoming_with_distance(1)
    }
    pub fn successors(&'a self) -> impl Iterator<Item=GraphNode<'a>> + 'a {
        self.neighbors_outgoing_with_distance(1)
    }
}

mod tests {
    use crate::graph::*;
    use crate::text::*;
    use pretty_assertions::{assert_eq};
    macro_rules! set {
        ( $( $x:expr ),* ) => {  // Match zero or more comma delimited items
            {
                let mut temp_set = HashSet::new();  // Create a mutable HashSet
                $(
                    temp_set.insert($x); // Insert each item matched into the HashSet
                )*
                temp_set // Return the populated HashSet
            }
        };
    }
    #[test]
    fn test_node() {
        let mut tg = TextGraph::new();
        tg.insert_text(Text::from("A B C D"));

        let empty = tg.find_node(&TextElement::Empty).unwrap();
        let a = tg.find_node(&(Word::from("A").into())).unwrap();
        let b = tg.find_node(&(Word::from("B").into())).unwrap();
        let c = tg.find_node(&(Word::from("C").into())).unwrap();
        let d = tg.find_node(&(Word::from("D").into())).unwrap();

        let mut a_preds = a.neighbors_incoming().collect::<HashSet<_>>();
        let mut a_succs = a.neighbors_outgoing().collect::<HashSet<_>>();
        assert_eq!(a_preds, set![empty.clone()]);
        assert_eq!(a_succs, set![b.clone(), c.clone(), d.clone(), empty.clone()]);

        let mut b_preds = b.neighbors_incoming().collect::<HashSet<_>>();
        let mut b_succs = b.neighbors_outgoing().collect::<HashSet<_>>();
        assert_eq!(b_preds, set![empty.clone(), a.clone()]);
        assert_eq!(b_succs, set![c.clone(), d.clone(), empty.clone()]);

        let mut c_preds = c.neighbors_incoming().collect::<HashSet<_>>();
        let mut c_succs = c.neighbors_outgoing().collect::<HashSet<_>>();
        assert_eq!(c_preds, set![empty.clone(), a.clone(), b.clone()]);
        assert_eq!(c_succs, set![d.clone(), empty.clone()]);

        let mut d_preds = d.neighbors_incoming().collect::<HashSet<_>>();
        let mut d_succs = d.neighbors_outgoing().collect::<HashSet<_>>();
        assert_eq!(d_preds, set![empty.clone(), a.clone(), b.clone(), c.clone()]);
        assert_eq!(d_succs, set![empty.clone()]);
    }
}
