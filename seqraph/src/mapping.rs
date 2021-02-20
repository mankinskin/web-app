use crate::{
    graph::{
        node::NodeData,
        edge::EdgeData,
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
    default::Default,
    iter::repeat,
    fmt::{
        self,
        Formatter,
        Debug,
        Display,
    },
};
#[allow(unused)]
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
#[derive(PartialEq, Clone, Debug, Eq)]
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
#[derive(PartialEq, Clone, Debug, Eq)]
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
        if let Some(i) = self.incoming.iter().position(|e| e.index == edge.index) {
            //debug!("Incoming edge already exists {:#?}", edge);
            i
        } else {
            let index = self.incoming.len();
            //debug!("Adding new incoming edge {:#?} with index {}", edge, index);
            self.incoming.push(edge);
            //debug!("{:#?}", self.incoming);
            self.matrix = self
                .matrix
                .clone()
                .insert_column(self.matrix.ncols(), false.into());
            index
        }
    }
    /// Add an outgoing edge
    fn add_outgoing_edge(&mut self, edge: Edge) -> usize {
        if let Some(i) = self.outgoing.iter().position(|e| e.index == edge.index) {
            //debug!("Outgoing edge already exists {:#?}", edge);
            i
        } else {
            let index = self.outgoing.len();
            //debug!("Adding new outgoing edge {:#?} with index {}", edge, index);
            self.outgoing.push(edge);
            //debug!("{:#?}", self.outgoing);
            self.matrix = self
                .matrix
                .clone()
                .insert_row(self.matrix.nrows(), false.into());
            index
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
    /// Get weights and sources of incoming edges
    pub fn incoming_sources<T: NodeData, E: EdgeData>(
        &'a self,
        graph: &'a Graph<T, E>,
    ) -> impl Iterator<Item = (E, NodeIndex)> + 'a {
        graph
            .edge_weights(self.incoming.iter().map(|e| e.index))
            .zip(graph.edge_sources(self.incoming.iter().map(|e| e.index)))
    }
    /// Get weights and targets of outgoing edges
    pub fn outgoing_targets<T: NodeData, E: EdgeData>(
        &'a self,
        graph: &'a Graph<T, E>,
    ) -> impl Iterator<Item = (E, NodeIndex)> + 'a {
        graph
            .edge_weights(self.outgoing.iter().map(|e| e.index))
            .zip(graph.edge_targets(self.outgoing.iter().map(|e| e.index)))
    }

    /// Get distance groups for incoming edges
    pub fn incoming_distance_groups<T: NodeData + Wide>(
        &self,
        graph: &SequenceGraph<T>,
    ) -> Vec<Vec<Node<T>>> {
        graph.distance_group_source_weights(self.incoming.iter().map(|e| e.index))
    }
    /// Get distance groups for outgoing edges
    pub fn outgoing_distance_groups<T: NodeData + Wide>(
        &self,
        graph: &SequenceGraph<T>,
    ) -> Vec<Vec<Node<T>>> {
        graph.distance_group_target_weights(self.outgoing.iter().map(|e| e.index))
    }
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo<T: TokenData> {
    pub element: Token<T>,
    pub incoming_groups: Vec<Vec<Token<T>>>,
    pub outgoing_groups: Vec<Vec<Token<T>>>,
}

pub trait TokenData: NodeData + Wide {}
impl<T: NodeData + Wide> TokenData for T {}

