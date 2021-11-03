use crate::{
    hypergraph::{
        *,
        search::*,
    },
    token::Tokenize,
};
use tokio_stream::{
    //Stream,
    StreamExt,
};
use async_std::sync::{
    Arc,
    RwLock,
};

impl<'t, 'a, T> Hypergraph<T>
where
    T: Tokenize + 't,
{
    pub fn get_vertex(&self, index: impl Indexed) -> Result<(&VertexKey<T>, &VertexData), NotFound> {
        self.graph.get_index(*index.index()).ok_or(NotFound::UnknownIndex)
    }
    pub fn get_vertex_mut(
        &mut self,
        index: impl Indexed,
    ) -> Result<(&mut VertexKey<T>, &mut VertexData), NotFound> {
        self.graph.get_index_mut(*index.index()).ok_or(NotFound::UnknownIndex)
    }
    pub fn expect_vertex(&self, index: impl Indexed) -> (&VertexKey<T>, &VertexData) {
        let index = *index.index();
        self.get_vertex(index)
            .unwrap_or_else(|_| panic!("Index {} does not exist!", index))
    }
    pub fn expect_vertex_mut(
        &mut self,
        index: impl Indexed,
    ) -> (&mut VertexKey<T>, &mut VertexData) {
        let index = *index.index();
        self.get_vertex_mut(index)
            .unwrap_or_else(|_| panic!("Index {} does not exist!", index))
    }
    pub fn get_vertex_key(&self, index: impl Indexed) -> Result<&VertexKey<T>, NotFound> {
        self.get_vertex(index).map(|entry| entry.0)
    }
    pub fn expect_vertex_key(&self, index: impl Indexed) -> &VertexKey<T> {
        self.expect_vertex(index).0
    }
    pub fn get_vertex_data(&self, index: impl Indexed) -> Result<&VertexData, NotFound> {
        self.get_vertex(index).map(|(_, v)| v)
    }
    pub fn get_vertex_data_mut(&mut self, index: impl Indexed) -> Result<&mut VertexData, NotFound> {
        self.get_vertex_mut(index).map(|(_, v)| v)
    }
    pub fn expect_vertex_data(&self, index: impl Indexed) -> &VertexData {
        self.expect_vertex(index).1
    }
    pub fn expect_vertex_data_mut(&mut self, index: impl Indexed) -> &mut VertexData {
        self.expect_vertex_mut(index).1
    }
    pub fn get_vertex_data_by_key(&self, key: &VertexKey<T>) -> Result<&VertexData, NotFound> {
        self.graph.get(key).ok_or(NotFound::UnknownKey)
    }
    pub fn get_vertex_data_by_key_mut(&mut self, key: &VertexKey<T>) -> Result<&mut VertexData, NotFound> {
        self.graph.get_mut(key).ok_or(NotFound::UnknownKey)
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
    ) -> Result<VertexPatternView<'_>, NotFound> {
        indices
            .map(move |index| self.get_vertex_data(index))
            .collect()
    }
    pub fn get_token_data(&self, token: &Token<T>) -> Result<&VertexData, NotFound> {
        self.get_vertex_data_by_key(&VertexKey::Token(*token))
    }
    pub fn get_token_data_mut(&mut self, token: &Token<T>) -> Result<&mut VertexData, NotFound> {
        self.get_vertex_data_by_key_mut(&VertexKey::Token(*token))
    }
    pub fn get_index_by_key(&self, key: &VertexKey<T>) -> Result<VertexIndex, NotFound> {
        self.graph.get_index_of(key).ok_or(NotFound::UnknownKey)
    }
    pub fn expect_index_by_key(&self, key: &VertexKey<T>) -> VertexIndex {
        self.graph.get_index_of(key).expect("Key does not exist")
    }
    pub fn get_token_index(&self, token: &Token<T>) -> Result<VertexIndex, NotFound> {
        self.get_index_by_key(&VertexKey::Token(*token))
    }
    pub fn to_token_indices_iter(
        &'a self,
        tokens: impl IntoIterator<Item = Token<T>> + 'a,
    ) -> impl Iterator<Item=Result<VertexIndex, NotFound>> + 'a {
        tokens
            .into_iter()
            .map(move |token| self.get_token_index(&token))
    }
    pub fn to_token_indices(
        &self,
        tokens: impl IntoIterator<Item = Token<T>>,
    ) -> Result<IndexPattern, NotFound> {
        tokens
            .into_iter()
            .map(|token| self.get_token_index(&token))
            .collect()
    }
    pub fn to_token_children_iter(&'a self, tokens: impl IntoIterator<Item = Token<T>> + 'a) -> impl Iterator<Item=Result<Child, NotFound>> + 'a {
        self.to_token_indices_iter(tokens)
            .map(move |index|
                index.map(|index| Child::new(index, 1))
            )
    }
    pub fn to_token_children(&self, tokens: impl IntoIterator<Item = Token<T>>) -> Result<impl IntoPattern<Item=Child>, NotFound> {
        self.to_token_children_iter(tokens).collect::<Result<Pattern, _>>()
    }
    pub fn get_token_indices(
        &self,
        tokens: impl Iterator<Item = &'t Token<T>>,
    ) -> Result<IndexPattern, NotFound> {
        let mut v = IndexPattern::with_capacity(tokens.size_hint().0);
        for token in tokens {
            let index = self.get_token_index(token)?;
            v.push(index);
        }
        Ok(v)
    }
    pub fn to_child(&self, index: impl Indexed) -> Child {
        Child::new(*index.index(), self.index_width(&index))
    }
    pub fn to_children(&self, indices: impl IntoIterator<Item = impl Indexed>) -> Pattern {
        indices.into_iter().map(|i| self.to_child(i)).collect()
    }
    pub fn get_pattern_parents(
        &self,
        pattern: impl IntoIterator<Item = impl Indexed>,
        parent: impl Indexed,
    ) -> Result<Vec<Parent>, NotFound> {
        pattern
            .into_iter()
            .map(|index| {
                let vertex = self.expect_vertex_data(index);
                vertex.get_parent(parent.index()).map(Clone::clone)
            })
            .collect()
    }
    pub fn get_common_pattern_in_parent(
        &self,
        pattern: impl IntoIterator<Item = impl Indexed>,
        parent: impl Indexed,
    ) -> Result<(PatternId, usize), NotFound> {
        let mut parents = self.get_pattern_parents(pattern, parent)?
            .into_iter()
            .enumerate();
        parents.next().and_then(|(_, first)|
            first
                .pattern_indices
                .iter()
                .find(|(pat, pos)| {
                    parents.all(|(i, post)| post.exists_at_pos_in_pattern(*pat, pos + i))
                })
                .cloned()
        )
        .ok_or(NotFound::NoMatchingParent)
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
impl<'t, 'a, T> Hypergraph<T>
where
    T: Tokenize + Send + Sync + 't,
{
    pub async fn async_to_token_indices_stream(
        arc: Arc<RwLock<Self>>,
        tokens: impl TokenStream<T> + 't,
    ) -> impl PatternStream<VertexIndex, Token<T>> + 't {
        let handle = tokio::runtime::Handle::current();
        tokens.map(move |token|
            // is this slow?
            handle.block_on(async {
                arc.read().await.get_token_index(&token.into_token())
                    .map_err(|_| Token::Element(token))
            })
        )
    }
    pub async fn async_to_token_children_stream(
        arc: Arc<RwLock<Self>>,
        tokens: impl TokenStream<T> + 't,
        ) -> impl PatternStream<Child, Token<T>> + 't {
        Self::async_to_token_indices_stream(arc, tokens).await
            .map(move |index|
                index.into_inner().map(|index| Child::new(index, 1))
            )
    }
    pub fn to_token_indices_stream(
        &'a self,
        tokens: impl TokenStream<T> + 'a,
    ) -> impl PatternStream<VertexIndex, Token<T>> + 'a {
        tokens.map(move |token|
            self.get_token_index(&token.into_token())
                .map_err(|_| Token::Element(token))
        )
    }
    pub fn to_token_children_stream(&'a self, tokens: impl TokenStream<T> + 'a) -> impl PatternStream<Child, Token<T>> + 'a {
        self.to_token_indices_stream(tokens)
            .map(move |index|
                index.into_inner().map(|index| Child::new(index, 1))
            )
    }
}
