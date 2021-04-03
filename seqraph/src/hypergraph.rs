use petgraph::{
    graph::{
        NodeIndex,
    },
    Direction,
};
use std::fmt::Debug;
use std::ops::{
    Deref,
    DerefMut,
};
use crate::{
    //pattern::{
    //    Pattern,
    //},
    token::{
        Token,
        TokenContext,
        Tokenize,
        ContextLink,
    },
};
use std::collections::HashSet;
use indexmap::IndexSet;
use std::borrow::Borrow;
use itertools::{
    Itertools,
    EitherOrBoth,
};

type VertexIndex = usize;
type VertexParents = Vec<Parent>;
type ChildPatterns = Vec<Pattern>;
type Pattern = Vec<Child>;
type TokenPosition = usize;
type IndexPattern = Vec<VertexIndex>;
type VertexPattern = Vec<VertexData>;
type PatternView<'a> = &'a[Child];
type VertexPatternView<'a> = Vec<&'a VertexData>;
type VertexPatternViewMut<'a> = Vec<&'a mut VertexData>;
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum VertexKey<T: Tokenize> {
    Token(Token<T>),
    Pattern(VertexIndex)
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Parent {
    index: VertexIndex, // the parent pattern
    width: usize,
    positions: HashSet<(usize, usize)>, // positions of child in parent patterns
}
impl Parent {
    pub fn new(index: VertexIndex, width: usize) -> Self {
        Self {
            index,
            width,
            positions: Default::default(),
        }
    }
    pub fn add_position(&mut self, index: usize, pos: usize) {
        self.positions.insert((index, pos));
    }
}
impl Into<VertexIndex> for Parent {
    fn into(self) -> VertexIndex {
        self.index
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Child {
    index: VertexIndex, // the child index
    width: usize, // the token width
}
impl Child {
    pub fn new(index: VertexIndex, width: usize) -> Self {
        Self {
            index,
            width,
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VertexData {
    width: usize,
    family: VertexFamily
}
impl VertexData {
    pub fn with_width(width: usize) -> Self {
        Self {
            width,
            family: VertexFamily::default(),
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VertexFamily {
    parents: VertexParents,
    children: ChildPatterns,
}
impl VertexFamily {
    pub fn new() -> Self {
        Self {
            parents: VertexParents::new(),
            children: ChildPatterns::new(),
        }
    }
    pub fn add_pattern<'c, I: IntoIterator<Item=&'c Child>>(&mut self, pat: I) {
        // TODO: detect unmatching pattern
        self.children.push(pat.into_iter().cloned().collect())
    }
    pub fn add_parent(&mut self, vertex: VertexIndex, width: usize, pattern_index: usize, position: usize) {
        if let Some(parent) = self.parents
            .iter_mut()
            .find(|parent| parent.index == vertex) {
            parent.add_position(pattern_index, position);
        } else {
            let mut parent = Parent::new(vertex, width);
            parent.add_position(pattern_index, position);
            self.parents.push(parent);
        }
    }
}
impl std::default::Default for VertexFamily {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug)]
pub struct Hypergraph<T: Tokenize> {
    graph: indexmap::IndexMap<VertexKey<T>, VertexData>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
enum Remainder {
    Left(Pattern),
    Right(Pattern),
}
#[derive(Debug, PartialEq, Eq, Clone)]
enum MatchResult {
    Remainder(Remainder),
    Matching,
    Mismatch,
}

impl<'t, 'a, T> Hypergraph<T>
where
    T: Tokenize + 't,
{
    pub fn new() -> Self {
        Self {
            graph: indexmap::IndexMap::new(),
        }
    }
    fn get_token_index(&self, token: &Token<T>) -> Option<VertexIndex> {
        self.graph.get_index_of(&VertexKey::Token(*token))
    }
    fn get_token_data(&self, token: &Token<T>) -> Option<&VertexData> {
        self.graph.get(&VertexKey::Token(*token))
    }
    fn get_token_data_mut(&mut self, token: &Token<T>) -> Option<&mut VertexData> {
        self.graph.get_mut(&VertexKey::Token(*token))
    }
    fn get_vertex_data<I: Borrow<VertexIndex>>(&self, index: I) -> Option<&VertexData> {
        self.get_vertex(index).map(|(_, v)| v)
    }
    fn expect_vertex_data<I: Borrow<VertexIndex>>(&self, index: I) -> &VertexData {
        self.get_vertex(index).map(|(_, v)| v).expect("Invalid index!")
    }
    fn get_vertex_data_mut(&mut self, index: VertexIndex) -> Option<&mut VertexData> {
        self.get_vertex_mut(index).map(|(_, v)| v)
    }
    fn get_vertex<I: Borrow<VertexIndex>>(&self, index: I) -> Option<(&VertexKey<T>, &VertexData)> {
        self.graph.get_index(*index.borrow())
    }
    fn get_vertex_mut(&mut self, index: VertexIndex) -> Option<(&mut VertexKey<T>, &mut VertexData)> {
        self.graph.get_index_mut(index)
    }
    pub fn insert_token(&mut self, token: Token<T>) -> VertexIndex {
        self.insert_vertex(VertexKey::Token(token), VertexData::with_width(1))
    }
    pub fn insert_vertex(&mut self, key: VertexKey<T>, data: VertexData) -> VertexIndex {
        self.graph.insert_full(key, data).0
    }
    fn to_token_indices(&mut self, tokens: impl IntoIterator<Item=Token<T>>) -> IndexPattern {
        tokens.into_iter()
            .map(|token|
                self.get_token_index(&token)
                    .unwrap_or_else(|| self.insert_token(token))
            )
            .collect()
    }
    fn expect_vertices<I: Borrow<VertexIndex>>(&self, indices: impl Iterator<Item=I>) -> VertexPatternView<'_> {
        indices
            .map(move |index| self.expect_vertex_data(index))
            .collect()
    }
    fn get_vertices<I: Borrow<VertexIndex>>(&self, indices: impl Iterator<Item=I>) -> Option<VertexPatternView<'_>> {
        indices
            .map(move |index| self.get_vertex_data(index))
            .collect()
    }
    fn get_token_indices(&mut self, tokens: impl Iterator<Item=&'t Token<T>>) -> IndexPattern {
        let mut v = IndexPattern::with_capacity(tokens.size_hint().0);
        for token in tokens {
            let index = self.get_token_index(token)
                .unwrap_or_else(|| self.insert_token(token.clone()));
            v.push(index);
        }
        v
    }
    fn match_parent_prefix(&self, sub_pat: PatternView<'a>, parent: &Parent, post_sup_pat: PatternView<'a>) -> MatchResult {
        // match successors in parents where a is at beginning
        let vert = self.expect_vertex_data(parent.index);
        let children = &vert.family.children;
        // find pattern where sub is at beginning
        // TODO: any heuristics to find the most efficient pattern to compare?
        if let Some((pattern_index, _pos_sub)) = parent.positions
            .iter()
            .find(|(_pattern_index, pos_sub)| *pos_sub == 0) {
            println!("matching remaining pattern with children");
            self.match_prefix(
                &sub_pat[1..],
                &[&children[*pattern_index][1..], post_sup_pat].concat()
            )
        } else {
            MatchResult::Mismatch
        }
    }
    fn pattern_width(pat: PatternView<'a>) -> usize {
        pat.into_iter().fold(0, |acc, child| acc + child.width)
    }
    fn match_prefix_impl(&self,
            sub_pat: PatternView<'a>,
            post_sup_pat: PatternView<'a>,
            sup: VertexIndex,
            sub: VertexIndex,
            sup_width: usize,
        ) -> MatchResult {
        // search parent of sub
        let sub_vert = self.expect_vertex_data(sub);
        if sub_vert.family.parents.len() < 1 {
            return MatchResult::Mismatch;
        }
        let parents = &sub_vert.family.parents;
        // if parents contain sup parent, only try that one
        let sup_parent = parents.iter()
            .find(|Parent { index, positions, .. }|
                  *index == sup && positions.iter().any(|(_, pos)| *pos == 0)
            );
        // try matching and get remainder or mismatch
        if let Some(parent) = sup_parent {
            println!("sup found in parents");
            self.match_parent_prefix(&sub_pat, parent, post_sup_pat)
        } else {
            println!("matching available parents");
            // search sup in parents
            sub_vert.family.parents
                .iter()
                .filter(|parent| parent.width < sup_width)
                .find_map(|parent| match self.match_parent_prefix(sub_pat, parent, post_sup_pat) {
                    MatchResult::Mismatch => None,
                    r @ _ => Some(r),
                })
                .unwrap_or(MatchResult::Mismatch)
        }
    }
    fn match_prefix(&self, pattern_a: PatternView<'a>, pattern_b: PatternView<'a>) -> MatchResult {
        let mut pattern_a_iter = pattern_a.iter().cloned().peekable();
        let mut pattern_b_iter = pattern_b.iter().cloned().enumerate();
        while let Some(_) = pattern_a_iter.peek() {

            if let Some((pos, Child {
                index: index_b,
                width: width_b,
            })) = pattern_b_iter.next() {
                let Child {
                    index: index_a,
                    width: width_a,
                } = pattern_a_iter.next().unwrap();

                if index_a == index_b {
                    continue;
                }
                // Note: depending on sizes of a, b it may be differently efficient
                // to search for children or parents, large patterns have less parents,
                // small patterns have less children
                // search larger in parents of smaller
                return if width_a == width_b {
                    // relatives can not have same sizes
                    MatchResult::Mismatch
                } else if width_a > width_b {
                    println!("right sub");
                    self.match_prefix_impl(
                        &pattern_b[pos..],
                        &pattern_a[pos+1..],
                        index_a,
                        index_b,
                        width_a,
                        //width_b
                    )
                } else {
                    println!("left sub");
                    self.match_prefix_impl(
                        &pattern_a[pos..],
                        &pattern_b[pos+1..],
                        index_b,
                        index_a,
                        width_b,
                        //width_a
                    )
                };
            } else {
                println!("right empty");
                return MatchResult::Remainder(Remainder::Left(
                    pattern_a_iter.collect()
                ));
            }
        }
        let rem: Vec<_> = pattern_b_iter.map(|(_, x)| x).collect();
        if rem.is_empty() {
            println!("no patterns left");
            return MatchResult::Matching;
        } else {
            println!("left empty");
            return MatchResult::Remainder(Remainder::Right(rem));
        }
    }
    //pub fn index_sequence<N: Into<T>, I: IntoIterator<Item = N>>(&mut self, seq: I) {
    //    let seq = seq.into_iter();
    //    let mut hyperedge: IndexPattern = Vec::with_capacity(seq.size_hint().0);
    //    let mut parents: Option<VertexParents> = None;
    //    let tokens = T::tokenize(seq);
    //    let indices = self.to_token_indices(tokens);
    //    let vertices = self.expect_vertices(indices.iter().cloned());
    //    let len = indices.len();
    //    for (pos_in_seq, (vertex_index, vertex)) in indices.iter().zip(vertices).enumerate() {
    //        let mut parent_families: Vec<Vec<_>> = vertex.family.parents.iter()
    //            .map(|parent| {
    //                let vertex = self.expect_vertex_data(parent.index.clone());
    //                parent.positions
    //                      .iter()
    //                      .zip(&vertex.family.children)
    //                      .collect()
    //            })
    //            .collect();
    //        for post_index in indices.iter().skip(pos_in_seq + 1) {
    //            if parent_families.len() < 1 {
    //                break;
    //            }
    //            parent_families.retain(|parent_positions|
    //                parent_positions.iter()
    //                    .any(|(pos, child)| child.get(*pos + post_index).map(|elem| elem == vertex_index).unwrap_or(false))
    //            );
    //        }
    //    }
    //}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::*;
    #[test]
    fn match_simple() {
        let mut g = Hypergraph::new();
        let a = g.insert_token(Token::Element('a'));
        let b = g.insert_token(Token::Element('b'));
        let c = g.insert_token(Token::Element('c'));
        let d = g.insert_token(Token::Element('d'));
        let e = g.insert_token(Token::Element('e'));

        let mut ab_data = VertexData::with_width(2);
        ab_data.family.add_pattern(&[Child::new(a, 1), Child::new(b, 1)]);
        let ab = g.insert_vertex(VertexKey::Pattern(0), ab_data);
        g.get_vertex_data_mut(a).unwrap().family.add_parent(ab, 2, 0, 0);
        g.get_vertex_data_mut(b).unwrap().family.add_parent(ab, 2, 0, 1);

        let mut bc_data = VertexData::with_width(2);
        bc_data.family.add_pattern(&[Child::new(b, 1), Child::new(c, 1)]);
        let bc = g.insert_vertex(VertexKey::Pattern(1), bc_data);
        g.get_vertex_data_mut(b).unwrap().family.add_parent(bc, 2, 0, 0);
        g.get_vertex_data_mut(c).unwrap().family.add_parent(bc, 2, 0, 1);

        let mut abc_data = VertexData::with_width(3);
        let a_bc_pattern = &[Child::new(a, 1), Child::new(bc, 2)];
        let ab_c_pattern = &[Child::new(ab, 2), Child::new(c, 1)];
        abc_data.family.add_pattern(a_bc_pattern);
        abc_data.family.add_pattern(ab_c_pattern);
        let abc = g.insert_vertex(VertexKey::Pattern(2), abc_data);
        g.get_vertex_data_mut(a).unwrap().family.add_parent(abc, 3, 0, 0);
        g.get_vertex_data_mut(bc).unwrap().family.add_parent(abc, 3, 0, 1);
        g.get_vertex_data_mut(ab).unwrap().family.add_parent(abc, 3, 1, 0);
        g.get_vertex_data_mut(c).unwrap().family.add_parent(abc, 3, 1, 1);
        let a_b_c_pattern = &[Child::new(a, 1), Child::new(b, 1), Child::new(c, 1)];


        let abc_d_pattern = &[Child::new(abc, 3), Child::new(d, 1)];
        let a_bc_d_pattern = &[Child::new(a, 1), Child::new(bc, 2), Child::new(d, 1)];
        let ab_c_d_pattern = &[Child::new(ab, 2), Child::new(c, 1), Child::new(d, 1)];

        let mut abcd_data = VertexData::with_width(4);
        abcd_data.family.add_pattern(abc_d_pattern);
        abcd_data.family.add_pattern(a_bc_d_pattern);
        abcd_data.family.add_pattern(ab_c_d_pattern);
        let abcd = g.insert_vertex(VertexKey::Pattern(3), abcd_data);
        g.get_vertex_data_mut(a).unwrap().family.add_parent(abcd, 4, 1, 0);
        g.get_vertex_data_mut(d).unwrap().family.add_parent(abcd, 4, 0, 1);
        g.get_vertex_data_mut(d).unwrap().family.add_parent(abcd, 4, 1, 2);
        g.get_vertex_data_mut(d).unwrap().family.add_parent(abcd, 4, 2, 2);
        g.get_vertex_data_mut(c).unwrap().family.add_parent(abcd, 4, 2, 1);
        g.get_vertex_data_mut(bc).unwrap().family.add_parent(abcd, 4, 1, 1);
        g.get_vertex_data_mut(ab).unwrap().family.add_parent(abcd, 4, 2, 0);
        g.get_vertex_data_mut(abc).unwrap().family.add_parent(abcd, 4, 0, 0);
        let abcd_pattern = &[Child::new(abcd, 4)];
        assert_eq!(g.match_prefix(a_bc_pattern, ab_c_pattern), MatchResult::Matching);
        assert_eq!(g.match_prefix(ab_c_pattern, a_bc_pattern), MatchResult::Matching);
        assert_eq!(g.match_prefix(a_bc_pattern, a_b_c_pattern), MatchResult::Matching);
        assert_eq!(g.match_prefix(a_b_c_pattern, a_b_c_pattern), MatchResult::Matching);
        assert_eq!(g.match_prefix(ab_c_pattern, a_b_c_pattern), MatchResult::Matching);
        assert_eq!(g.match_prefix(ab_c_d_pattern, a_bc_d_pattern), MatchResult::Matching);
        assert_eq!(g.match_prefix(abcd_pattern, a_bc_d_pattern), MatchResult::Matching);
        assert_eq!(g.match_prefix(abc_d_pattern, a_bc_d_pattern), MatchResult::Matching);
        assert_eq!(g.match_prefix(abc_d_pattern, abcd_pattern), MatchResult::Matching);
        let bc_pattern = &[Child::new(bc, 2)];
        let b_c_pattern = &[Child::new(b, 1), Child::new(c, 1)];
        let a_d_c_pattern = &[Child::new(a, 1), Child::new(d, 1), Child::new(c, 1)];
        assert_eq!(g.match_prefix(bc_pattern, abcd_pattern), MatchResult::Mismatch);
        assert_eq!(g.match_prefix(b_c_pattern, a_bc_pattern), MatchResult::Mismatch);
        assert_eq!(g.match_prefix(b_c_pattern, a_d_c_pattern), MatchResult::Mismatch);
        assert_eq!(g.match_prefix(a_b_c_pattern, abcd_pattern), MatchResult::Remainder(Remainder::Right(vec![Child::new(d, 1)])));
    }
}
