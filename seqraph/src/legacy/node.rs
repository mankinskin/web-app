use crate::{
    mapping::{
        EdgeMapping,
        EdgeMappingMatrix,
        LoadedEdge,
        LoadedEdgeMapping,
        Edge,
    },
    token::{
        Token,
        TokenContext,
        Tokenize,
        Wide,
    },
};
use itertools::Itertools;
use petgraph::graph::{
    NodeIndex,
};
use std::{
    fmt::{
        self,
        Debug,
        Display,
    },
    iter::repeat,
};
use tracing::debug;

/// Stores sequenced tokens with an edge map
#[derive(Clone, Eq)]
pub struct Node<T: Tokenize> {
    pub(crate) token: Token<T>,
    pub(crate) mapping: EdgeMapping,
}
impl<T: Tokenize> Node<T> {
    pub fn new(token: Token<T>) -> Self {
        Self {
            token,
            mapping: EdgeMapping::new(),
        }
    }
}
impl<T: Tokenize> TokenContext<T, Edge> for Node<T> {
    type Mapping = EdgeMapping;
    fn token(&self) -> &Token<T> {
        &self.token
    }
    fn into_token(self) -> Token<T> {
        self.token
    }
    fn mapping(&self) -> &Self::Mapping {
        &self.mapping
    }
    fn mapping_mut(&mut self) -> &mut Self::Mapping {
        &mut self.mapping
    }
}
impl<T: Tokenize> Wide for Node<T> {
    fn width(&self) -> usize {
        self.token.width()
    }
}
impl<T: Tokenize> PartialEq<T> for Node<T> {
    fn eq(&self, rhs: &T) -> bool {
        self.token == *rhs
    }
}
impl<T: Tokenize> PartialEq<Token<T>> for Node<T> {
    fn eq(&self, rhs: &Token<T>) -> bool {
        self.token == *rhs
    }
}
impl<T: Tokenize> PartialEq<Node<T>> for Node<T> {
    fn eq(&self, rhs: &Node<T>) -> bool {
        self.token == *rhs
    }
}
impl<T: Tokenize> PartialEq<Node<T>> for &Node<T> {
    fn eq(&self, rhs: &Node<T>) -> bool {
        self.token == *rhs
    }
}
impl PartialEq<Node<char>> for char {
    fn eq(&self, rhs: &Node<char>) -> bool {
        *self == rhs.token
    }
}
impl<T: Tokenize> Debug for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.token)
    }
}
impl<T: Tokenize + Display> Display for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.token)
    }
}
/// Stores sequenced tokens with an edge map
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LoadedNode<T: Tokenize> {
    pub(crate) token: Token<T>,
    pub(crate) index: NodeIndex,
    pub(crate) mapping: LoadedEdgeMapping,
}
impl<T: Tokenize> TokenContext<T, LoadedEdge> for LoadedNode<T> {
    type Mapping = LoadedEdgeMapping;
    fn token(&self) -> &Token<T> {
        &self.token
    }
    fn into_token(self) -> Token<T> {
        self.token
    }
    fn mapping(&self) -> &Self::Mapping {
        &self.mapping
    }
    fn mapping_mut(&mut self) -> &mut Self::Mapping {
        &mut self.mapping
    }
}
impl<T: Tokenize> Wide for LoadedNode<T> {
    fn width(&self) -> usize {
        self.token.width()
    }
}
impl<T: Tokenize> LoadedNode<T> {
    pub fn new(index: NodeIndex, token: Token<T>, mapping: LoadedEdgeMapping) -> Self {
        Self {
            index,
            token,
            mapping,
        }
    }
    fn intersections<
        L, R,
        W: Fn(&Token<T>, &Token<T>) -> usize,
        LE: Fn(Vec<LoadedEdge>, Vec<LoadedEdge>) -> (Vec<LoadedEdge>, Vec<LoadedEdge>),
        RE: Fn(Vec<LoadedEdge>, Vec<LoadedEdge>) -> (Vec<LoadedEdge>, Vec<LoadedEdge>),
        LMI: Fn(EdgeMappingMatrix) -> Vec<L>,
        RMI: Fn(EdgeMappingMatrix) -> Vec<R>,
        P: Fn(NodeIndex, NodeIndex, NodeIndex, NodeIndex, usize, usize, usize, usize) -> bool,
        LC: Fn(Vec<L>) -> EdgeMappingMatrix,
        RC: Fn(Vec<R>) -> EdgeMappingMatrix,
        LMC: Fn(Vec<LoadedEdge>, EdgeMappingMatrix, Vec<LoadedEdge>) -> LoadedEdgeMapping,
        RMC: Fn(Vec<LoadedEdge>, EdgeMappingMatrix, Vec<LoadedEdge>) -> LoadedEdgeMapping,
        LZ: Fn(&mut LoadedEdgeMapping),
        RZ: Fn(&mut LoadedEdgeMapping),
    >(
        #[allow(unused)] name: &str,
        #[allow(unused)] sec_name: &str,
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
        let li = lhs.index;
        let ri = rhs.index;
        //debug!("intersecting {}...", name);
        //debug!("lhs.token: {:?}", lhs.token);
        //debug!("rhs.token: {:?}", rhs.token);
        let w = w_sel(&lhs.token, &rhs.token);
        let lmap = lhs.mapping;
        let rmap = rhs.mapping;
        let (lprim, lsec) = ledge_sel(lmap.incoming, lmap.outgoing);
        let (rprim, rsec) = redge_sel(rmap.incoming, rmap.outgoing);
        //debug!("lprim: {:#?}", lprim);
        //debug!("rprim: {:#?}", rprim);
        let lprim_mat_iter = lmat_iter(lmap.matrix);
        let rprim_mat_iter = rmat_iter(rmap.matrix);
        //debug!("Finding shared {}...", name);
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
                    //debug!("{}: {:?} {} {}", name, le.node, le.dist, re.dist);
                    *rb = true;
                    *lb = true;
                }
            }
        }
        //debug!("Filtering shared {}...", name);
        let (lprim, lms): (Vec<LoadedEdge>, Vec<_>) = l
            .into_iter()
            .filter_map(|(b, (e, m))| b.then(|| (e, m)))
            .unzip();
        let (rprim, rms): (Vec<LoadedEdge>, Vec<_>) = r
            .into_iter()
            .filter_map(|(b, (e, m))| b.then(|| (e, m)))
            .unzip();
        //debug!("Checking if {} empty...", name);
        //debug!("lprim: {:#?}", lprim);
        //debug!("rprim: {:#?}", rprim);
        (!lprim.is_empty()).then(|| ())?;
        (!rprim.is_empty()).then(|| ())?;
        //debug!("Building new matrices");
        let lmat = lconstr(lms);
        let rmat = rconstr(rms);
        let mut lmap = lmap_ctr(lprim, lmat, lsec);
        let mut rmap = rmap_ctr(rprim, rmat, rsec);
        //debug!("Removing zero {}", sec_name);
        ldezero(&mut lmap);
        rdezero(&mut rmap);
        //debug!("Done.");
        Some((
            Self {
                index: li,
                mapping: lmap,
                ..lhs
            },
            Self {
                index: ri,
                mapping: rmap,
                ..rhs
            },
        ))
    }
    fn input_intersections(lhs: Self, rhs: Self, dist: usize) -> Option<(Self, Self)> {
        Self::intersections(
            "inputs",
            "rows",
            |lt, _| lt.width(),
            |inc, out| (inc, out),
            |inc, out| (inc, out),
            |mat| mat.column_iter().map(|m| m.into_owned()).collect(),
            |mat| mat.column_iter().map(|m| m.into_owned()).collect(),
            |li, _, ln, rn, ld, rd, d, w| li == rn && rd == d || ln == rn && ld + d + w == rd + 1,
            |it| EdgeMappingMatrix::from_columns(&it),
            |it| EdgeMappingMatrix::from_columns(&it),
            |prim, matrix, sec| {
                LoadedEdgeMapping {
                    incoming: prim,
                    matrix,
                    outgoing: sec,
                }
            },
            |prim, matrix, sec| {
                LoadedEdgeMapping {
                    incoming: prim,
                    matrix,
                    outgoing: sec,
                }
            },
            |map| map.remove_zero_rows(),
            |map| map.remove_zero_rows(),
            lhs,
            rhs,
            dist,
        )
    }
    fn output_intersections(lhs: Self, rhs: Self, dist: usize) -> Option<(Self, Self)> {
        Self::intersections(
            "outputs",
            "columns",
            |_, rt| rt.width(),
            |inc, out| (out, inc),
            |inc, out| (out, inc),
            |mat| mat.row_iter().map(|m| m.into_owned()).collect(),
            |mat| mat.row_iter().map(|m| m.into_owned()).collect(),
            |_, ri, ln, rn, ld, rd, d, w| ln == ri && ld == d || ln == rn && ld + 1 == rd + d + w,
            |it| EdgeMappingMatrix::from_rows(&it),
            |it| EdgeMappingMatrix::from_rows(&it),
            |prim, matrix, sec| {
                LoadedEdgeMapping {
                    outgoing: prim,
                    matrix,
                    incoming: sec,
                }
            },
            |prim, matrix, sec| {
                LoadedEdgeMapping {
                    outgoing: prim,
                    matrix,
                    incoming: sec,
                }
            },
            |map| map.remove_zero_columns(),
            |map| map.remove_zero_columns(),
            lhs,
            rhs,
            dist,
        )
    }
    fn connecting_intersections(lhs: Self, rhs: Self, dist: usize) -> Option<(Self, Self)> {
        Self::intersections(
            "connections",
            "rows & columns",
            |_, _| 0,
            |inc, out| (out, inc),
            |inc, out| (inc, out),
            |mat| mat.row_iter().map(|m| m.into_owned()).collect(),
            |mat| mat.column_iter().map(|m| m.into_owned()).collect(),
            |li, ri, ln, rn, ld, rd, d, _| ln == ri && rn == li && ld == d && rd == d,
            |it| EdgeMappingMatrix::from_rows(&it),
            |it| EdgeMappingMatrix::from_columns(&it),
            |prim, matrix, sec| {
                LoadedEdgeMapping {
                    incoming: sec,
                    matrix,
                    outgoing: prim,
                }
            },
            |prim, matrix, sec| {
                LoadedEdgeMapping {
                    incoming: prim,
                    matrix,
                    outgoing: sec,
                }
            },
            |map| map.remove_zero_columns(),
            |map| map.remove_zero_rows(),
            lhs,
            rhs,
            dist,
        )
    }
    ///// Join node from right with distance 1
    fn try_join_right(&self, rhs: &Self) -> Option<JoinedNode<T>> {
        let lhs = self.clone();
        let rhs = rhs.clone();
        let (lhs, rhs) = Self::input_intersections(lhs, rhs, 1)?;
        let (lhs, rhs) = Self::output_intersections(lhs, rhs, 1)?;
        let (lhs, rhs) = Self::connecting_intersections(lhs, rhs, 1)?;
        let lmap = lhs.mapping;
        let rmap = rhs.mapping;
        let lmat = EdgeMappingMatrix::from_rows(
            &lmap
                .outgoing
                .into_iter()
                .map(|e| e.index)
                .zip(lmap.matrix.row_iter())
                .sorted_by(|(e1, _), (e2, _)| e1.cmp(e2))
                .map(|(_, v)| v)
                .collect::<Vec<_>>(),
        );
        let rmat = EdgeMappingMatrix::from_columns(
            &rmap
                .incoming
                .into_iter()
                .map(|e| e.index)
                .zip(rmap.matrix.column_iter())
                .sorted_by(|(e1, _), (e2, _)| e1.cmp(e2))
                .map(|(_, v)| v)
                .collect::<Vec<_>>(),
        );
        Some(JoinedNode {
            mapping: LoadedEdgeMapping {
                incoming: lmap.incoming,
                outgoing: rmap.outgoing,
                matrix: rmat * lmat,
            },
            token: lhs.token + rhs.token,
        })
    }
    ///// Join node from right with distance 1
    pub fn join_right(&self, rhs: &Self) -> JoinedNode<T> {
        self.try_join_right(rhs).unwrap_or_else(|| {
            JoinedNode::new(<Self as TokenContext<T, LoadedEdge>>::token(self) + rhs.token())
        })
    }
}
/// Stores sequenced tokens with an edge map
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct JoinedNode<T: Tokenize> {
    token: Token<T>,
    mapping: LoadedEdgeMapping,
}
impl<T: Tokenize> JoinedNode<T> {
    pub fn new(token: Token<T>) -> Self {
        Self {
            token,
            mapping: LoadedEdgeMapping::new(),
        }
    }
}
impl<T: Tokenize> TokenContext<T, LoadedEdge> for JoinedNode<T> {
    type Mapping = LoadedEdgeMapping;
    fn token(&self) -> &Token<T> {
        &self.token
    }
    fn into_token(self) -> Token<T> {
        self.token
    }
    fn mapping(&self) -> &Self::Mapping {
        &self.mapping
    }
    fn mapping_mut(&mut self) -> &mut Self::Mapping {
        &mut self.mapping
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{
        token::{
            TokenContext,
            Token,
        },
        SequenceGraph,
    };
    use std::collections::HashSet;
    use maplit::hashset;
    use pretty_assertions::assert_eq;
    #[allow(unused)]
    use tracing::debug;
    use tracing_test::traced_test;
    use test::Bencher;
    lazy_static::lazy_static! {
        pub static ref SEQS: Vec<&'static str> = Vec::from([
            "",
            "bc",
            "aa",
            "abc",
            "bcade",
            "bcaade",
            "bcbcabc",
            "abcaa",
        ]);
        pub static ref G: SequenceGraph<char> = {
            let mut g = SequenceGraph::new();
            for &s in SEQS.iter() {
                g.read_sequence(s.chars());
            }
            g
        };
    }
    #[macro_export]
    macro_rules! assert_distances_match {
        ($name: expr, $g: expr, $input: expr, $ty: ty, [ $($e: expr),* $(,)? ]) => {
            assert_eq!(
                $input,
                hashset![
                    $($e),*
                ]
                .into_iter()
                .map(|(d, t): (usize, $ty)| (d, $g.find_node_index(t).unwrap()))
                .collect(),
                $name,
            )
        }
    }
    #[traced_test]
    #[test]
    fn join_bc() {
        debug!(
            "{:#?}",
            G.node_indices()
                .zip(G.all_node_weights())
                .collect::<Vec<_>>()
        );
        let b_node = G.load_node_from('b').unwrap();
        let c_node = G.load_node_from('c').unwrap();
        let bc_node = b_node.join_right(&c_node);

        assert_eq!(bc_node.token(), b_node.token() + c_node.token());
        let m = bc_node.mapping();
        debug!("Incoming");
        assert_distances_match!("Incoming", G,
            m.incoming_sources().collect::<HashSet<_>>(),
            Token<char>,
            [
                (1, Token::Start),

                (2, Token::Start),
                (1, Token::Element('a')),

                (3, Token::Start),
                (2, Token::Element('b')),
                (1, Token::Element('c')),

                (6, Token::Start),
                (5, Token::Element('b')),
                (4, Token::Element('c')),
                (3, Token::Element('b')),
                (2, Token::Element('c')),
            ]);
        assert_distances_match!("Outgoing", G,
            m.outgoing_targets().collect::<HashSet<_>>(),
            Token<char>,
            [
                (1, Token::End),

                (1, Token::Element('a')),
                (2, Token::Element('d')),
                (3, Token::Element('e')),
                (4, Token::End),

                (2, Token::Element('a')),
                (3, Token::Element('d')),
                (4, Token::Element('e')),
                (5, Token::End),

                (1, Token::Element('b')),
                (2, Token::Element('c')),
                (3, Token::Element('a')),
                (4, Token::Element('b')),
                (5, Token::Element('c')),
                (6, Token::End),

                (2, Token::Element('b')),
                (3, Token::Element('c')),
                (4, Token::End),

                (3, Token::End),
            ]);
        //debug!("{:#?}", _bc_node.get_info(&G));
    }
    #[bench]
    fn bench_join(b: &mut Bencher) {
        let b_node = G.load_node_from('b').unwrap();
        let c_node = G.load_node_from('c').unwrap();
        b.iter(|| b_node.join_right(&c_node))
    }
    #[traced_test]
    #[test]
    fn join_aa() {
        let a_node = G.load_node_from('a').unwrap();
        let aa_node = a_node.join_right(&a_node);
        assert_eq!(aa_node.token(), a_node.token() + a_node.token());
        let m = aa_node.mapping();
        assert_distances_match!("Incoming", G, m.incoming_sources().collect::<HashSet<_>>(),
            Token<char>,
            [
                (1, Token::Start),

                (3, Token::Start),
                (2, Token::Element('b')),
                (1, Token::Element('c')),

                (4, Token::Start),
                (3, Token::Element('a')),
            ]);
        assert_distances_match!("Outgoing", G, m.outgoing_targets().collect::<HashSet<_>>(),
            Token<char>,
            [
                (1, Token::End),

                (1, Token::Element('d')),
                (2, Token::Element('e')),
                (3, Token::End),
            ]);
        //debug!("{:#?}", _bc_node.get_info(&G));
    }
    #[traced_test]
    #[test]
    fn join_ba() {
        let b_node = G.load_node_from('b').unwrap();
        let a_node = G.load_node_from('a').unwrap();
        let ba_node = b_node.join_right(&a_node);
        assert_eq!(ba_node.token(), b_node.token() + a_node.token());
        let m = ba_node.mapping();
        assert_distances_match!("Incoming", G, m.incoming_sources().collect::<HashSet<_>>(),
            Token<char>,
            []);
        assert_distances_match!("Outgoing", G, m.outgoing_targets().collect::<HashSet<_>>(),
            Token<char>,
            []);
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
