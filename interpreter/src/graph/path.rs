use std::fmt::{self, Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};

use crate::*;
use crate::text::*;
use crate::graph::*;
use std::collections::{HashMap, HashSet};
use std::iter::{FromIterator};
use petgraph::{
    *,
    graph::*,
    graphmap::*,
    dot::*,
    visit::*,
};
use nalgebra::base::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct MergingEdge {
    pub matrix_index: usize,
    pub distance: usize,
    pub edge: EdgeIndex,
    pub node: NodeIndex,
}
impl MergingEdge {
    pub fn new(matrix_index: usize, edge_index: EdgeIndex, distance: usize, node_index: NodeIndex) -> Self {
        Self {
            matrix_index,
            edge: edge_index,
            distance,
            node: node_index,
        }
    }
}

#[derive(Clone)]
pub struct TextPath<'a> {
    graph: &'a TextGraph,
    nodes: Vec<NodeIndex>,
    mapping: EdgeMapping,
}

impl<'a> Debug for TextPath<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}
impl<'a> Display for TextPath<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}",
               self.nodes
                .iter()
                .map(|i| GraphNode::new(self.graph, i.clone()))
                .fold(String::new(),
                |acc, n| acc + &n.weight().element().to_string() + " ")
                .trim_end()
        )
    }
}
impl<'a> TextPath<'a> {
    pub fn from_nodes(nodes: Vec<GraphNode<'a>>) -> Option<Self> {
        let mut node_iter = nodes.iter();
        let mut res = Self::from_node(node_iter.next()?);
        res.nodes.reserve(nodes.len());
        for node in node_iter {
            res = Self::try_merge(res, Self::from_node(node))?;
        }
        Some(res)
    }
    pub fn from_text(graph: &'a TextGraph, t: Text) -> Option<Self> {
        Self::from_elements(graph, t.into())
    }
    pub fn from_elements(graph: &'a TextGraph, v: Vec<TextElement>) -> Option<Self> {
        graph.find_nodes(&v)
            .and_then(Self::from_nodes)
    }
    pub fn from_element(graph: &'a TextGraph, element: &TextElement) -> Option<Self> {
        graph.find_node(element)
             .map(|node| Self::from_node(&node))
    }
    pub fn from_node(node: &GraphNode<'a>) -> Self {
        Self {
            graph: node.graph(),
            nodes: vec![node.index()],
            mapping: node.mapping().clone(),
        }
    }
    pub fn new_empty(graph: &'a TextGraph) -> Self {
        Self {
            graph,
            mapping: EdgeMapping::new(),
            nodes: Vec::new(),
        }
    }
    pub fn try_merge(mut left: Self, mut right: Self) -> Option<Self> {
        //println!("EdgeMapping::push: {}", node.weight().element());
        if left.graph as *const _ != right.graph as *const _ {
            return None;
        }
        let graph = left.graph;
        let left_length = left.nodes.len();
        let right_length = right.nodes.len();

        if left_length < 1 {
            return Some(right);
        }

        //println!("Loading left mapping...");
        let incoming_left = load_edges_incoming(graph, &left.mapping.incoming_edges);
        let outgoing_left = load_edges_outgoing(graph, &left.mapping.outgoing_edges);
        let mut left_matrix = &mut left.mapping.matrix;

        //println!("incoming left:\n{:#?}\n", incoming_left);
        //println!("outgoing left:\n{:#?}\n", outgoing_left);
        //println!("left matrix:\n{}\n", left_matrix);


        //println!("Loading right mapping...");
        let incoming_right = load_edges_incoming(graph, &right.mapping.incoming_edges);
        let outgoing_right = load_edges_outgoing(graph, &right.mapping.outgoing_edges);
        let mut right_matrix = &right.mapping.matrix;

        //println!("incoming right:\n{:#?}\n", incoming_right);
        //println!("outgoing right:\n{:#?}\n", outgoing_right);
        //println!("right matrix:\n{}\n", right_matrix);

        // incoming stores the allowed incoming edges (based on distances only)
        // with matrix_index referencing the reduced_left_matrix columns
        //println!("Filter incoming by distance...");
        let mut incoming = filter_allowed_incoming_edges(&incoming_left, &incoming_right, left_length);

        // outgoing stores the allowed outgoing edges (based on distances only)
        // with matrix_index referencing the reduced_right_matrix rows
        //println!("Filter outgoing by distance...");
        let mut outgoing = filter_allowed_outgoing_edges(&outgoing_left, &outgoing_right, right_length);

        //println!("filtered incoming:\n{:#?}\n", incoming);
        //println!("filtered outgoing:\n{:#?}\n", outgoing);


        //println!("Reducing left matrix...");
        let mut reduced_left_matrix =
            EdgeMappingMatrix::from_element(
                outgoing_left.len(),
                incoming.len(),
                false.into());

        for (i, mut edge) in incoming.iter_mut().enumerate() {
            reduced_left_matrix.set_column(i, &left_matrix.column(edge.matrix_index));
            edge.matrix_index = i; // change index to reference reduced matrix
        }
        //println!("Reducing right matrix...");
        let mut reduced_right_matrix =
            EdgeMappingMatrix::from_element(
                outgoing.len(),
                incoming_right.len(),
                false.into());

        for (i, mut edge) in outgoing.iter_mut().enumerate() {
            reduced_right_matrix.set_row(i, &right_matrix.row(edge.matrix_index));
            edge.matrix_index = i; // change index to reference reduced matrix
        }
        let mut left_matrix = reduced_left_matrix;
        let mut right_matrix = reduced_right_matrix;

        //println!("Finding connecting edges...");
        // now find the connecting edges between the two nodes
        // and store their matrix row/column indicies
        let inner_distance = 1;
        let connecting_edges
            : Vec<(usize, usize)> =
            outgoing_left
                .iter()
                .filter(|e| e.distance == inner_distance)
                .filter_map(|left_edge|
                    incoming_right
                        .iter()
                        .filter(|e| e.distance == inner_distance)
                        .find(|right_edge|
                            left_edge.edge == right_edge.edge
                        )
                        .map(|right_edge|
                            (left_edge.matrix_index, right_edge.matrix_index)
                        )
                )
                .collect();
        if connecting_edges.len() < 1 {
            panic!("No connecting edges!");
        }
        //println!("connecting edges:\n{:#?}\n", connecting_edges);

        //println!("Loading connection matrices...");
        let mut connecting_left_matrix =
            EdgeMappingMatrix::from_element(
                connecting_edges.len(),
                incoming.len(),
                false.into());

        let mut connecting_right_matrix =
            EdgeMappingMatrix::from_element(
                outgoing.len(),
                connecting_edges.len(),
                false.into());

        connecting_edges
            .iter()
            .enumerate()
            .for_each(|(i,(left_matrix_index, right_matrix_index))| {
                connecting_left_matrix.set_row(i, &left_matrix.row(*left_matrix_index));
                connecting_right_matrix.set_column(i, &right_matrix.column(*right_matrix_index));
            });
        //println!("connecting left matrix:\n{}\n", connecting_left_matrix);
        //println!("connecting right matrix:\n{}\n", connecting_right_matrix);

        //println!("Filter incoming by connection matrix...");
        let mut incoming: Vec<MergingEdge> = incoming
            .iter()
            .filter(|edge|
                connecting_left_matrix
                    .column(edge.matrix_index)
                    .iter()
                    .any(|x| (*x).into())
            )
            .cloned()
            .collect();

        //println!("Filter outgoing by connection matrix...");
        let mut outgoing: Vec<MergingEdge> = outgoing
            .iter()
            .filter(|edge|
                connecting_right_matrix
                    .row(edge.matrix_index)
                    .iter()
                    .any(|x| (*x).into())
            )
            .cloned()
            .collect();

        //println!("Reducing left matrix by incoming...");
        let mut reduced_left_matrix =
            EdgeMappingMatrix::from_element(
                left_matrix.nrows(),
                incoming.len(),
                false.into());

        for (i, mut edge) in incoming.iter_mut().enumerate() {
            reduced_left_matrix.set_column(i, &left_matrix.column(edge.matrix_index));
            edge.matrix_index = i; // change index to reference reduced matrix
        }
        let mut left_matrix = reduced_left_matrix;

        //println!("Reducing right matrix by outgoing...");
        let mut reduced_right_matrix =
            EdgeMappingMatrix::from_element(
                outgoing.len(),
                right_matrix.ncols(),
                false.into());

        for (i, mut edge) in outgoing.iter_mut().enumerate() {
            reduced_right_matrix.set_row(i, &right_matrix.row(edge.matrix_index));
            edge.matrix_index = i; // change index to reference reduced matrix
        }
        let mut right_matrix = reduced_right_matrix;


        //println!("Reducing left matrix by outgoing...");
        let mut reduced_left_matrix =
            EdgeMappingMatrix::from_element(
                outgoing.len(),
                incoming.len(),
                false.into());

        for (i, mut edge) in outgoing.iter_mut().enumerate() {
            reduced_left_matrix.set_row(i, &left_matrix.row(edge.matrix_index));
            edge.matrix_index = i; // change index to reference reduced matrix
        }
        let mut left_matrix = reduced_left_matrix;

        //println!("Reducing right matrix by incoming...");
        let mut reduced_right_matrix =
            EdgeMappingMatrix::from_element(
                outgoing.len(),
                right_matrix.ncols(),
                false.into());

        for (i, mut edge) in incoming.iter_mut().enumerate() {
            reduced_right_matrix.set_column(i, &right_matrix.column(edge.matrix_index));
            edge.matrix_index = i; // change index to reference reduced matrix
        }
        let mut right_matrix = reduced_right_matrix;

        //println!("Combining matrices...");
        let mut new_matrix = EdgeMappingMatrix::from_fn(
                outgoing.len(),
                incoming.len(),
                |r, c| EdgeMappingMatrixValue::from(left_matrix[(r, c)].into() && right_matrix[(r,c)].into()));

        //println!("final matrix: {}", new_matrix);
        left.nodes.append(&mut right.nodes);
        left.mapping.incoming_edges =
            incoming
            .iter()
            .map(|e| e.edge)
            .collect();

        left.mapping.outgoing_edges =
            outgoing
            .iter()
            .map(|e| e.edge)
            .collect();

        left.mapping.matrix = new_matrix;
        //println!("Merge complete.");
        Some(left)
    }
    //pub fn push_front(&mut self, node: NodeIndex) {
    //    let node = self.graph.get_node(node);
    //    let mut tmp = vec![node];
    //    tmp.extend(self.stack.clone());
    //    self.stack = tmp;
    //}
    pub fn edges_incoming(&self) -> HashSet<GraphEdge<'a>> {
        self.mapping.incoming_edges
            .iter()
            .map(|i| GraphEdge::new(self.graph, i.clone()))
            .collect()
    }
    pub fn neighbors_incoming(&self) -> HashSet<GraphNode<'a>> {
        self.edges_incoming()
            .iter()
            .map(|e| GraphNode::new(self.graph, e.source().clone()))
            .collect()
    }
    pub fn edges_incoming_with_distance(&self, d: usize) -> HashSet<GraphEdge<'a>> {
        self.edges_incoming()
            .iter()
            .filter(|e| e.weight().distance() == d)
            .cloned()
            .collect()
    }
    pub fn neighbors_incoming_with_distance(&self, d: usize) -> HashSet<GraphNode<'a>> {
        self.edges_incoming_with_distance(d)
            .iter()
            .map(|e| GraphNode::new(self.graph, e.source().clone()))
            .collect()
    }
    pub fn predecessors(&'a self) -> HashSet<GraphNode<'a>> {
        self.neighbors_incoming_with_distance(1)
    }
    pub fn edges_outgoing(&self) -> HashSet<GraphEdge<'a>> {
        self.mapping.outgoing_edges
            .iter()
            .map(|i| GraphEdge::new(self.graph, i.clone()))
            .collect()
    }
    pub fn neighbors_outgoing(&self) -> HashSet<GraphNode<'a>> {
        self.edges_outgoing()
            .iter()
            .map(|e| GraphNode::new(self.graph, e.target().clone()))
            .collect()
    }
    pub fn edges_outgoing_with_distance(&self, d: usize) -> HashSet<GraphEdge<'a>> {
        self.edges_outgoing()
            .iter()
            .filter(|e| e.weight().distance() == d)
            .cloned()
            .collect()
    }
    pub fn neighbors_outgoing_with_distance(&self, d: usize) -> HashSet<GraphNode<'a>> {
        self.edges_outgoing_with_distance(d)
            .iter()
            .map(|e| GraphNode::new(self.graph, e.target().clone()))
            .collect()
    }
    pub fn successors(&self) -> HashSet<GraphNode<'a>> {
        self.neighbors_outgoing_with_distance(1)
    }
}

