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

type VertexIndex = usize;
type VertexParents = HashSet<Parent>;
type VertexChildren = HashSet<Child>;
type TokenPosition = usize;
type IndexPattern = Vec<VertexIndex>;
type VertexPattern = Vec<VertexData>;
type IndexPatternView<'a> = &'a[&'a VertexIndex];
type VertexPatternView<'a> = Vec<&'a VertexData>;
type VertexPatternViewMut<'a> = Vec<&'a mut VertexData>;
#[derive(Debug, Hash, PartialEq, Eq)]
enum VertexKey<T: Tokenize> {
    Token(Token<T>),
    Pattern(VertexIndex)
}
#[derive(Debug, Hash, PartialEq, Eq)]
struct Parent {
    index: VertexIndex, // the parent pattern
    positions: Vec<usize>, // positions of child in parent patterns
}
impl Into<VertexIndex> for Parent {
    fn into(self) -> VertexIndex {
        self.index
    }
}
#[derive(Debug, Hash, PartialEq, Eq)]
struct Child {
    pattern: IndexPattern
}
#[derive(Debug, PartialEq, Eq)]
struct VertexData {
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
#[derive(Debug, PartialEq, Eq)]
struct VertexFamily {
    children: VertexChildren,
    parents: VertexParents,
}
impl VertexFamily {
    pub fn new() -> Self {
        Self {
            parents: VertexParents::new(),
            children: VertexChildren::new(),
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
    fn get_vertex_data(&self, index: VertexIndex) -> Option<&VertexData> {
        self.get_vertex(index).map(|(_, v)| v)
    }
    fn expect_vertex_data(&self, index: VertexIndex) -> &VertexData {
        self.get_vertex(index).map(|(_, v)| v).expect("Invalid index!")
    }
    fn get_vertex_data_mut(&mut self, index: VertexIndex) -> Option<&mut VertexData> {
        self.get_vertex_mut(index).map(|(_, v)| v)
    }
    fn get_vertex(&self, index: VertexIndex) -> Option<(&VertexKey<T>, &VertexData)> {
        self.graph.get_index(index)
    }
    fn get_vertex_mut(&mut self, index: VertexIndex) -> Option<(&mut VertexKey<T>, &mut VertexData)> {
        self.graph.get_index_mut(index)
    }
    fn insert_token(&mut self, token: Token<T>) -> VertexIndex {
        self.graph.insert_full(VertexKey::Token(token), VertexData::with_width(1)).0
    }
    fn to_token_indices(&mut self, tokens: impl IntoIterator<Item=Token<T>>) -> IndexPattern {
        tokens.into_iter()
            .map(|token|
                self.get_token_index(&token)
                    .unwrap_or_else(|| self.insert_token(token))
            )
            .collect()
    }
    fn expect_vertices<I: Into<VertexIndex>>(&self, indices: impl Iterator<Item=I>) -> VertexPatternView<'_> {
        indices
            .map(move |index| self.expect_vertex_data(index.into()))
            .collect()
    }
    fn get_vertices<I: Into<VertexIndex>>(&self, indices: impl Iterator<Item=I>) -> Option<VertexPatternView<'_>> {
        indices
            .map(move |index| self.get_vertex_data(index.into()))
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
    pub fn index_sequence<N: Into<T>, I: IntoIterator<Item = N>>(&mut self, seq: I) {
        let seq = seq.into_iter();
        let mut hyperedge: IndexPattern = Vec::with_capacity(seq.size_hint().0);
        let mut parents: Option<VertexParents> = None;
        let tokens = T::tokenize(seq);
        let indices = self.to_token_indices(tokens);
        let vertices = self.expect_vertices(indices.iter().cloned());
        let len = indices.len();
        for (pos_in_seq, (vertex_index, vertex)) in indices.iter().zip(vertices).enumerate() {
            let mut parent_families: Vec<Vec<_>> = vertex.family.parents.iter()
                .map(|parent| {
                    let vertex = self.expect_vertex_data(parent.index.clone());
                    parent.positions
                          .iter()
                          .zip(&vertex.family.children)
                          .collect()
                })
                .collect();
            for post_index in indices.iter().skip(pos_in_seq + 1) {
                if parent_families.len() < 1 {
                    break;
                }
                parent_families.retain(|parent_positions|
                    parent_positions.iter()
                        .any(|(pos, child)| child.pattern.get(*pos + post_index).map(|elem| elem == vertex_index).unwrap_or(false))
                );
            }
        }
    }
}

