use crate::{
    hypergraph::*,
    token::Tokenize,
};

impl<'t, 'a, T> Hypergraph<T>
where
    T: Tokenize + 't,
{
    pub fn get_vertex(&self, index: impl Indexed) -> Option<(&VertexKey<T>, &VertexData)> {
        self.graph.get_index(*index.borrow())
    }
    pub fn get_vertex_mut(
        &mut self,
        index: impl Indexed,
    ) -> Option<(&mut VertexKey<T>, &mut VertexData)> {
        self.graph.get_index_mut(*index.borrow())
    }
    pub fn expect_vertex(&self, index: impl Indexed) -> (&VertexKey<T>, &VertexData) {
        let index = *index.borrow();
        self.get_vertex(index)
            .unwrap_or_else(|| panic!("Index {} does not exist!", index))
    }
    pub fn expect_vertex_mut(
        &mut self,
        index: impl Indexed,
    ) -> (&mut VertexKey<T>, &mut VertexData) {
        let index = *index.borrow();
        self.get_vertex_mut(index)
            .unwrap_or_else(|| panic!("Index {} does not exist!", index))
    }
    pub fn get_vertex_key(&self, index: impl Indexed) -> Option<&VertexKey<T>> {
        self.get_vertex(index).map(|entry| entry.0)
    }
    pub fn expect_vertex_key(&self, index: impl Indexed) -> &VertexKey<T> {
        self.expect_vertex(index).0
    }
    pub fn get_vertex_data(&self, index: impl Indexed) -> Option<&VertexData> {
        self.get_vertex(index).map(|(_, v)| v)
    }
    pub fn get_vertex_data_mut(&mut self, index: impl Indexed) -> Option<&mut VertexData> {
        self.get_vertex_mut(index).map(|(_, v)| v)
    }
    pub fn expect_vertex_data(&self, index: impl Indexed) -> &VertexData {
        self.expect_vertex(index).1
    }
    pub fn expect_vertex_data_mut(&mut self, index: impl Indexed) -> &mut VertexData {
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
    pub fn vertex_iter(&self) -> impl Iterator<Item = (&VertexKey<T>, &VertexData)> {
        self.graph.iter()
    }
    pub fn vertex_iter_mut(&mut self) -> impl Iterator<Item = (&VertexKey<T>, &mut VertexData)> {
        self.graph.iter_mut()
    }
    pub fn vertex_key_iter(&self) -> impl Iterator<Item = &VertexKey<T>> {
        self.graph.keys()
    }
    pub fn vertex_data_iter(&self) -> impl Iterator<Item = &VertexData> {
        self.graph.values()
    }
    pub fn vertex_data_iter_mut(&mut self) -> impl Iterator<Item = &mut VertexData> {
        self.graph.values_mut()
    }
    pub fn expect_vertices(
        &self,
        indices: impl Iterator<Item = impl Indexed>,
    ) -> VertexPatternView<'_> {
        indices
            .map(move |index| self.expect_vertex_data(index))
            .collect()
    }
    pub fn get_vertices(
        &self,
        indices: impl Iterator<Item = impl Indexed>,
    ) -> Option<VertexPatternView<'_>> {
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
    pub fn to_token_indices(
        &self,
        tokens: impl IntoIterator<Item = Token<T>>,
    ) -> Option<IndexPattern> {
        tokens
            .into_iter()
            .map(|token| self.get_token_index(&token).ok_or(()))
            .collect::<Result<_, ()>>()
            .ok()
    }
    pub fn to_token_children(&self, tokens: impl IntoIterator<Item = Token<T>>) -> Option<Pattern> {
        Some(
            self.to_token_indices(tokens)?
                .into_iter()
                .map(|index| Child::new(index, 1))
                .collect(),
        )
    }
    pub fn get_token_indices(
        &self,
        tokens: impl Iterator<Item = &'t Token<T>>,
    ) -> Option<IndexPattern> {
        let mut v = IndexPattern::with_capacity(tokens.size_hint().0);
        for token in tokens {
            let index = self.get_token_index(token)?;
            v.push(index);
        }
        Some(v)
    }
    pub fn to_child(&self, index: impl Indexed) -> Child {
        Child::new(*index.borrow(), self.index_width(&index))
    }
    pub fn to_children(&self, indices: impl IntoIterator<Item = impl Indexed>) -> Pattern {
        indices.into_iter().map(|i| self.to_child(i)).collect()
    }
    pub fn get_common_pattern_in_parent(
        &self,
        pattern: impl IntoIterator<Item = impl Indexed>,
        parent: impl Indexed,
    ) -> Option<(PatternId, usize)> {
        let mut parents = pattern
            .into_iter()
            .map(|index| self.expect_vertex_data(index))
            .map(|vertex| vertex.get_parent(parent.index()))
            .collect::<Option<Vec<_>>>()?
            .into_iter()
            .enumerate();
        let (_, first) = parents.next()?;
        first
            .pattern_indices
            .iter()
            .find(|(pat, pos)| {
                parents.all(|(i, post)| post.exists_at_pos_in_pattern(*pat, pos + i))
            })
            .cloned()
    }
    pub fn expect_common_pattern_in_parent(
        &self,
        pattern: impl IntoIterator<Item = impl Indexed>,
        parent: impl Indexed,
    ) -> (PatternId, usize) {
        self.get_common_pattern_in_parent(pattern, parent)
            .expect("No common pattern in parent for children.")
    }
}