/// Type for storing elements of a sequence
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq)]
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
impl<T: TokenData> PartialEq<T> for Token<T> {
    fn eq(&self, rhs: &T) -> bool {
        match self {
            Token::Element(e) => *e == *rhs,
            _ => false,
        }
    }
}
impl<T: TokenData> PartialEq<Node<T>> for Token<T> {
    fn eq(&self, rhs: &Node<T>) -> bool {
        *self == rhs.token
    }
}
impl PartialEq<Token<char>> for char {
    fn eq(&self, rhs: &Token<char>) -> bool {
        *rhs == *self
    }
}
/// Stores sequenced tokens with an edge map
#[derive(Clone, Eq)]
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
    #[allow(unused)]
    fn groups_to_string(groups: Vec<Vec<Self>>) -> String {
        let mut lines = Vec::new();
        let max = groups.iter().map(Vec::len).max().unwrap_or(0);
        for i in 0..max {
            let mut line = Vec::new();
            for group in &groups {
                line.push(group.get(i).map(ToString::to_string));
            }
            lines.push(line);
        }
        lines.iter().fold(String::new(), |a, line| {
            format!(
                "{}{}\n",
                a,
                line.iter().fold(String::new(), |a, elem| {
                    format!("{}{} ", a, elem.clone().unwrap_or(String::new()))
                })
            )
        })
    }
    fn map_to_tokens(groups: Vec<Vec<Node<T>>>) -> Vec<Vec<Token<T>>> {
        groups
            .iter()
            .map(|g| g.iter().map(|m| m.token.clone()).collect())
            .collect()
    }
    pub fn get_info(&self, graph: &SequenceGraph<T>) -> NodeInfo<T> {
        let mut incoming_groups: Vec<Vec<Node<T>>> = self.mapping.incoming_distance_groups(graph);
        incoming_groups.reverse();
        let outgoing_groups: Vec<Vec<Node<T>>> = self.mapping.outgoing_distance_groups(graph);
        NodeInfo {
            element: self.token.clone(),
            incoming_groups: Self::map_to_tokens(incoming_groups),
            outgoing_groups: Self::map_to_tokens(outgoing_groups),
        }
    }
}
impl<T: TokenData> Wide for Node<T> {
    fn width(&self) -> usize {
        self.token.width()
    }
}
impl<T: TokenData> PartialEq<T> for Node<T> {
    fn eq(&self, rhs: &T) -> bool {
        self.token == *rhs
    }
}
impl<T: TokenData> PartialEq<Token<T>> for Node<T> {
    fn eq(&self, rhs: &Token<T>) -> bool {
        self.token == *rhs
    }
}
impl<T: TokenData> PartialEq<Node<T>> for Node<T> {
    fn eq(&self, rhs: &Node<T>) -> bool {
        self.token == *rhs
    }
}
impl<T: TokenData> PartialEq<Node<T>> for &Node<T> {
    fn eq(&self, rhs: &Node<T>) -> bool {
        self.token == *rhs
    }
}
impl PartialEq<Node<char>> for char {
    fn eq(&self, rhs: &Node<char>) -> bool {
        *self == rhs.token
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
/// Stores sequenced tokens with an edge map
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LoadedNode<T: TokenData> {
    node: Node<T>,
    index: NodeIndex,
}
impl<T: TokenData> LoadedNode<T> {
    pub fn of(index: NodeIndex, node: Node<T>) -> Self {
        Self {
            index,
            node,
        }
    }
    fn output_intersections(lhs: Self, rhs: Self, dist: usize) -> Option<(Node<T>, Node<T>)> {
        let ln = lhs.node;
        let rn = rhs.node;
        debug!("intersecting outputs...");
        debug!("lhs.token: {:?}", ln.token);
        debug!("rhs.token: {:?}", rn.token);
        let lmap = ln.mapping;
        let lout = lmap.outgoing;
        let lmat = lmap.matrix;
        // find all edges input to left also input to right with respect to dist
        let rw = rn.width();
        let rmap = rn.mapping;
        let rout = rmap.outgoing;
        let rmat = rmap.matrix;
        debug!("lout: {:#?}", lout);
        debug!("rout: {:#?}", rout);
        debug!("Finding shared outputs...");
        let mut l = repeat(false).take(lout.len()).zip(lout.into_iter().zip(lmat.row_iter())).collect::<Vec<_>>();
        let mut r = repeat(false).take(rout.len()).zip(rout.into_iter().zip(rmat.row_iter())).collect::<Vec<_>>();
        for (lb, (le, _)) in &mut l {
            if le.dist == dist && le.node == rhs.index {
                *lb = true;
                continue;
            }
            for (rb, (re, _)) in &mut r {
                let b = le.node == re.node && le.dist == re.dist + dist + rw - 1;
                *rb = *rb || b;
                *lb = *lb || b;
            }
        }
        debug!("Filtering shared outputs...");
        let (lout, lmat): (Vec<Edge>, Vec<_>) = l.into_iter().filter_map(|(b, v)| b.then(|| v)).unzip();
        let (rout, rmat): (Vec<Edge>, Vec<_>) = r.into_iter().filter_map(|(b, v)| b.then(|| v)).unzip();
        debug!("Checking if outputs empty...");
        debug!("lout: {:#?}", lout);
        debug!("rout: {:#?}", rout);
        (!lout.is_empty()).then(|| ())?;
        (!rout.is_empty()).then(|| ())?;
        debug!("Building new matrices");
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
        debug!("Removing zero columns");
        lmap.remove_zero_columns();
        rmap.remove_zero_columns();
        debug!("Done.");
        Some((
            Node {
                mapping: lmap,
                ..ln
            },
            Node {
                mapping: rmap,
                ..rn
            }
        ))
    }
    ///// Find mapping indices of input intersection at dist
    fn input_intersections(lhs: Self, rhs: Self, dist: usize) -> Option<(Node<T>, Node<T>)> {
        let ln = lhs.node;
        let rn = rhs.node;
        debug!("intersecting inputs...");
        debug!("lhs.node.token: {:?}", ln.token);
        debug!("rhs.node.token: {:?}", rn.token);
        let lw = ln.width();
        let lmap = ln.mapping;
        let lin = lmap.incoming;
        let lmat = lmap.matrix;
        // find all edges input to left also input to right with respect to dist
        let rmap = rn.mapping;
        let rin = rmap.incoming;
        let rmat = rmap.matrix;

        debug!("lin: {:#?}", lin);
        debug!("rin: {:#?}", rin);
        debug!("Finding shared inputs...");
        let mut l = repeat(false).take(lin.len()).zip(lin.into_iter().zip(lmat.column_iter())).collect::<Vec<_>>();
        let mut r = repeat(false).take(rin.len()).zip(rin.into_iter().zip(rmat.column_iter())).collect::<Vec<_>>();
        for (rb, (re, _)) in &mut r {
            if re.dist == dist && re.node == lhs.index {
                *rb = true;
                continue;
            }
            for (lb, (le, _)) in &mut l {
                debug!("le.dist: {}", le.dist);
                debug!("re.dist: {}", re.dist);
                let b = le.node == re.node && le.dist + dist + lw - 1 == re.dist;
                debug!("b: {}", b);
                *rb = *rb || b;
                *lb = *lb || b;
            }
        }
        debug!("Filtering shared inputs...");
        let (lin, lmat): (Vec<Edge>, Vec<_>) = l.into_iter().filter_map(|(b, v)| b.then(|| v)).unzip();
        let (rin, rmat): (Vec<Edge>, Vec<_>) = r.into_iter().filter_map(|(b, v)| b.then(|| v)).unzip();
        debug!("Checking if inputs empty...");
        debug!("lin: {:#?}", lin);
        debug!("rin: {:#?}", rin);
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
            Node {
                mapping: lmap,
                ..ln
            },
            Node {
                mapping: rmap,
                ..rn
            }
        ))
    }
    ///// Find mapping indices of connecting edges at dist
    //fn connecting_intersections(lhs: Self, rhs: Self, dist: usize) -> Option<(Node<T>, Node<T>)> {
    //    let ln = lhs.node;
    //    let rn = rhs.node;
    //    debug!("intersecting connections...");
    //    debug!("lhs.node.token: {:?}", ln.token);
    //    debug!("rhs.node.token: {:?}", rn.token);
    //    let lmap = ln.mapping;
    //    let lout = lmap.outgoing;
    //    let lmat = lmap.matrix;
    //    // find all edges input to left also input to right with respect to dist
    //    let rmap = rn.mapping;
    //    let rin = rmap.incoming;
    //    let rmat = rmap.matrix;

