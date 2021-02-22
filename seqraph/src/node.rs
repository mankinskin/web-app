use crate::{
    SequenceGraph,
    token::{
        Token,
        TokenData,
        Wide,
    },
    mapping::{
        EdgeMappingMatrix,
        EdgeMapping,
        Mapped,
    },
};
use serde::{
    Deserialize,
    Serialize,
};
use petgraph::graph::{
    EdgeIndex,
    NodeIndex,
};
use itertools::Itertools;
use std::{
    iter::repeat,
    fmt::{
        self,
        Debug,
        Display,
    },
};
#[allow(unused)]
use tracing::{
    debug,
};

#[derive(PartialEq, Clone, Debug, Eq)]
pub struct Edge {
    pub index: EdgeIndex,
    pub node: NodeIndex,
    pub dist: usize,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo<T: TokenData> {
    pub element: Token<T>,
    pub incoming_groups: Vec<Vec<Token<T>>>,
    pub outgoing_groups: Vec<Vec<Token<T>>>,
}

/// Stores sequenced tokens with an edge map
#[derive(Clone, Eq)]
pub struct Node<T: TokenData> {
    token: Token<T>,
    mapping: EdgeMapping,
}
impl<T: TokenData> Node<T> {
    pub fn new(token: Token<T>) -> Self {
        Self {
            token,
            mapping: EdgeMapping::new(),
        }
    }
    pub fn token(&self) -> &Token<T> {
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
impl<T: TokenData> Mapped for Node<T> {
    fn mapping(&self) -> &EdgeMapping {
        &self.mapping
    }
    fn mapping_mut(&mut self) -> &mut EdgeMapping {
        &mut self.mapping
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
    pub fn token(&self) -> &Token<T> {
        &self.node.token()
    }
    fn intersections<
        L: Clone,
        R: Clone,
        W: Fn(&Node<T>, &Node<T>) -> usize,
        LE: Fn(Vec<Edge>, Vec<Edge>) -> (Vec<Edge>, Vec<Edge>),
        RE: Fn(Vec<Edge>, Vec<Edge>) -> (Vec<Edge>, Vec<Edge>),
        LMI: Fn(EdgeMappingMatrix) -> Vec<L>,
        RMI: Fn(EdgeMappingMatrix) -> Vec<R>,
        P: Fn(NodeIndex, NodeIndex, NodeIndex, NodeIndex, usize, usize, usize, usize) -> bool,
        LC: Fn(Vec<L>) -> EdgeMappingMatrix,
        RC: Fn(Vec<R>) -> EdgeMappingMatrix,
        LMC: Fn(Vec<Edge>, EdgeMappingMatrix, Vec<Edge>) -> EdgeMapping,
        RMC: Fn(Vec<Edge>, EdgeMappingMatrix, Vec<Edge>) -> EdgeMapping,
        LZ: Fn(&mut EdgeMapping),
        RZ: Fn(&mut EdgeMapping),
    >(
        #[allow(unused)]
        name: &str,
        #[allow(unused)]
        sec_name: &str,
        w_sel: W,
        ledge_sel: LE,
        redge_sel: RE,
        lmat_iter: LMI,
        rmat_iter: RMI,
        pred: P,
        lconstr: LC,
        rconstr: RC,
        lmap_ctr: LMC,
        rmap_ctr: RMC,
        ldezero: LZ,
        rdezero: RZ,
        lhs: Self,
        rhs: Self,
        dist: usize,
    ) -> Option<(Self, Self)> {
        let ln = lhs.node;
        let rn = rhs.node;
        let li = lhs.index;
        let ri = rhs.index;
        debug!("intersecting {}...", name);
        debug!("lhs.token: {:?}", ln.token);
        debug!("rhs.token: {:?}", rn.token);
        let w = w_sel(&ln, &rn);
        let lmap = ln.mapping;
        let rmap = rn.mapping;
        let (lprim, lsec) = ledge_sel(lmap.incoming, lmap.outgoing);
        let (rprim, rsec) = redge_sel(rmap.incoming, rmap.outgoing);
        debug!("lprim: {:#?}", lprim);
        debug!("rprim: {:#?}", rprim);
        let lprim_mat_iter = lmat_iter(lmap.matrix);
        let rprim_mat_iter = rmat_iter(rmap.matrix);
        debug!("Finding shared {}...", name);
        let mut l = repeat(false)
            .take(lprim.len())
            .zip(lprim.into_iter().zip(lprim_mat_iter))
            .collect::<Vec<_>>();
        let mut r = repeat(false)
            .take(rprim.len())
            .zip(rprim.into_iter().zip(rprim_mat_iter))
            .collect::<Vec<_>>();
        for (lb, (le, _)) in &mut l {
            for (rb, (re, _)) in &mut r {
                if pred(li, ri, le.node, re.node, le.dist, re.dist, dist, w) {
                    debug!("{}: {:?} {} {}", name, le.node, le.dist, re.dist);
                    *rb = true;
                    *lb = true;
                }
            }
        }
        debug!("Filtering shared {}...", name);
        let (lprim, lms): (Vec<Edge>, Vec<_>) = l.into_iter().filter_map(|(b, (e, m))| b.then(|| (e, m))).unzip();
        let (rprim, rms): (Vec<Edge>, Vec<_>) = r.into_iter().filter_map(|(b, (e, m))| b.then(|| (e, m))).unzip();
        debug!("Checking if {} empty...", name);
        debug!("lprim: {:#?}", lprim);
        debug!("rprim: {:#?}", rprim);
        (!lprim.is_empty()).then(|| ())?;
        (!rprim.is_empty()).then(|| ())?;
        debug!("Building new matrices");
        let lmat = lconstr(lms);
        let rmat = rconstr(rms);
        let mut lmap = lmap_ctr(lprim, lmat, lsec);
        let mut rmap = rmap_ctr(rprim, rmat, rsec);
        debug!("Removing zero {}", sec_name);
        ldezero(&mut lmap);
        rdezero(&mut rmap);
        debug!("Done.");
        Some((
            Self::of(li, Node {
                mapping: lmap,
                ..ln
            }),
            Self::of(ri, Node {
                mapping: rmap,
                ..rn
            })
        ))
    }
    fn input_intersections(lhs: Self, rhs: Self, dist: usize) -> Option<(Self, Self)> {
        Self::intersections("inputs", "rows",
            |ln, _| ln.width(),
            |inc, out| (inc, out),
            |inc, out| (inc, out),
            |mat| mat.column_iter().map(|m| m.into_owned()).collect(),
            |mat| mat.column_iter().map(|m| m.into_owned()).collect(),
            |li, _, ln, rn, ld, rd, d, w| li == rn && rd == d || ln == rn && ld + d + w == rd + 1,
            |it| EdgeMappingMatrix::from_columns(&it),
            |it| EdgeMappingMatrix::from_columns(&it),
            |prim, matrix, sec| EdgeMapping {
                incoming: prim,
                matrix,
                outgoing: sec
            },
            |prim, matrix, sec| EdgeMapping {
                incoming: prim,
                matrix,
                outgoing: sec
            },
            |map| map.remove_zero_rows(),
            |map| map.remove_zero_rows(),
            lhs, rhs, dist)
    }
    fn output_intersections(lhs: Self, rhs: Self, dist: usize) -> Option<(Self, Self)> {
        Self::intersections("outputs", "columns",
            |_, rn| rn.width(),
            |inc, out| (out, inc),
            |inc, out| (out, inc),
            |mat| mat.row_iter().map(|m| m.into_owned()).collect(),
            |mat| mat.row_iter().map(|m| m.into_owned()).collect(),
            |_, ri, ln, rn, ld, rd, d, w| ln == ri && ld == d || ln == rn && ld + 1 == rd + d + w,
            |it| EdgeMappingMatrix::from_rows(&it),
            |it| EdgeMappingMatrix::from_rows(&it),
            |prim, matrix, sec| EdgeMapping {
                outgoing: prim,
                matrix,
                incoming: sec,
            },
            |prim, matrix, sec| EdgeMapping {
                outgoing: prim,
                matrix,
                incoming: sec,
            },
            |map| map.remove_zero_columns(),
            |map| map.remove_zero_columns(),
            lhs, rhs, dist)
    }
    fn connecting_intersections(lhs: Self, rhs: Self, dist: usize) -> Option<(Self, Self)> {
        Self::intersections("connections", "rows & columns",
            |ln, _| ln.width(),
            |inc, out| (out, inc),
            |inc, out| (inc, out),
            |mat| mat.row_iter().map(|m| m.into_owned()).collect(),
            |mat| mat.column_iter().map(|m| m.into_owned()).collect(),
            |li, ri, ln, rn, ld, rd, d, _| ln == ri && rn == li && ld == d && rd == d,
            |it| EdgeMappingMatrix::from_rows(&it),
            |it| EdgeMappingMatrix::from_columns(&it),
            |prim, matrix, sec| EdgeMapping {
                incoming: sec,
                matrix,
                outgoing: prim
            },
            |prim, matrix, sec| EdgeMapping {
                incoming: prim,
                matrix,
                outgoing: sec
            },
            |map| map.remove_zero_columns(),
            |map| map.remove_zero_rows(),
            lhs, rhs, dist)
    }
    ///// Join node from right with distance 1
    fn try_join_right(&self, rhs: &Self) -> Option<Node<T>> {
        let lhs = self.clone();
        let rhs = rhs.clone();
        let (lhs, rhs) = Self::input_intersections(lhs, rhs, 1)?;
        let (lhs, rhs) = Self::output_intersections(lhs, rhs, 1)?;
        let (lhs, rhs) = Self::connecting_intersections(lhs, rhs, 1)?;
        let ln = lhs.node;
        let rn = rhs.node;
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
        Some(Node {
            mapping: EdgeMapping {
                incoming: lmap.incoming,
                outgoing: rmap.outgoing,
                matrix: rmat * lmat,
            },
            token: ln.token + rn.token,
        })
    }
    ///// Join node from right with distance 1
    pub fn join_right(&self, rhs: &Self) -> Node<T> {
        self.try_join_right(rhs)
            .unwrap_or_else(|| Node::new(self.token() + rhs.token()))
    }
}
impl<T: TokenData> Wide for LoadedNode<T> {
    fn width(&self) -> usize {
        self.node.width()
    }
}
impl<T: TokenData> Mapped for LoadedNode<T> {
    fn mapping(&self) -> &EdgeMapping {
        self.node.mapping()
    }
    fn mapping_mut(&mut self) -> &mut EdgeMapping {
        self.node.mapping_mut()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use crate::{
        SequenceGraph,   
        mapping::Mapped,
    };
    use petgraph::graph::NodeIndex;
    #[allow(unused)]
    use tracing::{
        debug,
    };
    use tracing_test::traced_test;
    use maplit::hashset;
    lazy_static::lazy_static! {
        pub static ref SEQS: Vec<&'static str> = Vec::from([
            "bc",
            "aa",
            "abc",
            //"bcade",
            //"aa",
            //"bcaade",
            //"bcbcabc",
            //"abcaa",
        ]);
        pub static ref G: SequenceGraph<char> = {
            let mut g = SequenceGraph::new();
            for &s in SEQS.iter() {
                g.read_sequence(s.chars());
            }
            g
        };
    }
    #[traced_test]
    #[test]
    fn join_bc() {
        debug!("{:#?}", G.node_indices().zip(G.all_node_weights()).collect::<Vec<_>>());
        let b_node = G.load_node('b').unwrap();
        let c_node = G.load_node('c').unwrap();
        let bc_node = b_node.join_right(&c_node);

        assert_eq!(bc_node.token(), b_node.token() + c_node.token());
        let m = bc_node.mapping(); 
        assert_eq!(
            (m.incoming_sources().collect(), m.outgoing_targets().collect()),
            (hashset![
                (1, NodeIndex::new(0)),
                (1, NodeIndex::new(4)),
                (2, NodeIndex::new(0)),
            ],
            hashset![
                (1, NodeIndex::new(3)),
            ])
        );
        //debug!("{:#?}", _bc_node.get_info(&G));
    }
    #[traced_test]
    #[test]
    fn join_aa() {
        let a_node = G.load_node('a').unwrap();
        let aa_node = a_node.join_right(&a_node);
        assert_eq!(aa_node.token(), a_node.token() + a_node.token());
        let m = aa_node.mapping(); 
        assert_eq!(
            (m.incoming_sources().collect(), m.outgoing_targets().collect()),
            (vec![
                (1, NodeIndex::new(0)),
            ],
            vec![
                (1, NodeIndex::new(3)),
            ])
        );
        //debug!("{:#?}", _bc_node.get_info(&G));
    }
    #[traced_test]
    #[test]
    fn join_ba() {
        let b_node = G.load_node('b').unwrap();
        let a_node = G.load_node('a').unwrap();
        let ba_node = b_node.join_right(&a_node);
        assert_eq!(ba_node.token(), b_node.token() + a_node.token());
        let m = ba_node.mapping(); 
        assert_eq!(
            (m.incoming_sources().collect(), m.outgoing_targets().collect()),
            (vec![], vec![])
        );
        //debug!("{:#?}", _bc_node.get_info(&G));
    }
    //#[traced_test]
    //#[test]
    //fn join_abc() {
    //    let a_node = G.load_node('a').unwrap();
    //    let b_node = G.load_node('b').unwrap();
    //    let c_node = G.load_node('c').unwrap();
    //    let ab_node = a_node.join_right(&b_node);
    //    //let abc_node = ab_node.join_right(&c_node);
    //    assert_eq!(ab_node.token(), a_node.token() + b_node.token());
    //    //let _abc_node = _ab_node.join_right(&c_node).unwrap();
    //    //debug!("{:#?}", _bc_node.get_info(&G));
    //}
}
