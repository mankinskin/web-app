use petgraph::{
    *,
    graph::*,
    graphmap::*,
    dot::*,
    visit::*,
};
use std::fmt::{self, Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::convert::TryFrom;

use crate::graph::edges::*;
use crate::graph::*;

#[derive(PartialEq)]
pub struct TextGraphNodeWeight  {
    text_element: TextElement,
    mapping: EdgeMapping,
}
impl TextGraphNodeWeight  {
    pub fn element(&self) -> &TextElement {
        &self.text_element
    }
    pub fn mapping(&self) -> &EdgeMapping {
        &self.mapping
    }
}

impl Debug for TextGraphNodeWeight {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        //write!(f, "{:?} mapping: {:?}", self.text_element, self.mapping)
        write!(f, "{:?}", self.text_element)
    }
}
impl<'a> TextGraphNodeWeight {
    pub fn new(text_element: TextElement) -> Self {
        Self {
            text_element,
            mapping: EdgeMapping::new(),
        }
    }
}
impl Deref for TextGraphNodeWeight {
    type Target = EdgeMapping;
    fn deref(&self) -> &Self::Target {
        &self.mapping
    }
}
impl DerefMut for TextGraphNodeWeight {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mapping
    }
}

#[derive(Clone)]
pub struct GraphNode<'a>  {
    graph: &'a TextGraph,
    index: NodeIndex,
}
impl<'a> GraphNode<'a> {
    pub fn new(graph: &'a TextGraph, index: NodeIndex) -> Self {
        GraphNode {
            graph,
            index,
        }
    }
    pub fn graph(&self) -> &'a TextGraph {
        self.graph
    }
    pub fn weight(&self) -> &TextGraphNodeWeight {
        <&TextGraphNodeWeight>::from(self)
    }
    pub fn index(&'a self) -> NodeIndex {
        self.index
    }
    pub fn is_at_distance(&'a self, other: &GraphNode<'a>, distance: usize) -> bool {
        self.graph
            .find_edge(self, other, distance)
            .is_some()
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
        self.edges().into_iter().filter(move |e| e.weight().distance() == *distance)
    }
    pub fn edges_with_distance_directed(&'a self, distance: usize, direction: Direction)
        -> impl Iterator<Item=GraphEdge<'a>> + 'a {
        self.edges_directed(direction)
            .into_iter()
            .filter(move |e| e.weight().distance() == distance)
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
        n.weight().element()
    }
}
impl<'a> Into<NodeIndex> for GraphNode<'a> {
    fn into(self) -> NodeIndex {
        self.index
    }
}
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
        let incoming_weight_groups = incoming_edges.clone().group_by_distance();
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

        let outgoing_weight_groups = outgoing_edges.clone().group_by_distance();
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
pub mod tests {
    pub use super::*;
    pub use crate::{
        *,
        graph::{
            *,
        },
        text::{
            *,
        },
    };
    lazy_static! {
        pub static ref TG: TextGraph = {
            let mut graph = TextGraph::new();
            graph.read_text(
                Text::try_from("\
                    A B C D.\
                    B B C C.
                    E A C F.\
                    E B D F.
                    A A F A")
                    .unwrap()
            );
            graph
        };
        pub static ref EMPTY: GraphNode<'static> = TG.find_node(&TextElement::Empty).unwrap();
        pub static ref START: GraphNode<'static> = TG.find_node(&TextElement::Start).unwrap();
        pub static ref STOP: GraphNode<'static> = TG.find_node(&TextElement::Stop).unwrap();
        pub static ref A: GraphNode<'static>= TG.find_node(&(Word::from("A").into())).unwrap();
        pub static ref B: GraphNode<'static>= TG.find_node(&(Word::from("B").into())).unwrap();
        pub static ref C: GraphNode<'static>= TG.find_node(&(Word::from("C").into())).unwrap();
        pub static ref D: GraphNode<'static>= TG.find_node(&(Word::from("D").into())).unwrap();
        pub static ref E: GraphNode<'static>= TG.find_node(&(Word::from("E").into())).unwrap();
        pub static ref F: GraphNode<'static>= TG.find_node(&(Word::from("F").into())).unwrap();

        static ref A_PREDS: HashSet<GraphNode<'static>> = A.neighbors_incoming().collect::<HashSet<_>>();
        static ref B_PREDS: HashSet<GraphNode<'static>> = B.neighbors_incoming().collect::<HashSet<_>>();
        static ref C_PREDS: HashSet<GraphNode<'static>> = C.neighbors_incoming().collect::<HashSet<_>>();
        static ref D_PREDS: HashSet<GraphNode<'static>> = D.neighbors_incoming().collect::<HashSet<_>>();

        static ref A_SUCCS: HashSet<GraphNode<'static>> = A.neighbors_outgoing().collect::<HashSet<_>>();
        static ref B_SUCCS: HashSet<GraphNode<'static>> = B.neighbors_outgoing().collect::<HashSet<_>>();
        static ref C_SUCCS: HashSet<GraphNode<'static>> = C.neighbors_outgoing().collect::<HashSet<_>>();
        static ref D_SUCCS: HashSet<GraphNode<'static>> = D.neighbors_outgoing().collect::<HashSet<_>>();
    }
    mod neighbors {
        pub use pretty_assertions::{assert_eq};
        pub use super::*;
        #[test]
        fn a_preds() {
            assert_eq!(*A_PREDS, set![
                EMPTY.clone(),
                START.clone(),
                E.clone(),
                A.clone(),
                F.clone(),
            ]);
        }
        #[test]
        fn a_succs() {
            assert_eq!(*A_SUCCS, set![
                EMPTY.clone(),
                STOP.clone(),
                A.clone(),
                B.clone(),
                C.clone(),
                D.clone(),
                F.clone(),
            ]);

        }
        #[test]
        fn b_preds() {
            assert_eq!(*B_PREDS, set![
                EMPTY.clone(),
                START.clone(),
                A.clone(),
                B.clone(),
                E.clone(),
            ]);
        }
        #[test]
        fn b_succs() {
            assert_eq!(*B_SUCCS, set![
                EMPTY.clone(),
                STOP.clone(),
                B.clone(),
                C.clone(),
                D.clone(),
                F.clone(),
            ]);

        }
        #[test]
        fn c_preds() {
            assert_eq!(*C_PREDS, set![
                EMPTY.clone(),
                START.clone(),
                A.clone(),
                C.clone(),
                B.clone(),
                E.clone(),
            ]);
        }
        #[test]
        fn c_succs() {
            assert_eq!(*C_SUCCS, set![
                EMPTY.clone(),
                STOP.clone(),
                C.clone(),
                D.clone(),
                F.clone(),
            ]);
        }
        #[test]
        fn d_preds() {

            assert_eq!(*D_PREDS, set![
                EMPTY.clone(),
                START.clone(),
                A.clone(),
                B.clone(),
                C.clone(),
                E.clone(),
            ]);
        }
        #[test]
        fn d_succs() {
            assert_eq!(*D_SUCCS, set![
                EMPTY.clone(),
                STOP.clone(),
                F.clone(),
            ]);
        }
    }
}