    //    debug!("lout: {:#?}", lout);
    //    debug!("rin: {:#?}", rin);
    //    debug!("Finding connections...");
    //    let mut l = repeat(false).take(lout.len()).zip(lout.into_iter().zip(lmat.row_iter())).collect::<Vec<_>>();
    //    let mut r = repeat(false).take(rin.len()).zip(rin.into_iter().zip(rmat.column_iter())).collect::<Vec<_>>();
    //    for (lb, (le, _)) in &mut l {
    //        for (rb, (re, _)) in &mut r {
    //            let b = re.index == le.index && re.dist == dist;
    //            *rb = *rb || b;
    //            *lb = *lb || b;
    //        }
    //    }
    //    debug!("Filtering connections...");
    //    let (lout, lmat): (Vec<Edge>, Vec<_>) = l.into_iter().filter_map(|(b, v)| b.then(|| v)).unzip();
    //    let (rin, rmat): (Vec<Edge>, Vec<_>) = r.into_iter().filter_map(|(b, v)| b.then(|| v)).unzip();
    //    debug!("Checking if connections empty...");
    //    debug!("lout: {:#?}", lout);
    //    debug!("rin: {:#?}", rin);
    //    (!lout.is_empty()).then(|| ())?;
    //    (!rin.is_empty()).then(|| ())?;
    //    debug!("Building new matrices");
    //    let lmat = EdgeMappingMatrix::from_rows(&lmat);
    //    let rmat = EdgeMappingMatrix::from_columns(&rmat);
    //    let mut lmap = EdgeMapping {
    //        outgoing: lout,
    //        matrix: lmat,
    //        ..lmap
    //    };
    //    let mut rmap = EdgeMapping {
    //        incoming: rin,
    //        matrix: rmat,
    //        ..rmap
    //    };
    //    debug!("Removing left zero columns and right zero rows");
    //    lmap.remove_zero_columns();
    //    rmap.remove_zero_rows();
    //    debug!("Done.");
    //    Some((
    //        Node {
    //            mapping: lmap,
    //            ..ln
    //        },
    //        Node {
    //            mapping: rmap,
    //            ..rn
    //        }
    //    ))
    //}
    ///// Join node from right with distance 1
    pub fn join_right(&self, rhs: &Self) -> Option<Node<T>> {
        let lhs = self.clone();
        let rhs = rhs.clone();
        let li = lhs.index.clone();
        let ri = rhs.index.clone();
        let (ln, rn) = Self::input_intersections(lhs, rhs, 1)?;
        let lhs = Self::of(li, ln);
        let rhs = Self::of(ri, rn);
        let (ln, rn) = Self::output_intersections(lhs, rhs, 1)?;
        let lmap = ln.mapping;
        let rmap = rn.mapping;
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
        let incoming = lmap.incoming;
        let outgoing = rmap.outgoing;
        let matrix = rmat * lmat;
        Some(Node {
            mapping: EdgeMapping {
                incoming,
                outgoing,
                matrix,
            },
            token: ln.token + rn.token, // TODO
        })
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
    #[allow(unused)]
    use tracing::{
        debug,
    };
    use crate::tests::{
        G,
    };
    use tracing_test::traced_test;
    #[traced_test]
    #[test]
    fn join_right() {
        let b_node = G.load_node('b').unwrap();
        let c_node = G.load_node('c').unwrap();
        let bc_node = b_node.join_right(&c_node).unwrap();
        debug!("{:#?}", bc_node.get_info(&G));
    }
}