fn filter_allowed_directed_edges(outer: &Vec<MergingEdge>, inner: &Vec<MergingEdge>, distance: usize, direction: Direction) -> Vec<MergingEdge> {
    outer
        .iter()
        .filter(|outer_edge|
            inner
                .iter()
                .find(|inner_edge| {
                    outer_edge.node == inner_edge.node &&
                    outer_edge.distance + distance == inner_edge.distance
                })
                .is_some()
        )
        .cloned()
        .collect()
}
fn filter_allowed_incoming_edges(left: &Vec<MergingEdge>, right: &Vec<MergingEdge>, distance: usize) -> Vec<MergingEdge> {
    filter_allowed_directed_edges(left, right, distance, Direction::Incoming)
}
fn filter_allowed_outgoing_edges(left: &Vec<MergingEdge>, right: &Vec<MergingEdge>, distance: usize) -> Vec<MergingEdge> {
    filter_allowed_directed_edges(right, left, distance, Direction::Outgoing)
}
fn load_edges_directed(graph: &TextGraph, edges: &Vec<EdgeIndex>, direction: Direction) -> Vec<MergingEdge> {
        edges.iter()
            .enumerate()
            .map(|(i, e)|
                MergingEdge::new(i,
                    e.clone(),
                    graph.get_edge(*e).weight().distance(),
                    match direction {
                        Direction::Incoming => graph.get_edge(*e).source(),
                        Direction::Outgoing => graph.get_edge(*e).target(),
                    }
                    )
                )
            .collect()
}
fn load_edges_incoming(graph: &TextGraph, edges: &Vec<EdgeIndex>) -> Vec<MergingEdge> {
    load_edges_directed(graph, edges, Direction::Incoming)
}
fn load_edges_outgoing(graph: &TextGraph, edges: &Vec<EdgeIndex>) -> Vec<MergingEdge> {
    load_edges_directed(graph, edges, Direction::Outgoing)
}
pub mod tests {
    pub use super::*;
    pub use crate::{
        *,
        graph::{
            *,
            node::{
                *,
                tests::{
                    *,
                },
            },
            tests::{
                *,
            },
        },
        text::{
            *,
        },
    };
    lazy_static! {
        pub static ref A_PATH: TextPath<'static> = TG.get_text_path(vec![
            A.clone(),
        ]).unwrap();
        pub static ref B_PATH: TextPath<'static> = TG.get_text_path(vec![
            B.clone(),
        ]).unwrap();
        pub static ref C_PATH: TextPath<'static> = TG.get_text_path(vec![
            C.clone(),
        ]).unwrap();
        pub static ref START_A_PATH: TextPath<'static> = TG.get_text_path(vec![
            START.clone(),
            A.clone(),
        ]).unwrap();
        pub static ref AA_PATH: TextPath<'static> = TG.get_text_path(vec![
            A.clone(),
            A.clone(),
        ]).unwrap();
        pub static ref AB_PATH: TextPath<'static> = TG.get_text_path(vec![
            A.clone(),
            B.clone(),
        ]).unwrap();
        pub static ref BC_PATH: TextPath<'static> = TG.get_text_path(vec![
            B.clone(),
            C.clone(),
        ]).unwrap();
        pub static ref BCD_PATH: TextPath<'static> = TG.get_text_path(vec![
            B.clone(),
            C.clone(),
            D.clone(),
        ]).unwrap();


        static ref A_PREDS: HashSet<GraphNode<'static>> = A_PATH.neighbors_incoming();
        static ref A_SUCCS: HashSet<GraphNode<'static>> = A_PATH.neighbors_outgoing();
        static ref B_PREDS: HashSet<GraphNode<'static>> = B_PATH.neighbors_incoming();
        static ref B_SUCCS: HashSet<GraphNode<'static>> = B_PATH.neighbors_outgoing();
        static ref START_A_PREDS: HashSet<GraphNode<'static>> = START_A_PATH.neighbors_incoming();
        static ref START_A_SUCCS: HashSet<GraphNode<'static>> = START_A_PATH.neighbors_outgoing();
        static ref AA_PREDS: HashSet<GraphNode<'static>> = AA_PATH.neighbors_incoming();
        static ref AA_SUCCS: HashSet<GraphNode<'static>> = AA_PATH.neighbors_outgoing();
        static ref AB_PREDS: HashSet<GraphNode<'static>> = AB_PATH.neighbors_incoming();
        static ref AB_SUCCS: HashSet<GraphNode<'static>> = AB_PATH.neighbors_outgoing();
        static ref BC_PREDS: HashSet<GraphNode<'static>> = BC_PATH.neighbors_incoming();
        static ref BC_SUCCS: HashSet<GraphNode<'static>> = BC_PATH.neighbors_outgoing();
        static ref BCD_PREDS: HashSet<GraphNode<'static>> = BCD_PATH.neighbors_incoming();
        static ref BCD_SUCCS: HashSet<GraphNode<'static>> = BCD_PATH.neighbors_outgoing();

        static ref B_INCOMING: Vec<MergingEdge> = load_edges_incoming(&TG, &B_PATH.mapping.incoming_edges);
        static ref B_OUTGOING: Vec<MergingEdge> = load_edges_outgoing(&TG, &B_PATH.mapping.outgoing_edges);
        static ref C_INCOMING: Vec<MergingEdge> = load_edges_incoming(&TG, &C_PATH.mapping.incoming_edges);
        static ref C_OUTGOING: Vec<MergingEdge> = load_edges_outgoing(&TG, &C_PATH.mapping.outgoing_edges);

    }
    #[test]
    fn find_text() {
        assert!(
            TG.get_text_path(vec![
                B.clone(),
                C.clone(),
                D.clone(),
                STOP.clone(),
            ]).is_some()
        );
    }
    #[test]
    fn from_text() {
        let text = Text::try_from("B C D.")
                    .unwrap();
        assert!(
            TextPath::from_text(&TG, text).is_some()
        );
    }
    pub mod incoming_edges {
        pub use super::*;
        pub use pretty_assertions::{assert_eq};
        pub mod filter {
            pub use super::*;
            pub use pretty_assertions::{assert_eq};
            #[test]
            fn bc_path() {
                assert_eq!(filter_allowed_incoming_edges(&B_INCOMING, &C_INCOMING, 1),
                        B_INCOMING
                            .iter()
                            .filter(|edge|
                                set![
                                    (EMPTY.clone(), 3),
                                    (START.clone(), 2),
                                    (A.clone(), 1),
                                    (B.clone(), 1),
                                    (E.clone(), 1),
                                ].contains(&(TG.get_node(edge.node), edge.distance))
                            )
                            .cloned()
                            .collect::<Vec<_>>()
                        );
            }
        }
        #[test]
        fn bc_path() {
            assert_eq!(
                BC_PATH.mapping.incoming_edges
                    .iter()
                    .cloned()
                    .collect::<HashSet<_>>(),
                B_PATH.mapping.incoming_edges
                    .iter()
                    .filter(|edge| {
                        let edge = TG.get_edge(**edge);
                        set![
                            (EMPTY.clone(), 3),
                            (START.clone(), 2),
                            (STOP.clone(), 2),
                            (A.clone(), 1),
                            (B.clone(), 1),
                        ].contains(&(TG.get_node(edge.source()), edge.weight().distance()))
                    })
                    .cloned()
                    .collect::<HashSet<_>>(),
                );
        }
    }
    pub mod outgoing_edges {
        pub use super::*;
        pub use pretty_assertions::{assert_eq};
        pub mod filter {
            pub use super::*;
            pub use pretty_assertions::{assert_eq};
            #[test]
            fn bc_path() {
                assert_eq!(filter_allowed_outgoing_edges(&B_OUTGOING, &C_OUTGOING, 1),
                        C_OUTGOING
                            .iter()
                            .filter(|edge|
                                set![
                                    (EMPTY.clone(), 3),
                                    (STOP.clone(), 2),
                                    (D.clone(), 1),
                                    (C.clone(), 1),
                                    (F.clone(), 1),
                                ].contains(&(TG.get_node(edge.node), edge.distance))
                            )
                            .cloned()
                            .collect::<Vec<_>>()
                        );
            }
        }
        #[test]
        fn bc_path() {
            assert_eq!(
                BC_PATH
                    .mapping
                    .outgoing_edges
                    .iter()
                    .cloned()
                    .collect::<HashSet<_>>(),
                C_PATH.mapping.outgoing_edges
                    .iter()
                    .filter(|edge| {
                        let edge = TG.get_edge(**edge);
                        set![
                            (EMPTY.clone(), 3),
                            (STOP.clone(), 2),
                            (C.clone(), 1),
                            (D.clone(), 1)
                        ].contains(&(TG.get_node(edge.target()), edge.weight().distance()))
                    })
                    .cloned()
                    .collect::<HashSet<_>>(),
                );
        }
    }
    pub mod neighbors {
        pub use super::*;
        pub use pretty_assertions::{assert_eq};
        #[test]
        fn start_a_preds() {
            assert_eq!(*START_A_PREDS, set![
                EMPTY.clone()
            ]);
        }
        #[test]
        fn start_a_succs() {
            assert_eq!(*START_A_SUCCS, set![
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
        fn aa_preds() {
            assert_eq!(*AA_PREDS, set![
                EMPTY.clone(),
                START.clone(),
            ]);
        }
        #[test]
        fn aa_succs() {
            assert_eq!(*AA_SUCCS, set![
                EMPTY.clone(),
                STOP.clone(),
                A.clone(),
                F.clone(),
            ]);
        }
        #[test]
        fn ab_preds() {
            assert_eq!(*AB_PREDS, set![
                EMPTY.clone(),
                START.clone(),
            ]);
        }
        #[test]
        fn ab_succs() {
            assert_eq!(*AB_SUCCS, set![
                EMPTY.clone(),
                STOP.clone(),
                C.clone(),
                D.clone(),
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
        fn bc_preds() {
            assert_eq!(*BC_PREDS, set![
                EMPTY.clone(),
                START.clone(),
                A.clone(),
                B.clone(),
            ]);
        }
        #[test]
        fn bc_succs() {
            assert_eq!(*BC_SUCCS, set![
                EMPTY.clone(),
                STOP.clone(),
                D.clone(),
                C.clone(),
            ]);
        }
        #[test]
        fn bcd_preds() {
            assert_eq!(*BCD_PREDS, set![
                EMPTY.clone(),
                START.clone(),
                A.clone(),
            ]);
        }
        #[test]
        fn bcd_succs() {
            assert_eq!(*BCD_SUCCS, set![
                EMPTY.clone(),
                STOP.clone(),
            ]);
        }
    }
}
