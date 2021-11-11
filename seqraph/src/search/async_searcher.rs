use crate::{
    r#match::*,
    search::*,
    vertex::*,
    *,
};
use tokio_stream::{
    Stream,
    StreamExt,
};
use itertools::Itertools;
use async_std::sync::{
    Arc,
    RwLock,
};

pub struct AsyncSearcher<T: Tokenize + Send, D: AsyncMatchDirection<T>> {
    graph: HypergraphHandle<T>,
    _ty: std::marker::PhantomData<D>,
}
impl<T: Tokenize + Send + 'static> AsyncSearcher<T, MatchRight> {
    pub fn search_right(graph: HypergraphHandle<T>) -> Self {
        Self::new(graph)
    }
}
impl<'a, T: Tokenize + Send + 'static, D: AsyncMatchDirection<T>> AsyncSearcher<T, D> {
    pub fn new(graph: HypergraphHandle<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    pub(crate) fn matcher(&self) -> AsyncMatcher<T, D> {
        AsyncMatcher::new(self.graph)
    }
    pub async fn find_sequence(&self, pattern: impl IntoIterator<Item = impl Into<T>>) -> SearchResult {
        let iter = tokenizing_iter(pattern.into_iter());
        let pattern = self.graph.read().await.to_token_children(iter)?;
        self.find_pattern(pattern).await
    }
    pub(crate) async fn find_pattern_iter(
        &self,
        pattern: impl IntoIterator<Item=Result<impl Into<Child>, NotFound>>,
    ) -> SearchResult {
        let pattern: Pattern = pattern.into_iter().map(|r| r.map(Into::into)).collect::<Result<Pattern, NotFound>>()?;
        MatchRight::split_head_tail(&pattern)
            .ok_or(NotFound::EmptyPatterns)
            .and_then(|(head, tail)| {
                if tail.is_empty() {
                    // single index is not a pattern
                    Err(NotFound::SingleIndex)
                } else {
                    async_std::task::block_on(
                        self.find_largest_matching_parent(head, tokio_stream::iter(tail.to_vec()))
                    )
                }
            })
    }
    pub(crate) async fn find_pattern(
        &self,
        pattern: impl IntoPattern<Item=impl Into<Child>>,
    ) -> SearchResult {
        let pattern: Pattern = pattern.into_iter().map(Into::into).collect();
        MatchRight::split_head_tail(&pattern)
            .ok_or(NotFound::EmptyPatterns)
            .and_then(|(head, tail)| {
                if tail.is_empty() {
                    // single index is not a pattern
                    Err(NotFound::SingleIndex)
                } else {
                    async_std::task::block_on(
                        self.find_largest_matching_parent(head, tokio_stream::iter(tail.to_vec()))
                    )
                }
            })
    }
    pub async fn find_largest_matching_parent(
        &self,
        index: impl Indexed,
        context: impl PatternStream<Child, Token<T>>,
    ) -> SearchResult {
        self.find_largest_matching_parent_below_width(index, context, None).await
    }
    pub async fn find_largest_matching_parent_below_width(
        &self,
        index: impl Indexed,
        context: impl PatternStream<Child, Token<T>>,
        width_ceiling: Option<TokenPosition>,
    ) -> SearchResult {
        let vertex = self.graph.read().await.expect_vertex_data(index);
        let parents = vertex.get_parents_below_width(width_ceiling);
        let matching_parent = self.find_parent_with_matching_children(parents.clone(), context).await;
        let search_found = if let Some((
                &index,
                child_patterns,
                parent,
                pattern_id,
                sub_index
            )) = matching_parent {
            self.matcher()
                .compare_child_pattern_at_offset(child_patterns, context, pattern_id, sub_index)
                .await
                .map(|parent_match| SearchFound {
                    index: Child::new(index, parent.width),
                    pattern_id,
                    sub_index,
                    parent_match,
                })
                .map_err(NotFound::Mismatch)
        } else {
            // compare all parent's children
            parents
                .into_iter()
                .find_map(|(&index, parent)| {
                    async_std::task::block_on(async {
                        self.matcher()
                            .compare_with_parent_children(context, index, parent)
                            .await
                            .map(|(parent_match, pattern_id, sub_index)| SearchFound {
                                index: Child::new(index, parent.width),
                                pattern_id,
                                sub_index,
                                parent_match,
                            })
                            .ok()
                    })
                })
                .ok_or(NotFound::NoMatchingParent)
        }?;
        match search_found.parent_match.remainder {
            Some(post) => {
                self.find_largest_matching_parent(search_found.index, tokio_stream::iter(post[..].to_vec())).await
            },
            // todo: match prefixes
            _ => Ok(search_found),
        }
    }
    /// find parent with a child pattern matching context
    pub async fn find_parent_with_matching_children(
        &'a self,
        mut parents: impl Iterator<Item = (&'a VertexIndex, &'a Parent)>,
        context: impl PatternStream<Child, Token<T>>,
    ) -> Option<(
        &'a VertexIndex,
        &'a ChildPatterns,
        &'a Parent,
        PatternId,
        usize,
    )> {
        parents.find_map(|(index, parent)| {
            async_std::task::block_on(async {
                let vert = self.graph.read().await.expect_vertex_data(*index);
                let child_patterns = vert.get_children();
                //print!("matching parent \"{}\" ", self.index_string(parent.index));
                // get child pattern indices able to match at all
                D::candidate_parent_pattern_indices(parent, child_patterns)
                    .into_iter()
                    .find(|(pattern_index, sub_index)| {
                        // find pattern with same next index
                        async_std::task::block_on(
                            Self::compare_next_index_in_child_pattern(
                                child_patterns,
                                context,
                                pattern_index,
                                *sub_index,
                            )
                        )
                    })
                    .map(|(pattern_index, sub_index)| {
                        (index, child_patterns, parent, pattern_index, sub_index)
                    })
            })
        })
    }
    pub(crate) async fn compare_next_index_in_child_pattern(
        child_patterns: &'a ChildPatterns,
        context: impl PatternStream<Child, Token<T>>,
        pattern_index: &PatternId,
        sub_index: usize,
    ) -> bool {
        if let Some(next_sub) = D::take_head(context).await {
            D::index_next(sub_index)
                .and_then(|i| {
                    child_patterns
                        .get(pattern_index)
                        .and_then(|pattern| pattern.get(i).map(|next_sup| next_sub == next_sup))
                })
                .unwrap_or(false)
        } else {
            false
        }
    }
    /// try to find child pattern with context matching sub_context
    pub(crate) async fn find_best_child_pattern(
        child_patterns: &'a ChildPatterns,
        candidates: impl Iterator<Item = (usize, PatternId)>,
        sub_context: impl PatternStream<Child, Token<T>>,
    ) -> Result<(PatternId, usize), NotFound> {
        candidates
            .find_or_first(|(pattern_index, sub_index)| {
                async_std::task::block_on(
                    Self::compare_next_index_in_child_pattern(
                        child_patterns,
                        sub_context,
                        pattern_index,
                        *sub_index,
                    )
                )
            })
            .ok_or(NotFound::NoChildPatterns)
    }
}
