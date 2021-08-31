use crate::hypergraph::*;
use crate::{
    token::{
        Tokenize,
    },
};
use std::borrow::Borrow;

impl<'t, 'a, T> Hypergraph<T>
    where T: Tokenize + 't,
{
    pub fn get_vertex<I: Borrow<VertexIndex>>(&self, index: I) -> Option<(&VertexKey<T>, &VertexData)> {
        self.graph.get_index(*index.borrow())
    }
    pub fn get_vertex_mut<I: Borrow<VertexIndex>>(&mut self, index: I) -> Option<(&mut VertexKey<T>, &mut VertexData)> {
        self.graph.get_index_mut(*index.borrow())
    }
    pub fn expect_vertex<I: Borrow<VertexIndex>>(&self, index: I) -> (&VertexKey<T>, &VertexData) {
        let index = *index.borrow();
        self.get_vertex(index).unwrap_or_else(|| panic!("Index {} does not exist!", index))
    }
    pub fn expect_vertex_mut<I: Borrow<VertexIndex>>(&mut self, index: I) -> (&mut VertexKey<T>, &mut VertexData) {
        let index = *index.borrow();
        self.get_vertex_mut(index).unwrap_or_else(|| panic!("Index {} does not exist!", index))
    }
    pub fn get_vertex_key<I: Borrow<VertexIndex>>(&self, index: I) -> Option<&VertexKey<T>> {
        self.get_vertex(index).map(|entry| entry.0)
    }
    pub fn expect_vertex_key<I: Borrow<VertexIndex>>(&self, index: I) -> &VertexKey<T> {
        self.expect_vertex(index).0
    }
    pub fn get_vertex_data<I: Borrow<VertexIndex>>(&self, index: I) -> Option<&VertexData> {
        self.get_vertex(index).map(|(_, v)| v)
    }
    pub fn get_vertex_data_mut(&mut self, index: VertexIndex) -> Option<&mut VertexData> {
        self.get_vertex_mut(index).map(|(_, v)| v)
    }
    pub fn expect_vertex_data<I: Borrow<VertexIndex>>(&self, index: I) -> &VertexData {
        self.expect_vertex(index).1
    }
    pub fn expect_vertex_data_mut(&mut self, index: VertexIndex) -> &mut VertexData {
        self.expect_vertex_mut(index).1
    }
    pub fn get_vertex_data_by_key(&self, key: &VertexKey<T>) -> Option<&VertexData> {
        self.graph.get(key)
    }
    pub fn get_vertex_data_by_key_mut(&mut self, key: &VertexKey<T>) -> Option<&mut VertexData> {
        self.graph.get_mut(key)
    }
    pub fn expect_vertex_data_by_key(&self, key: &VertexKey<T>) -> &VertexData {
        self.graph.get(key).expect("Key does not exist")
    }
    pub fn expect_vertex_data_by_key_mut(&mut self, key: &VertexKey<T>) -> &mut VertexData {
        self.graph.get_mut(key).expect("Key does not exist")
    }
    pub fn vertex_iter(&self) -> impl Iterator<Item=(&VertexKey<T>, &VertexData)> {
        self.graph.iter()
    }
    pub fn vertex_iter_mut(&mut self) -> impl Iterator<Item=(&VertexKey<T>, &mut VertexData)> {
        self.graph.iter_mut()
    }
    pub fn vertex_key_iter(&self) -> impl Iterator<Item=&VertexKey<T>> {
        self.graph.keys()
    }
    pub fn vertex_data_iter(&self) -> impl Iterator<Item=&VertexData> {
        self.graph.values()
    }
    pub fn vertex_data_iter_mut(&mut self) -> impl Iterator<Item=&mut VertexData> {
        self.graph.values_mut()
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
    pub fn get_token_data(&self, token: &Token<T>) -> Option<&VertexData> {
        self.get_vertex_data_by_key(&VertexKey::Token(*token))
    }
    pub fn get_token_data_mut(&mut self, token: &Token<T>) -> Option<&mut VertexData> {
        self.get_vertex_data_by_key_mut(&VertexKey::Token(*token))
    }
    pub fn get_index_by_key(&self, key: &VertexKey<T>) -> Option<VertexIndex> {
        self.graph.get_index_of(key)
    }
    pub fn expect_index_by_key(&self, key: &VertexKey<T>) -> VertexIndex {
        self.graph.get_index_of(key).expect("Key does not exist")
    }
    pub fn get_token_index(&self, token: &Token<T>) -> Option<VertexIndex> {
        self.get_index_by_key(&VertexKey::Token(*token))
    }
    pub fn to_token_indices(&self, tokens: impl IntoIterator<Item=Token<T>>) -> Option<IndexPattern> {
        tokens.into_iter()
            .map(|token|
                self.get_token_index(&token).ok_or(())
            )
            .collect::<Result<_, ()>>()
            .ok()
    }
    pub fn to_token_children(&self, tokens: impl IntoIterator<Item=Token<T>>) -> Option<Pattern> {
        Some(self.to_token_indices(tokens)?.into_iter()
            .map(|index| Child::new(index, 1))
            .collect())
    }
    pub fn get_token_indices(&self, tokens: impl Iterator<Item=&'t Token<T>>) -> Option<IndexPattern> {
        let mut v = IndexPattern::with_capacity(tokens.size_hint().0);
        for token in tokens {
            let index = self.get_token_index(token)?;
            v.push(index);
        }
        Some(v)
    }
}