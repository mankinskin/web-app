use crate::{
    graph::{
        edge::EdgeData,
        node::NodeData,
        Graph,
    },
    SequenceGraph,
};
use petgraph::graph::{
    EdgeIndex,
    NodeIndex,
};
use serde::{
    Deserialize,
    Serialize,
};
use itertools::Itertools;
use std::{
    ops::{
        Mul,
        MulAssign,
        Add,
        AddAssign,
    },
    iter::repeat,
    default::Default,
    fmt::{
        self,
        Formatter,
        Debug,
        Display,
    },
};
use nalgebra::*;
use tracing::{
    debug,
};
pub type EdgeMappingMatrix = nalgebra::DMatrix<ArithmeticBool>;

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Default, Debug)]
pub struct ArithmeticBool(bool);

impl Display for ArithmeticBool {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", match self.0 {
            true => "1",
            false => "0",
        })
    }
}
impl num_traits::Zero for ArithmeticBool {
    fn zero() -> Self {
        Self(false)
    }
    fn is_zero(&self) -> bool {
        !self.0
    }
}
impl num_traits::One for ArithmeticBool {
    fn one() -> Self {
        Self(true)
    }
    fn is_one(&self) -> bool {
        self.0
    }
}
impl From<bool> for ArithmeticBool {
    fn from(b: bool) -> Self {
        Self(b)
    }
}
impl Into<bool> for ArithmeticBool {
    fn into(self) -> bool {
        self.0
    }
}
impl Mul for ArithmeticBool {
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        Self::from(self.0 && other.0)
    }
}
impl Add for ArithmeticBool {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self::from(self.0 || other.0)
    }
}
impl AddAssign for ArithmeticBool {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}
impl MulAssign for ArithmeticBool {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}
#[derive(PartialEq, Clone, Debug)]
pub struct Edge {
    index: EdgeIndex,
    node: NodeIndex,
    dist: usize,
}
impl Edge {
    pub fn new(index: EdgeIndex, node: NodeIndex, dist: usize) -> Self {
        Self {
            index,
            node,
            dist,
        }
    }
}
#[derive(PartialEq, Clone, Debug)]
pub struct EdgeMapping {
    pub matrix: EdgeMappingMatrix,
    pub incoming: Vec<Edge>,
    pub outgoing: Vec<Edge>,
}
impl<'a> EdgeMapping {
    /// New EdgeMapping
    pub fn new() -> Self {
        Self {
            matrix: EdgeMappingMatrix::from_element(0, 0, false.into()),
            incoming: Vec::new(),
            outgoing: Vec::new(),
        }
    }
    /// Add an incoming edge
    fn add_incoming_edge(&mut self, edge: Edge) -> usize {
        if let Some(i) = self.incoming.iter().position(|e| *e == edge) {
            i
        } else {
            self.incoming.push(edge);
            self.matrix = self
                .matrix
                .clone()
                .insert_column(self.matrix.ncols(), false.into());
            self.incoming.len() - 1
        }
    }
    /// Add an outgoing edge
    fn add_outgoing_edge(&mut self, edge: Edge) -> usize {
        if let Some(i) = self.outgoing.iter().position(|e| *e == edge) {
            i
        } else {
            self.outgoing.push(edge);
            self.matrix = self
                .matrix
                .clone()
                .insert_row(self.matrix.nrows(), false.into());
            self.outgoing.len() - 1
        }
    }
    /// Add a transition between two edges
    pub fn add_transition(&mut self, l: Edge, r: Edge) {
        let li = self.add_incoming_edge(l);
        let ri = self.add_outgoing_edge(r);
        self.matrix[(ri, li)] = true.into();
    }
    pub fn remove_zero_columns(&mut self) {
        let (incoming, columns): (_, Vec<_>) = self.incoming
            .iter()
            .cloned()
            .zip(self.matrix.column_iter())
            .filter(|(_, col)| col.iter().any(|b| b.0))
            .unzip();
        self.incoming = incoming;
        self.matrix = EdgeMappingMatrix::from_columns(&columns);
    }
    pub fn remove_zero_rows(&mut self) {
        let (outgoing, rows): (_, Vec<_>) = self.outgoing
            .iter()
            .cloned()
            .zip(self.matrix.row_iter())
            .filter(|(_, row)| row.iter().any(|b| b.0))
            .unzip();
        self.outgoing = outgoing;
        self.matrix = EdgeMappingMatrix::from_rows(&rows);
    }
    ///// Get weights and sources of incoming edges
    //pub fn incoming_sources<T: NodeData, E: EdgeData>(
    //    &'a self,
    //    graph: &'a Graph<T, E>,
    //) -> impl Iterator<Item = (E, NodeIndex)> + 'a {
    //    graph
    //        .edge_weights(self.incoming.iter())
    //        .zip(graph.edge_sources(self.incoming.iter()))
    //}
    ///// Get weights and targets of outgoing edges
    //pub fn outgoing_targets<T: NodeData, E: EdgeData>(
    //    &'a self,
    //    graph: &'a Graph<T, E>,
    //) -> impl Iterator<Item = (E, NodeIndex)> + 'a {
    //    graph
    //        .edge_weights(self.outgoing.iter())
    //        .zip(graph.edge_targets(self.outgoing.iter()))
    //}

    ///// Get distance groups for incoming edges
    //pub fn incoming_distance_groups<T: NodeData + Wide>(
    //    &self,
    //    graph: &SequenceGraph<T>,
    //) -> Vec<Vec<Node<T>>> {
    //    graph.distance_group_source_weights(self.incoming.iter())
    //}
    ///// Get distance groups for outgoing edges
    //pub fn outgoing_distance_groups<T: NodeData + Wide>(
    //    &self,
    //    graph: &SequenceGraph<T>,
    //) -> Vec<Vec<Node<T>>> {
    //    graph.distance_group_target_weights(self.outgoing.iter())
    //}
}
impl Default for EdgeMapping {
    fn default() -> Self {
        Self::new()
    }
}
pub trait Wide {
    fn width(&self) -> usize;
}
impl Wide for char {
    fn width(&self) -> usize {
        1
    }
}

