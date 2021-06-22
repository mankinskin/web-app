use std::fmt::Debug;
use crate::{
    //pattern::{
    //    Pattern,
    //},
    token::{
        Token,
        Tokenize,
    },
};
use std::collections::{
    HashSet,
};
use std::borrow::Borrow;
use itertools::{
    Itertools,
};
use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};

mod search;
mod r#match;
mod split;
mod path_tree;

pub type VertexIndex = usize;
type VertexParents = Vec<Parent>;
type ChildPatterns = Vec<Pattern>;
type ChildPatternView<'a> = &'a[PatternView<'a>];
pub type Pattern = Vec<Child>;
type TokenPosition = usize;
pub type PatternIndex = usize;
type IndexPattern = Vec<VertexIndex>;
type VertexPattern = Vec<VertexData>;
pub type PatternView<'a> = &'a[Child];
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
    width: TokenPosition,
    pattern_indices: HashSet<(usize, PatternIndex)>, // positions of child in parent patterns
}
impl Parent {
    pub fn new(index: impl Borrow<VertexIndex>, width: TokenPosition) -> Self {
        Self {
            index: *index.borrow(),
            width,
            pattern_indices: Default::default(),
        }
    }
    pub fn add_pattern_index(&mut self, pattern: usize, index: PatternIndex) {
        self.pattern_indices.insert((pattern, index));
    }
}
impl Into<VertexIndex> for Parent {
    fn into(self) -> VertexIndex {
        self.index
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Child {
    index: VertexIndex, // the child index
    width: TokenPosition, // the token width
}
impl Child {
    pub fn new(index: impl Borrow<VertexIndex>, width: TokenPosition) -> Self {
        Self {
            index: *index.borrow(),
            width,
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VertexData {
    width: TokenPosition,
    parents: VertexParents,
    children: ChildPatterns,
}
impl VertexData {
    pub fn with_width(width: TokenPosition) -> Self {
        Self {
            width,
            parents: VertexParents::new(),
            children: ChildPatterns::new(),
        }
    }
    pub fn add_pattern<'c, I: IntoIterator<Item=&'c Child>>(&mut self, pat: I) -> usize {
        // TODO: detect unmatching pattern
        let id = self.children.len();
        self.children.push(pat.into_iter().cloned().collect());
        id
    }
    pub fn add_parent(&mut self, vertex: VertexIndex, width: TokenPosition, pattern: usize, index: PatternIndex) {
        if let Some(parent) = self.parents.iter_mut().find(|parent| parent.index == vertex) {
            parent.add_pattern_index(pattern, index);
        } else {
            let mut parent = Parent::new(vertex, width);
            parent.add_pattern_index(pattern, index);
            self.parents.push(parent);
        }
    }
}

#[derive(Debug)]
pub struct Hypergraph<T: Tokenize> {
    graph: indexmap::IndexMap<VertexKey<T>, VertexData>,
}
impl<'t, 'a, T> Hypergraph<T>
    where T: Tokenize + 't,
{
    fn next_pattern_id(&mut self) -> VertexIndex {
        static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
        ID_COUNTER.fetch_add(1, Ordering::Relaxed)
    }
    pub fn new() -> Self {
        Self {
            graph: indexmap::IndexMap::new(),
        }
    }
    fn get_token_index(&self, token: &Token<T>) -> Option<VertexIndex> {
        self.graph.get_index_of(&VertexKey::Token(*token))
    }
    pub fn get_token_data(&self, token: &Token<T>) -> Option<&VertexData> {
        self.graph.get(&VertexKey::Token(*token))
    }
    pub fn get_token_data_mut(&mut self, token: &Token<T>) -> Option<&mut VertexData> {
        self.graph.get_mut(&VertexKey::Token(*token))
    }
    fn get_vertex_data<I: Borrow<VertexIndex>>(&self, index: I) -> Option<&VertexData> {
        self.get_vertex(index).map(|(_, v)| v)
    }
    fn expect_vertex_data<I: Borrow<VertexIndex>>(&self, index: I) -> &VertexData {
        self.expect_vertex(index).1
    }
    fn expect_vertex<I: Borrow<VertexIndex>>(&self, index: I) -> (&VertexKey<T>, &VertexData) {
        self.get_vertex(index).expect("Index does not exist!")
    }
    fn expect_vertex_mut(&mut self, index: VertexIndex) -> (&mut VertexKey<T>, &mut VertexData) {
        self.get_vertex_mut(index).expect("Index does not exist!")
    }
    fn expect_vertex_data_mut(&mut self, index: VertexIndex) -> &mut VertexData {
        self.expect_vertex_mut(index).1
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
    pub fn insert_tokens(&mut self, tokens: impl IntoIterator<Item=Token<T>>) -> Vec<VertexIndex> {
        tokens.into_iter()
            .map(|token|
                self.insert_vertex(VertexKey::Token(token), VertexData::with_width(1))
            )
            .collect()
    }
    fn add_pattern_parent(&mut self, indices: Vec<VertexIndex>, parent_index: VertexIndex, width: TokenPosition, pattern_index: usize) {
        for (i, child_index) in indices.into_iter().enumerate() {
            let node = self.expect_vertex_data_mut(child_index);
            node.add_parent(parent_index, width, pattern_index, i);
        }
    }
    fn to_width_indices_children(
        &self,
        indices: impl IntoIterator<Item=impl Borrow<VertexIndex>>,
        ) -> (TokenPosition, Vec<VertexIndex>, Vec<Child>) {
        let mut width = 0;
        let (a, b) = indices.into_iter()
            .map(|index| {
                let index = *index.borrow();
                let w = self.expect_vertex_data(index).width.clone();
                width += w;
                (index, Child::new(index, w))
            })
            .unzip();
        (width, a, b)
    }
    pub fn insert_to_pattern(&mut self, index: VertexIndex, indices: impl IntoIterator<Item=impl Borrow<VertexIndex>>) -> usize {
        // todo handle token nodes
        let (width, indices, children) = self.to_width_indices_children(indices);
        let data = self.expect_vertex_data_mut(index);
        let pattern_index = data.add_pattern(&children);
        self.add_pattern_parent(indices, index, width, pattern_index);
        pattern_index
    }
    pub fn insert_patterns(&mut self, indices: impl IntoIterator<Item=impl IntoIterator<Item=impl Borrow<VertexIndex>>>) -> usize {
        // todo handle token nodes
        let mut iter = indices.into_iter();
        let first = iter.next().unwrap();
        let node = self.insert_pattern(first);
        for pat in iter {
            self.insert_to_pattern(node, pat);
        }
        node
    }
    pub fn insert_pattern(&mut self, indices: impl IntoIterator<Item=impl Borrow<VertexIndex>>) -> VertexIndex {
        // todo check if exists already
        let id = self.next_pattern_id();
        // todo handle token nodes
        let (width, indices, children) = self.to_width_indices_children(indices);
        let mut new_data = VertexData::with_width(width);
        let pattern_index = new_data.add_pattern(&children);
        let index = self.insert_vertex(VertexKey::Pattern(id), new_data);
        self.add_pattern_parent(indices, index, width, pattern_index);
        index
    }
    pub fn insert_vertex(&mut self, key: VertexKey<T>, data: VertexData) -> VertexIndex {
        self.graph.insert_full(key, data).0
    }
    pub fn to_token_indices(&mut self, tokens: impl IntoIterator<Item=Token<T>>) -> IndexPattern {
        tokens.into_iter()
            .map(|token|
                 self.get_token_index(&token)
                 .unwrap_or_else(|| self.insert_token(token))
                )
            .collect()
    }
    pub fn to_token_children(&mut self, tokens: impl IntoIterator<Item=Token<T>>) -> Pattern {
        self.to_token_indices(tokens).into_iter()
            .map(|index| Child { index, width: 1, })
            .collect()
    }
    pub fn expect_vertices<I: Borrow<VertexIndex>>(&self, indices: impl Iterator<Item=I>) -> VertexPatternView<'_> {
        indices
            .map(move |index| self.expect_vertex_data(index))
            .collect()
    }
    pub fn get_vertices<I: Borrow<VertexIndex>>(&self, indices: impl Iterator<Item=I>) -> Option<VertexPatternView<'_>> {
        indices
            .map(move |index| self.get_vertex_data(index))
            .collect()
    }
    pub fn get_token_indices(&mut self, tokens: impl Iterator<Item=&'t Token<T>>) -> IndexPattern {
        let mut v = IndexPattern::with_capacity(tokens.size_hint().0);
        for token in tokens {
            let index = self.get_token_index(token)
                .unwrap_or_else(|| self.insert_token(token.clone()));
            v.push(index);
        }
        v
    }
    pub fn pattern_width(pat: PatternView<'a>) -> TokenPosition {
        pat.into_iter().fold(0, |acc, child| acc + child.width)
    }
    //pub fn index_sequence<N: Into<T>, I: IntoIterator<Item = N>>(&mut self, seq: I) -> VertexIndex {
    //    let seq = seq.into_iter();
    //    let tokens = T::tokenize(seq);
    //    let pattern = self.to_token_children(tokens);
    //    self.index_pattern(&pattern[..])
    //}
}
impl<'t, 'a, T> Hypergraph<T>
    where T: Tokenize + 't + std::fmt::Display,
{
    fn sub_pattern_string(&'a self, pattern: impl IntoIterator<Item=&'a Child>) -> String {
        pattern.into_iter().map(|child| self.sub_index_string(child.index)).join("")
    }
    fn pattern_string(&self, pattern: PatternView<'_>) -> String {
        pattern.iter().map(|child| self.sub_index_string(child.index)).join("_")
    }
    fn sub_index_string(&self, index: VertexIndex) -> String {
        let (key, data) = self.expect_vertex(index);
        match key {
            VertexKey::Token(token) => token.to_string(),
            VertexKey::Pattern(_) => {
                self.sub_pattern_string(&data.children.get(0).expect("Pattern vertex with no children!")[..])
            },
        }
    }
    pub fn index_string(&self, index: VertexIndex) -> String {
        let (key, data) = self.expect_vertex(index);
        match key {
            VertexKey::Token(token) => token.to_string(),
            VertexKey::Pattern(_) => {
                self.pattern_string(&data.children.get(0).expect("Pattern vertex with no children!")[..])
            },
        }
    }
}

#[cfg(test)]
#[macro_use]
mod tests {
    use super::*;
    use crate::token::*;
    lazy_static::lazy_static! {
        pub static ref 
            CONTEXT: (
                Hypergraph<char>,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                ) = {
        let mut graph = Hypergraph::new();
        if let [a, b, c, d, e, f, g, h, i] = graph.insert_tokens(
            vec![
                Token::Element('a'),
                Token::Element('b'),
                Token::Element('c'),
                Token::Element('d'),
                Token::Element('e'),
                Token::Element('f'),
                Token::Element('g'),
                Token::Element('h'),
                Token::Element('i'),
            ])[..] {
            // abcdefghi
            // ababababcdbcdefdefcdefefghefghghi
            // ->
            // abab ab abcdbcdefdefcdefefghefghghi
            // ab abab abcdbcdefdefcdefefghefghghi

            // abcdbcdef def cdef efgh efgh ghi

            // abcd b cdef
            // abcd bcd ef

            // ab cd
            // abc d
            // a bcd

        let ab = graph.insert_pattern(&[a, b]);
        let bc = graph.insert_pattern(&[b, c]);
        let ef = graph.insert_pattern(&[e, f]);
        let def = graph.insert_pattern(&[d, ef]);
        let cdef = graph.insert_pattern(&[c, def]);
        let gh = graph.insert_pattern(&[g, h]);
        let efgh = graph.insert_pattern(&[ef, gh]);
        let ghi = graph.insert_pattern(&[gh, i]);
        let abc = graph.insert_patterns(&[
            [ab, c],
            [a, bc],
        ]);
        let cd = graph.insert_pattern(&[c, d]);
        let bcd = graph.insert_patterns(&[
            [bc, d],
            [b, cd],
        ]);
        //let abcd = graph.insert_pattern(&[abc, d]);
        //graph.insert_to_pattern(abcd, &[a, bcd]);
        let abcd = graph.insert_patterns(&[
            [abc, d],
            [a, bcd],
        ]);
        let efghi = graph.insert_patterns(&[
            [efgh, i],
            [ef, ghi],
        ]);
        let abcdefghi = graph.insert_pattern(&[abcd, efghi]);
        let aba = graph.insert_pattern(&[ab, a]);
        let abab = graph.insert_patterns(&[
            [aba, b],
            [ab, ab],
        ]);
        let ababab = graph.insert_patterns(&[
            [abab, ab],
            [ab, abab],
        ]);
        let ababcd = graph.insert_patterns(&[
            [ab, abcd],
            [aba, bcd],
            [abab, cd],
        ]);
        let ababababcd = graph.insert_patterns(&[
            [ababab, abcd],
            [abab, ababcd],
        ]);
        let ababcdefghi = graph.insert_patterns(vec![
            [ab, abcdefghi].into_iter(),
            [ababcd, efghi].into_iter(),
        ]);
        let ababababcdefghi = graph.insert_patterns(vec![
            [ababababcd, efghi].into_iter(),
            [abab, ababcdefghi].into_iter(),
        ]);
        let longer_pattern = graph.insert_pattern(&[ababab, abcdefghi]);
        (
            graph,
            a,
            b,
            c,
            d,
            e,
            f,
            g,
            h,
            i,
            ab,
            bc,
            bcd,
            abc,
            abcd,
        )
        } else {
            panic!();
        }
                };
    }
}