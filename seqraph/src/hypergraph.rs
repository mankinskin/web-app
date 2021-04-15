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
use either::Either;

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
pub enum MatchResult {
    Remainder(Either<Pattern, Pattern>),
    Matching,
    Mismatch,
}
impl MatchResult {
	pub fn flip_remainder(self) -> Self {
		match self {
			Self::Remainder(e) => Self::Remainder(e.flip()),
			_ => self,
		}
	}
}

impl<'t, 'a, T> Hypergraph<T>
where
    T: Tokenize + 't + std::fmt::Display,
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
        self.expect_vertex(index).1
    }
    fn expect_vertex<I: Borrow<VertexIndex>>(&self, index: I) -> (&VertexKey<T>, &VertexData) {
        self.get_vertex(index).expect("Invalid index!")
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
    fn to_token_children(&mut self, tokens: impl IntoIterator<Item=Token<T>>) -> Pattern {
        tokens.into_iter()
            .map(|token|
				Child {
                	index: self.get_token_index(&token)
                        .unwrap_or_else(|| self.insert_token(token)),
					width: 1,
				}
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
    fn pattern_width(pat: PatternView<'a>) -> usize {
        pat.into_iter().fold(0, |acc, child| acc + child.width)
    }
    fn pattern_string(&self, pattern: PatternView<'_>) -> String {
		pattern.iter().map(|child| self.index_string(child.index)).join("")
    }
    fn index_string(&self, index: VertexIndex) -> String {
		let (key, data) = self.expect_vertex(index);
		match key {
			VertexKey::Token(token) => token.to_string(),
			VertexKey::Pattern(index) => {
				self.pattern_string(&data.family.children.get(0).expect("Pattern vertex with no children!")[..])
			},
		}
    }
    fn match_parent(&self, post_sub_pat: PatternView<'a>, parent: &Parent) -> MatchResult {
		self.match_parent_at_pos(post_sub_pat, parent, None)
    }
    fn match_parent_prefix(&self, post_sub_pat: PatternView<'a>, parent: &Parent) -> MatchResult {
        // match successors in parents where a is at beginning
		self.match_parent_at_pos(post_sub_pat, parent, Some(0))
    }
    fn find_matching_parent_prefix_below_width(&self,
            post_sub_pat: PatternView<'a>,
            parents: &Vec<Parent>,
            width_ceiling: Option<usize>,
        ) -> MatchResult {
		self.find_matching_parent_at_pos_below_width(post_sub_pat, parents, Some(0), width_ceiling)
	}
    fn find_matching_parent_at_pos(&self,
            post_sub_pat: PatternView<'a>,
            parents: &Vec<Parent>,
            pos: Option<usize>,
        ) -> MatchResult {
		self.find_matching_parent_at_pos_below_width(post_sub_pat, parents, pos, None)
	}
    fn find_matching_parent(&self,
            post_sub_pat: PatternView<'a>,
            parents: &Vec<Parent>,
        ) -> MatchResult {
		self.find_matching_parent_at_pos_below_width(post_sub_pat, parents, None, None)
	}
	/// match sub_pat against children in parent with an optional offset.
    fn match_parent_at_pos(&self, post_sub_pat: PatternView<'a>, parent: &Parent, pos: Option<usize>) -> MatchResult {
		println!("match_parent");
        let vert = self.expect_vertex_data(parent.index);
        let children = &vert.family.children;
        // find pattern where sub is at pos
        // TODO: any heuristics to find the most efficient pattern to compare?
		let candidates = if let Some(pos) = pos {
			Either::Left(parent.positions.iter()
				.filter(move |(_pattern_index, pos_sub)| *pos_sub == pos))
		} else {
			Either::Right(parent.positions.iter())
        };
		// find children with matching successors or pick last candidate
		let best_match = candidates.find_or_first(|(pattern_index, pos_sub)|
			post_sub_pat.get(0)
				.and_then(|i|
					children[*pattern_index].get(pos_sub+1)
						.map(|b| i.index == b.index)
				).is_some()
		);
		if let Some((pattern_index, pos_sub)) = best_match {
			let children = &children[*pattern_index][pos_sub+1..];
			println!("matching remaining pattern (\"{}\") with children (\"{}\")",
				self.pattern_string(post_sub_pat),
				self.pattern_string(children),
			);
			self.match_prefix(
			    post_sub_pat,
			    children,
			)
		} else {
            println!("no matching parents");
        	MatchResult::Mismatch
		}
    }
    fn find_matching_parent_at_pos_below_width(&self,
            post_sub_pat: PatternView<'a>,
            parents: &Vec<Parent>,
			pos: Option<usize>,
            width_ceiling: Option<usize>,
        ) -> MatchResult {
        if let Some(ceil) = width_ceiling {
			Either::Left(parents.iter()
            	.filter(move |parent| parent.width < ceil))
		} else {
			Either::Right(parents.iter())
		}
        .find_map(|parent| match self.match_parent_at_pos(post_sub_pat, parent, pos) {
            MatchResult::Mismatch => None,
            r @ _ => Some(r),
        })
        .unwrap_or(MatchResult::Mismatch)
	}
    fn match_sub_pattern_to_super(&self,
            post_sub_pat: PatternView<'a>,
            sub: VertexIndex,
            sup: VertexIndex,
            sup_width: usize,
        ) -> MatchResult {
		println!("match_prefix_impl");
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
			self.match_parent_prefix(post_sub_pat, parent)
        } else {
            println!("matching available parents");
            // search sup in parents
			self.find_matching_parent_prefix_below_width(
				post_sub_pat,
				parents,
				Some(sup_width),
			)
        }
    }
    fn match_prefix(&self, pattern_a: PatternView<'a>, pattern_b: PatternView<'a>) -> MatchResult {
		println!("match_prefix \"{}\" and \"{}\"", self.pattern_string(pattern_a), self.pattern_string(pattern_b));
        let pattern_a_iter = pattern_a.iter();
        let pattern_b_iter = pattern_b.iter();
		let mut zipped = pattern_a_iter
			.zip_longest(pattern_b_iter)
			.enumerate()
			.skip_while(|(_, eob)|
				match eob {
					EitherOrBoth::Both(a, b) => a == b,
					_ => false,
				}
			);
		if let Some((pos, eob)) = zipped.next() {
			match eob {
				// different elements on both sides
				EitherOrBoth::Both(&Child {
					index: index_a,
					width: width_a,
				}, &Child {
					index: index_b,
					width: width_b,
				}) => {
					println!("matching \"{}\" and \"{}\"", self.index_string(index_a), self.index_string(index_b));
					// Note: depending on sizes of a, b it may be differently efficient
					// to search for children or parents, large patterns have less parents,
					// small patterns have less children
					// search larger in parents of smaller
					let post_sub;
					let post_sup;
					let sub;
					let sup;
					let sup_width;
					let rotate = if width_a == width_b {
						// relatives can not have same sizes
						return MatchResult::Mismatch;
					} else if width_a < width_b {
						println!("right super");
						post_sub = &pattern_a[pos+1..];
						post_sup = &pattern_b[pos+1..];
						sub = index_a;
						sup = index_b;
						sup_width = width_b;
						false
    	            } else {
						println!("left super");
						post_sub = &pattern_b[pos+1..];
						post_sup = &pattern_a[pos+1..];
						sub = index_b;
						sup = index_a;
						sup_width = width_a;
						true
					};
					let result = self.match_sub_pattern_to_super(
						post_sub,
					    sub,
					    sup,
					    sup_width,
					);
					// left remainder: sub remainder
					// right remainder: sup remainder
					// matching: sub & sup finished
					println!("return {:#?}", result);
					let result = match result {
						MatchResult::Remainder(remainder) =>
							match remainder {
								Either::Left(rem) => {
									self.match_prefix(
										&rem,
										post_sup,
									)
								},
								Either::Right(rem) => MatchResult::Remainder(Either::Right([&rem, post_sup].concat())),
							},
						MatchResult::Matching => {
							let rem: Vec<_> = post_sup.iter().cloned().collect();
							if rem.is_empty() {
								MatchResult::Matching
							} else {
								MatchResult::Remainder(Either::Right(rem))
							}
						},
						MatchResult::Mismatch => MatchResult::Mismatch,
					};
					if rotate {
						result.flip_remainder()
					} else {
						result
					}
				},
				EitherOrBoth::Left(_) => MatchResult::Remainder(Either::Left(pattern_a[pos..].iter().cloned().collect())),
				EitherOrBoth::Right(_) => MatchResult::Remainder(Either::Right(pattern_b[pos..].iter().cloned().collect())),
			}
		} else {
			// skipped all elements
            println!("no patterns left");
			MatchResult::Matching
		}
    }
    pub fn find_pattern(&self, pattern_a: PatternView<'a>) {
        let mut pattern_iter = pattern_a.iter().cloned().enumerate();
        while let Some((pos, Child {
                index,
                width,
            })) = pattern_iter.next() {
			let vert = self.expect_vertex_data(index);
			//vert.family
			//	.parents.iter()
			//	.find(|parent| 
			//		self.match_parent(pattern, parent, post_sup_pat: PatternView<'a>)
			//	);
        }
    }
    pub fn index_pattern(&mut self, indices: IndexPattern) {
        let vertices = self.expect_vertices(indices.into_iter());
        let len = vertices.len();
    }
    pub fn index_token_sequence<N: Into<T>, I: IntoIterator<Item = N>>(&mut self, seq: I) {
        let seq = seq.into_iter();
        let tokens = T::tokenize(seq);
        let indices = self.to_token_indices(tokens);
		self.index_pattern(indices)
	}
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
        let bc_pattern = &[Child::new(bc, 2)];
        let b_c_pattern = &[Child::new(b, 1), Child::new(c, 1)];
        let a_d_c_pattern = &[Child::new(a, 1), Child::new(d, 1), Child::new(c, 1)];
        assert_eq!(g.match_prefix(a_bc_pattern, ab_c_pattern), MatchResult::Matching);
        assert_eq!(g.match_prefix(ab_c_pattern, a_bc_pattern), MatchResult::Matching);
        assert_eq!(g.match_prefix(a_b_c_pattern, a_bc_pattern), MatchResult::Matching);
        assert_eq!(g.match_prefix(a_b_c_pattern, a_b_c_pattern), MatchResult::Matching);
        assert_eq!(g.match_prefix(ab_c_pattern, a_b_c_pattern), MatchResult::Matching);
        assert_eq!(g.match_prefix(a_bc_d_pattern, ab_c_d_pattern), MatchResult::Matching);
        assert_eq!(g.match_prefix(abcd_pattern, a_bc_d_pattern), MatchResult::Matching);
        assert_eq!(g.match_prefix(abc_d_pattern, a_bc_d_pattern), MatchResult::Matching);
        assert_eq!(g.match_prefix(abc_d_pattern, abcd_pattern), MatchResult::Matching);
        assert_eq!(g.match_prefix(bc_pattern, abcd_pattern), MatchResult::Mismatch);
        assert_eq!(g.match_prefix(b_c_pattern, a_bc_pattern), MatchResult::Mismatch);
        assert_eq!(g.match_prefix(b_c_pattern, a_d_c_pattern), MatchResult::Mismatch);

        assert_eq!(g.match_prefix(a_b_c_pattern, abcd_pattern), MatchResult::Remainder(Either::Right(vec![Child::new(d, 1)])));

        assert_eq!(g.match_prefix(ab_c_d_pattern, a_bc_pattern), MatchResult::Remainder(Either::Left(vec![Child::new(d, 1)])));
        assert_eq!(g.match_prefix(a_bc_pattern, ab_c_d_pattern), MatchResult::Remainder(Either::Right(vec![Child::new(d, 1)])));
    }
}
