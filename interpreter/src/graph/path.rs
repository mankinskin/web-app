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
        write!(f, "TextPath(\n\tnodes: {:#?},\n\tmapping: {:#?}\n)", self.nodes, self.mapping)
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
    pub fn new(graph: &'a TextGraph, v: Vec<TextElement>) -> Option<Self> {
        let mut res = Self {
            graph,
            nodes: Vec::new(),
            mapping: EdgeMapping::new(),
        };
        res.nodes.reserve(v.len());
        for e in &v {
            res = Self::try_merge(res, Self::from_element(graph, e)?)?;
        }
        Some(res)
    }
    pub fn from_element(graph: &'a TextGraph, element: &TextElement) -> Option<Self> {
        let node = graph.find_node(element)?;
        Self::from_node(&node)
    }
    pub fn from_node(node: &GraphNode<'a>) -> Option<Self> {
        Some(Self {
            graph: node.graph(),
            nodes: vec![node.index()],
            mapping: node.mapping().clone(),
        })
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

        if left_length < 1 {
            return Some(right);
        }

        let incoming_left = load_edges_incoming(graph, &left.mapping.incoming_edges);
        let outgoing_left = load_edges_outgoing(graph, &left.mapping.outgoing_edges);
        let mut left_matrix = &mut left.mapping.matrix;

        //println!("incoming left:\n{:#?}\n", incoming_left);
        //println!("outgoing left:\n{:#?}\n", outgoing_left);
        //println!("left matrix:\n{}\n", left_matrix);


        let incoming_right = load_edges_incoming(graph, &right.mapping.incoming_edges);
        let outgoing_right = load_edges_outgoing(graph, &right.mapping.outgoing_edges);
        let mut right_matrix = &right.mapping.matrix;

        //println!("incoming right:\n{:#?}\n", incoming_right);
        //println!("outgoing right:\n{:#?}\n", outgoing_right);
        //println!("right matrix:\n{}\n", right_matrix);

        // outer edges
        let mut incoming = filter_allowed_incoming_edges(&incoming_left, &incoming_right, left_length);
        let mut outgoing = filter_allowed_outgoing_edges(&outgoing_left, &outgoing_right, 1);

        //println!("filtered incoming:\n{:#?}\n", incoming);
        //println!("filtered outgoing:\n{:#?}\n", outgoing);

        let mut reduced_left_matrix =
            EdgeMappingMatrix::from_element(
                outgoing_left.len(),
                incoming.len(),
                false.into());

        for (i, mut edge) in incoming.iter_mut().enumerate() {
            reduced_left_matrix.set_column(i, &left_matrix.column(edge.matrix_index));
            edge.matrix_index = i; // change index to reference reduced matrix
        }
        // incoming stores the allowed incoming edges (based on distances only)
        // with matrix_index referencing the reduced_left_matrix columns

        let mut reduced_right_matrix =
            EdgeMappingMatrix::from_element(
                outgoing.len(),
                incoming_right.len(),
                false.into());

        for (i, mut edge) in outgoing.iter_mut().enumerate() {
            reduced_right_matrix.set_row(i, &right_matrix.row(edge.matrix_index));
            edge.matrix_index = i; // change index to reference reduced matrix
        }
        // outgoing stores the allowed outgoing edges (based on distances only)
        // with matrix_index referencing the reduced_right_matrix rows

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

        // now reduce the matrices again to only include the connected edges
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
                connecting_left_matrix.set_row(i, &reduced_left_matrix.row(*left_matrix_index));
                connecting_right_matrix.set_column(i, &reduced_right_matrix.column(*right_matrix_index));
            });

        //println!("connecting left matrix:\n{}\n", connecting_left_matrix);
        //println!("connecting right matrix:\n{}\n", connecting_right_matrix);


        let valid_column_iter = connecting_left_matrix
                .column_iter()
                .zip(incoming)
                .filter(|(c, _)| c.iter().any(|x| (*x).into()));
        let valid_column_count = valid_column_iter.clone().count();
        let mut new_left_matrix =
            EdgeMappingMatrix::from_element(
                connecting_edges.len(),
                valid_column_count,
                false.into());

        let mut new_incoming = Vec::new();
        new_incoming.reserve(valid_column_count);

        for (i, (column, edge)) in valid_column_iter.enumerate() {
            new_incoming.push(edge);
            new_left_matrix.set_column(i, &column);
        }
        //println!("final incoming:\n{:#?}\n", new_incoming);
        //println!("new left matrix:\n{}\n", new_left_matrix);

        let valid_row_iter = connecting_right_matrix
                .row_iter()
                .zip(outgoing)
                .filter(|(r, _)| r.iter().any(|x| (*x).into()));
        let valid_row_count = valid_row_iter.clone().count();
        let mut new_right_matrix =
            EdgeMappingMatrix::from_element(
                valid_row_count,
                connecting_edges.len(),
                false.into());

        let mut new_outgoing = Vec::new();
        new_outgoing.reserve(valid_row_count);

        for (i, (row, edge)) in valid_row_iter.enumerate() {
            new_outgoing.push(edge);
            new_right_matrix.set_row(i, &row);
        }
        //println!("final outgoing:\n{:#?}\n", new_outgoing);
        //println!("new right matrix:\n{}\n", new_right_matrix);
        // multiply matricies together to get a new matrix applying both matricies
        let new_matrix = &new_right_matrix * &new_left_matrix;

        //println!("final matrix: {}", new_matrix);
        left.nodes.append(&mut right.nodes);
        left.mapping.incoming_edges =
            new_incoming
            .iter()
            .map(|e| e.edge)
            .collect();

        left.mapping.outgoing_edges =
            new_outgoing
            .iter()
            .map(|e| e.edge)
            .collect();

        left.mapping.matrix = new_matrix;
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
                    outer_edge.distance == inner_edge.distance - distance
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
mod tests {