pub trait TokenData: NodeData + Wide {}
impl<T: NodeData + Wide> TokenData for T {}

/// Type for storing elements of a sequence
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Token<T: TokenData> {
    Element(T),
    Tokens(Vec<Self>),
    Start,
    End,
}
impl<T: TokenData + Display> Display for Token<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Token::Element(t) => t.to_string(),
                Token::Tokens(v) => format!(
                    "{:#?}",
                    v.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                ),
                Token::Start => "START".to_string(),
                Token::End => "END".to_string(),
            }
        )
    }
}
impl<T: TokenData> Add for Token<T> {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        match self {
            Token::Tokens(mut v) => match other {
                Token::Tokens(t) => {
                    v.extend(t);
                    Token::Tokens(v)
                },
                _ => {
                    v.push(other);
                    Token::Tokens(v)
                },
            },
            _ => match other {
                Token::Tokens(t) => {
                    let mut v = vec![self];
                    v.extend(t);
                    Token::Tokens(v)
                },
                _ => Token::Tokens(vec![self, other]),
            },
        }
    }
}
impl<T: TokenData> Wide for Token<T> {
    fn width(&self) -> usize {
        match self {
            Token::Element(t) => t.width(),
            Token::Tokens(v) => v.iter().fold(0, |acc, x| acc + x.width()),
            Token::Start => 0,
            Token::End => 0,
        }
    }
}
impl<T: TokenData> From<T> for Token<T> {
    fn from(e: T) -> Self {
        Token::Element(e)
    }
}
impl<T: TokenData> PartialEq<Node<T>> for Token<T> {
    fn eq(&self, rhs: &Node<T>) -> bool {
        *self == rhs.token
    }
}
impl<T: TokenData> PartialEq<Node<T>> for &Token<T> {
    fn eq(&self, rhs: &Node<T>) -> bool {
        **self == rhs.token
    }
}
/// Stores sequenced tokens with an edge map
#[derive(PartialEq, Clone)]
pub struct Node<T: TokenData> {
    pub token: Token<T>,
    mapping: EdgeMapping,
}
impl<T: TokenData> Node<T> {
    pub fn new(token: Token<T>) -> Self {
        Self {
            token,
            mapping: EdgeMapping::new(),
        }
    }
    pub fn get_token(&self) -> &Token<T> {
        &self.token
    }
    fn output_intersections(lhs: Self, rhs: Self, dist: usize) -> Option<(Self, Self)> {
        println!("intersecting outputs...");
        let lmap = lhs.mapping;
        let lout = lmap.outgoing;
        let lmat = lmap.matrix;
        // find all edges input to left also input to right with respect to dist
        let rw = rhs.width();
        let rmap = rhs.mapping;
        let rout = rmap.outgoing;
        let rmat = rmap.matrix;
        let mut l = vec![false; lout.len()];
        let mut r = vec![false; rout.len()];
        let mut l = l.iter_mut().zip(lout.into_iter().zip(lmat.row_iter()));
        let mut r = r.iter_mut().zip(rout.into_iter().zip(rmat.row_iter()));
        for (lb, (le, _)) in &mut l {
            for (rb, (re, _)) in &mut r {
                let b = le.node == re.node && le.dist + dist + rw == re.dist;
                *rb = *rb || b;
                *lb = *lb || b;
            }
        }
        let (lout, lmat): (Vec<Edge>, Vec<_>) = l.filter_map(|(b, v)| b.then(|| v)).unzip();
        let (rout, rmat): (Vec<Edge>, Vec<_>) = r.filter_map(|(b, v)| b.then(|| v)).unzip();
        (!lout.is_empty()).then(|| ())?;
        (!rout.is_empty()).then(|| ())?;
        let lmat = EdgeMappingMatrix::from_rows(&lmat);
        let rmat = EdgeMappingMatrix::from_rows(&rmat);
        let mut lmap = EdgeMapping {
            outgoing: lout,
            matrix: lmat,
            ..lmap
        };
        let mut rmap = EdgeMapping {
            outgoing: rout,
            matrix: rmat,
            ..rmap
        };
        lmap.remove_zero_columns();
        rmap.remove_zero_columns();
        Some((
            Self {
                mapping: lmap,
                ..lhs
            },
            Self {
                mapping: rmap,
                ..rhs
            }
        ))
    }
    ///// Find mapping indices of input intersection at dist
    fn input_intersections(lhs: Self, rhs: Self, dist: usize) -> Option<(Self, Self)> {
        debug!("intersecting inputs...");
        debug!("lhs.token: {:?}", lhs.token);
        debug!("rhs.token: {:?}", rhs.token);
        let lw = lhs.width();
        let lmap = lhs.mapping;
        let lin = lmap.incoming;
        let lmat = lmap.matrix;
        // find all edges input to left also input to right with respect to dist
        let rmap = rhs.mapping;
        let rin = rmap.incoming;
        let rmat = rmap.matrix;

        debug!("lin: {:#?}", lin);
        debug!("rin: {:#?}", rin);
        debug!("Finding shared inputs...");
        let mut l = vec![false; lin.len()];
        let mut r = vec![false; rin.len()];
        let mut l = l.iter_mut().zip(lin.into_iter().zip(lmat.column_iter()));
        let mut r = r.iter_mut().zip(rin.into_iter().zip(rmat.column_iter()));
        for (lb, (le, _)) in &mut l {
            for (rb, (re, _)) in &mut r {
                let b = le.node == re.node && le.dist == re.dist + dist + lw;
                *rb = *rb || b;
                *lb = *lb || b;
            }
        }
        debug!("Filtering shared inputs...");
        let (lin, lmat): (Vec<Edge>, Vec<_>) = l.filter_map(|(b, v)| b.then(|| v)).unzip();
        let (rin, rmat): (Vec<Edge>, Vec<_>) = r.filter_map(|(b, v)| b.then(|| v)).unzip();
        debug!("Checking if inputs empty...");
        (!lin.is_empty()).then(|| ())?;
        (!rin.is_empty()).then(|| ())?;
        debug!("Building new matrices");
        let lmat = EdgeMappingMatrix::from_columns(&lmat);
        let rmat = EdgeMappingMatrix::from_columns(&rmat);
        let mut lmap = EdgeMapping {
            incoming: lin,
            matrix: lmat,
            ..lmap
        };
        let mut rmap = EdgeMapping {
            incoming: rin,
            matrix: rmat,
            ..rmap
        };
        debug!("Removing zero rows");
        lmap.remove_zero_rows();
        rmap.remove_zero_rows();
        debug!("Done.");
        Some((
            Self {
                mapping: lmap,
                ..lhs
            },
            Self {
                mapping: rmap,
                ..rhs
            }
        ))
    }
    /// Find mapping indices of connecting edges at dist
    fn connecting_intersections(lhs: Self, rhs: Self, dist: usize) -> Option<(Self, Self)> {
        println!("intersecting connections...");
        let lmap = lhs.mapping;
        let lout = lmap.outgoing;
        let lmat = lmap.matrix;
        // find all edges input to left also input to right with respect to dist
        let rmap = rhs.mapping;
        let rin = rmap.incoming;
        let rmat = rmap.matrix;

        let mut l = vec![false; lout.len()];
        let mut r = vec![false; rin.len()];
        let mut l = l.iter_mut().zip(lout.into_iter().zip(lmat.row_iter()));
        let mut r = r.iter_mut().zip(rin.into_iter().zip(rmat.column_iter()));
        for (lb, (le, _)) in &mut l {
            for (rb, (re, _)) in &mut r {
                let b = re.index == le.index && re.dist == dist;
                *rb = *rb || b;
                *lb = *lb || b;
            }
        }
        let (lout, lmat): (Vec<Edge>, Vec<_>) = l.filter_map(|(b, v)| b.then(|| v)).unzip();
        let (rin, rmat): (Vec<Edge>, Vec<_>) = r.filter_map(|(b, v)| b.then(|| v)).unzip();
        (!lout.is_empty()).then(|| ())?;
        (!rin.is_empty()).then(|| ())?;
        let lmat = EdgeMappingMatrix::from_rows(&lmat);
        let rmat = EdgeMappingMatrix::from_columns(&rmat);
        let mut lmap = EdgeMapping {
            outgoing: lout,
            matrix: lmat,
            ..lmap
        };
        let mut rmap = EdgeMapping {
            incoming: rin,
            matrix: rmat,
            ..rmap
        };
        lmap.remove_zero_columns();
        rmap.remove_zero_rows();
        Some((
            Self {
                mapping: lmap,
                ..lhs
            },
            Self {
                mapping: rmap,
                ..rhs
            }
        ))
    }
    ///// Join node from right with distance 1
    pub fn join_right(&self, rhs: &Self) -> Option<Self> {
        let lhs = self.clone();
        let rhs = rhs.clone();
        let (lhs, rhs) = Self::input_intersections(lhs, rhs, 1)?;
        let (lhs, rhs) = Self::output_intersections(lhs, rhs, 1)?;
        let (lhs, rhs) = Self::connecting_intersections(lhs, rhs, 1)?;
        let lmap = lhs.mapping;
        let rmap = rhs.mapping;
        let incoming = lmap.incoming;
        let outgoing = rmap.outgoing;
        let lmat = EdgeMappingMatrix::from_rows(&lmap.outgoing
            .into_iter()
            .map(|e| e.index)
            .zip(lmap.matrix.row_iter())
            .sorted_by(|(e1, _), (e2, _)| e1.cmp(e2))
            .map(|(_, v)| v)
            .collect::<Vec<_>>());
        let rmat = EdgeMappingMatrix::from_columns(&rmap.incoming
            .into_iter()
            .map(|e| e.index)
            .zip(rmap.matrix.column_iter())
            .sorted_by(|(e1, _), (e2, _)| e1.cmp(e2))
            .map(|(_, v)| v)
            .collect::<Vec<_>>());
        let matrix = rmat * lmat;
        Some(Self {
            mapping: EdgeMapping {
                incoming,
                outgoing,
                matrix,
            },
            token: lhs.token + rhs.token, // TODO
        })
    }
}
impl<T: TokenData> Wide for Node<T> {
    fn width(&self) -> usize {
        self.token.width()
    }
}
impl PartialEq<Node<char>> for char {
    fn eq(&self, rhs: &Node<char>) -> bool {
        *self == rhs.token
    }
}
impl PartialEq<Token<char>> for char {
    fn eq(&self, rhs: &Token<char>) -> bool {
        match rhs {
            Token::Element(e) => e == self,
            _ => false,
        }
    }
}
impl<T: TokenData> PartialEq<T> for Node<T> {
    fn eq(&self, rhs: &T) -> bool {
        self.token == *rhs
    }
}
impl<T: TokenData> PartialEq<T> for Token<T> {
    fn eq(&self, rhs: &T) -> bool {
        match self {
            Token::Element(e) => e == rhs,
            _ => false,
        }
    }
}
impl<T: TokenData> Debug for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.token)
    }
}
impl<T: TokenData + Display> Display for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.token)
    }
}
/// Trait for token that can be wrapped in a sequence
pub trait Sequencable: TokenData {
    fn sequenced<T: Into<Self>, I: Iterator<Item = T>>(seq: I) -> Vec<Token<Self>> {
        let mut v = vec![Token::Start];
        v.extend(seq.map(|t| Token::Element(t.into())));
        v.push(Token::End);
        v
    }
}
impl<T: TokenData + Into<Token<T>>> Sequencable for T {}
/// Trait for token that can be mapped in a sequence
pub trait Mappable: Sequencable + Wide {}
impl<T: TokenData + Wide> Mappable for T {}

pub trait Mapped: Wide {
    fn mapping(&self) -> &EdgeMapping;
    fn mapping_mut(&mut self) -> &mut EdgeMapping;
}
impl<T: TokenData> Mapped for Node<T> {
    fn mapping(&self) -> &EdgeMapping {
        &self.mapping
    }
    fn mapping_mut(&mut self) -> &mut EdgeMapping {
        &mut self.mapping
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{
        ELEMS,
        SEQS,
        EDGES,
        G,
    };
    use tracing_test::traced_test;
    #[traced_test]
    #[test]
    fn join_right() {
        let b_node = G.find_node_weight(&'b').unwrap();
        let c_node = G.find_node_weight(&'c').unwrap();
        let bc_node = b_node.join_right(&c_node).unwrap();
    }
}