    use crate::*;
    use crate::graph::*;
    use crate::text::*;
    use pretty_assertions::{assert_eq};
    use super::*;
    #[test]
    fn push_node() {
        let mut tg = TextGraph::new();
        tg.read_text(Text::from("\
                A B C D.\
                B B C C.
                E A C F.\
                E B D F."));

        let empty = tg.find_node(&TextElement::Empty).unwrap();
        let a = tg.find_node(&(Word::from("A").into())).unwrap();
        let b = tg.find_node(&(Word::from("B").into())).unwrap();
        let c = tg.find_node(&(Word::from("C").into())).unwrap();
        let d = tg.find_node(&(Word::from("D").into())).unwrap();
        let e = tg.find_node(&(Word::from("E").into())).unwrap();
        let f = tg.find_node(&(Word::from("F").into())).unwrap();

        let mut bc_path = tg.get_text_path(vec![
            Word::from("B").into(),
            Word::from("C").into()
        ]).unwrap();
        let mut b_path = tg.get_text_path(vec![
            Word::from("B").into(),
        ]).unwrap();
        let mut c_path = tg.get_text_path(vec![
            Word::from("C").into()
        ]).unwrap();

        assert_eq!(
            bc_path.mapping.incoming_edges
                .iter()
                .cloned()
                .collect::<HashSet<_>>(),
            b_path.mapping.incoming_edges
                .iter()
                .filter(|edge| {
                    let edge = tg.get_edge(**edge);
                    set![
                        (empty.clone(), 2),
                        (a.clone(), 1),
                        (b.clone(), 1)
                    ].contains(&(tg.get_node(edge.source()), edge.weight().distance()))
                })
                .cloned()
                .collect::<HashSet<_>>(),
            );
        assert_eq!(
            bc_path.mapping.outgoing_edges
                .iter()
                .cloned()
                .collect::<HashSet<_>>(),
            c_path.mapping.outgoing_edges
                .iter()
                .filter(|edge| {
                    let edge = tg.get_edge(**edge);
                    set![
                        (empty.clone(), 2),
                        (c.clone(), 1),
                        (d.clone(), 1)
                    ].contains(&(tg.get_node(edge.target()), edge.weight().distance()))
                })
                .cloned()
                .collect::<HashSet<_>>(),
            );
    }
    #[test]
    fn filter_allowed_edges() {
        let mut tg = TextGraph::new();
        tg.read_text(Text::from("\
                A B C D.\
                E A C F.\
                E B D F."));

        let empty = tg.find_node(&TextElement::Empty).unwrap();
        let a = tg.find_node(&(Word::from("A").into())).unwrap();
        let b = tg.find_node(&(Word::from("B").into())).unwrap();
        let c = tg.find_node(&(Word::from("C").into())).unwrap();
        let d = tg.find_node(&(Word::from("D").into())).unwrap();
        let e = tg.find_node(&(Word::from("E").into())).unwrap();
        let f = tg.find_node(&(Word::from("F").into())).unwrap();

        let b_path = tg.get_text_path(vec![Word::from("B").into()]).unwrap();
        let c_path = tg.get_text_path(vec![Word::from("C").into()]).unwrap();

        let b_incoming = load_edges_incoming(&tg, &b_path.mapping.incoming_edges);
        let b_outgoing = load_edges_outgoing(&tg, &b_path.mapping.outgoing_edges);

        let c_incoming = load_edges_incoming(&tg, &c_path.mapping.incoming_edges);
        let c_outgoing = load_edges_outgoing(&tg, &c_path.mapping.outgoing_edges);

        assert_eq!(filter_allowed_incoming_edges(&b_incoming, &c_incoming, b_path.nodes.len()),
                b_incoming.iter()
                .filter(|edge| set![
                    empty.clone(),
                    a.clone(),
                    e.clone()
                ].contains(&tg.get_node(edge.node)))
                .cloned()
                .collect::<Vec<_>>()
                );
        assert_eq!(filter_allowed_outgoing_edges(&b_outgoing, &c_outgoing, 1),
                c_outgoing.iter()
                .filter(|edge| set![
                    empty.clone(),
                    d.clone(),
                    f.clone()
                ].contains(&tg.get_node(edge.node)))
                .cloned()
                .collect::<Vec<_>>()
                );
    }
    #[test]
    fn direct_neighbors() {
        let mut tg = TextGraph::new();
        tg.read_text(Text::from("\
                A B C D E.\
                A B D A C.\
                A A A A A."));
        //tg.read_text(Text::from("\
        //        A B C.\
        //        A C.\
        //        A A."));
        tg.write_to_file("graphs/test_graph");

        let empty = tg.find_node(&TextElement::Empty).unwrap();
        let a = tg.find_node(&(Word::from("A").into())).unwrap();
        let b = tg.find_node(&(Word::from("B").into())).unwrap();
        let c = tg.find_node(&(Word::from("C").into())).unwrap();
        let d = tg.find_node(&(Word::from("D").into())).unwrap();
        let e = tg.find_node(&(Word::from("E").into())).unwrap();
        //let dot = tg.find_node(&(Punctuation::Dot.into())).unwrap();

        let a_stack = tg.get_text_path(vec![
            Word::from("A").into(),
        ]).unwrap();
        let a_preds = a_stack.predecessors();
        let a_succs = a_stack.successors();
        assert_eq!(a_preds, set![
            empty.clone(),
            d.clone(),
            a.clone()
        ]);
        assert_eq!(a_succs, set![
            a.clone(),
            b.clone(),
            c.clone(),
            empty.clone()
        ]);

        let b_stack = tg.get_text_path(vec![
            Word::from("B").into(),
        ]).unwrap();
        let b_preds = b_stack.predecessors();
        let b_succs = b_stack.successors();
        assert_eq!(b_preds, set![
            a.clone()
        ]);
        assert_eq!(b_succs, set![
            c.clone(),
            d.clone()
        ]);

        let ab = tg.get_text_path(vec![
            Word::from("A").into(),
            Word::from("B").into()
        ]).unwrap();
        let ab_preds = ab.predecessors();
        let ab_succs = ab.successors();
        assert_eq!(ab_preds, set![
            empty.clone()
        ]);
        assert_eq!(ab_succs, set![
            c.clone(),
            d.clone()
        ]);

        let bc = tg.get_text_path(vec![
            Word::from("B").into(),
            Word::from("C").into()
        ]).unwrap();
        let bc_preds = bc.predecessors();
        let bc_succs = bc.successors();
        assert_eq!(bc_preds, set![
            a.clone()
        ]);
        assert_eq!(bc_succs, set![
            d.clone()
        ]);

        let bcd = tg.get_text_path(
            vec![
            Word::from("B").into(),
            Word::from("C").into(),
            Word::from("D").into()
        ]).unwrap();
        let bcd_preds = bcd.predecessors();
        let bcd_succs = bcd.successors();
        assert_eq!(bcd_preds, set![
            a.clone()
        ]);
        assert_eq!(bcd_succs, set![
            e.clone()
        ]);

        let aa = tg.get_text_path(vec![
            Word::from("A").into(),
            Word::from("A").into()
        ]).unwrap();

        let aa_preds = aa.predecessors();
        let aa_succs = aa.successors();
        assert_eq!(aa_preds, set![
            empty.clone(),
            a.clone()
        ]);
        assert_eq!(aa_succs, set![
            a.clone(),
            empty.clone()
        ]);
    }
    #[test]
    fn neighbors() {
        let mut tg = TextGraph::new();
        tg.read_text(Text::from("\
                A B C D E.\
                A B D A C.\
                A A A A A."));
        tg.write_to_file("graphs/test_graph");

        let empty = tg.find_node(&TextElement::Empty).unwrap();
        let a = tg.find_node(&(Word::from("A").into())).unwrap();
        let b = tg.find_node(&(Word::from("B").into())).unwrap();
        let c = tg.find_node(&(Word::from("C").into())).unwrap();
        let d = tg.find_node(&(Word::from("D").into())).unwrap();
        let e = tg.find_node(&(Word::from("E").into())).unwrap();
        //let dot = tg.find_node(&(Punctuation::Dot.into())).unwrap();


        let b_stack = tg.get_text_path(vec![
            Word::from("B").into(),
        ]).unwrap();
        let mut b_preds = b_stack.neighbors_incoming();
        let mut b_succs = b_stack.neighbors_outgoing();
        assert_eq!(b_preds, set![
            empty.clone(),
            a.clone()
        ]);
        assert_eq!(b_succs, set![
            c.clone(),
            d.clone(),
            e.clone(),
            a.clone(),
            empty.clone()
        ]);

        let a_stack = tg.get_text_path(vec![
            Word::from("A").into(),
        ]).unwrap();
        let mut a_preds = a_stack.neighbors_incoming();
        let mut a_succs = a_stack.neighbors_outgoing();
        assert_eq!(a_preds, set![
            empty.clone(),
            b.clone(),
            d.clone(),
            a.clone()
        ]);
        assert_eq!(a_succs, set![
            b.clone(),
            c.clone(),
            d.clone(),
            e.clone(),
            a.clone(),
            empty.clone()
        ]);

        //let empty_a_stack = tg.get_text_path(vec![
        //    TextElement::Empty,
        //    Word::from("A").into(),
        //]).unwrap();
        //let mut empty_a_preds = empty_a_stack.neighbors_incoming();
        //let mut empty_a_succs = empty_a_stack.neighbors_outgoing();
        //assert_eq!(empty_a_preds, set![
        //    a.clone()
        //]);
        //assert_eq!(empty_a_succs, set![
        //    b.clone(),
        //    c.clone(),
        //    d.clone(),
        //    e.clone(),
        //    a.clone()
        //]);

        let ab = tg.get_text_path(vec![
            Word::from("A").into(),
            Word::from("B").into()
        ]).unwrap();
        let mut ab_preds = ab.neighbors_incoming();
        let mut ab_succs = ab.neighbors_outgoing();
        assert_eq!(ab_preds, set![
            empty.clone()
        ]);
        assert_eq!(ab_succs, set![
            c.clone(),
            d.clone(),
            e.clone(),
            a.clone(),
            empty.clone()
        ]);

        let bc = tg.get_text_path(vec![
            Word::from("B").into(),
            Word::from("C").into()
        ]).unwrap();
        let mut bc_preds = bc.neighbors_incoming();
        let mut bc_succs = bc.neighbors_outgoing();
        assert_eq!(bc_preds, set![
            empty.clone(),
            a.clone()
        ]);
        assert_eq!(bc_succs, set![
            d.clone(),
            e.clone(),
            empty.clone()
        ]);

        let bcd = tg.get_text_path(
            vec![
            Word::from("B").into(),
            Word::from("C").into(),
            Word::from("D").into()
        ]).unwrap();
        let mut bcd_preds = bcd.neighbors_incoming();
        let mut bcd_succs = bcd.neighbors_outgoing();
        assert_eq!(bcd_preds, set![
            empty.clone(),
            a.clone()
        ]);
        assert_eq!(bcd_succs, set![
            e.clone(),
            empty.clone()
        ]);
    }
}
