use crate::{
    hypergraph::{
        r#match::*,
        search::*,
        Child,
        Hypergraph,
        Indexed,
        Pattern,
        PatternId,
        PatternView,
        VertexData,
        VertexIndex,
        Parent,
        ChildPatterns,
        TokenPosition,
    },
    token::Tokenize,
};
use itertools::Itertools;

pub struct Searcher<'g, T: Tokenize, D: MatchDirection> {
    graph: &'g Hypergraph<T>,
    _ty: std::marker::PhantomData<D>,
}
impl<'g, T: Tokenize> Searcher<'g, T, MatchRight> {
    pub fn search_right(graph: &'g Hypergraph<T>) -> Self {
        Self::new(graph)
    }
}
impl<'g, T: Tokenize> Searcher<'g, T, MatchLeft> {
    #[allow(unused)]
    pub fn search_left(graph: &'g Hypergraph<T>) -> Self {
        Self::new(graph)
    }
}
impl<'g, T: Tokenize, D: MatchDirection> std::ops::Deref for Searcher<'g, T, D> {
    type Target = Hypergraph<T>;
    fn deref(&self) -> &Self::Target {
        self.graph
    }
}
impl<'g, T: Tokenize + 'g, D: MatchDirection> Searcher<'g, T, D> {
    pub fn new(graph: &'g Hypergraph<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    pub(crate) fn matcher(&self) -> Matcher<'g, T, D> {
        Matcher::new(self.graph)
    }
    pub fn find_sequence(
        &self,
        pattern: impl IntoIterator<Item = impl Into<T>>,
    ) -> SearchResult {
        let pattern = T::tokenize(pattern.into_iter());
        self.to_token_children(pattern)
            .ok_or(NotFound::UnknownTokens)
            .and_then(|pattern|
                self.find_pattern(pattern)
            )
    }
    pub(crate) fn find_pattern(
        &self,
        pattern: impl IntoIterator<Item = impl Into<Child>>,
    ) -> SearchResult {
        let pattern: Pattern = pattern.into_iter().map(Into::into).collect();
        MatchRight::split_head_tail(&pattern)
            .ok_or(NotFound::EmptyPatterns)
            .and_then(|(head, tail)|
                if tail.is_empty() {
                    // single index is not a pattern
                    Err(NotFound::SingleIndex)
                } else {
                    self.find_largest_matching_parent(head, tail.to_vec())
                }
            )
    }
    #[allow(unused)]
    pub fn find_parent_matching_context(
        &self,
        context: PatternView<'g>,
        vertex: &VertexData,
    ) -> SearchResult {
        self.find_parent_matching_context_below_width(context, vertex, None)
    }
    pub(crate) fn find_largest_matching_parent(
        &self,
        index: impl Indexed,
        postfix: Pattern,
    ) -> SearchResult {
        let vertex = self.expect_vertex_data(index);
        //let width = pattern_width(&postfix) + vertex.width;
        self.find_parent_matching_context_below_width(
            &postfix[..],
            vertex,
            None,
        )
        .and_then(|search_found|
            match search_found.parent_match.remainder {
                Some(post) =>
                    self.find_largest_matching_parent(
                        search_found.index,
                        post[..].to_vec()
                    ),
                // todo: match prefixes
                _ => Ok(search_found),
            }
        )
    }
    pub fn find_parent_matching_context_below_width(
        &self,
        context: PatternView<'g>,
        vertex: &VertexData,
        width_ceiling: Option<TokenPosition>,
    ) -> SearchResult {
        let parents = vertex.get_parents_below_width(width_ceiling);
        if let Some((&index, child_patterns, parent, pattern_id, sub_index)) =
            self.find_parent_with_matching_children(parents.clone(), context) {
                self.matcher().compare_child_pattern_at_offset(
                    child_patterns,
                    context,
                    pattern_id,
                    sub_index,
                )
                .map(|parent_match|
                    SearchFound {
                        index: Child::new(index, parent.width),
                        pattern_id,
                        sub_index,
                        parent_match,
                    }
                )
                .map_err(NotFound::Mismatch)
        } else {
            // compare all parent's children
            parents.into_iter().find_map(|(&index, parent)|
                self.matcher().compare_with_parent_children(context, index, parent)
                    .map(|(parent_match, pattern_id, sub_index)|
                        SearchFound {
                            index: Child::new(index, parent.width),
                            pattern_id,
                            sub_index,
                            parent_match,
                        }
                    )
                    .ok()
            )
            .ok_or(NotFound::NoMatchingParent)
        }
    }
    /// find parent with a child pattern matching context
    fn find_parent_with_matching_children(
        &'g self,
        mut parents: impl Iterator<Item = (&'g VertexIndex, &'g Parent)>,
        context: PatternView<'g>,
    ) -> Option<(
        &'g VertexIndex,
        &'g ChildPatterns,
        &'g Parent,
        PatternId,
        usize,
    )> {
        parents.find_map(|(index, parent)| {
            let vert = self.expect_vertex_data(*index);
            let child_patterns = vert.get_children();
            //print!("matching parent \"{}\" ", self.index_string(parent.index));
            // get child pattern indices able to match at all
            let candidates = D::candidate_parent_pattern_indices(parent, child_patterns);
            candidates
                .into_iter()
                .find(|(pattern_index, sub_index)| {
                    // find pattern with same next index
                    Self::compare_next_index_in_child_pattern(
                        child_patterns,
                        context,
                        pattern_index,
                        *sub_index,
                    )
                })
                .map(|(pattern_index, sub_index)| {
                    (index, child_patterns, parent, pattern_index, sub_index)
                })
        })
    }
    pub(crate) fn compare_next_index_in_child_pattern(
        child_patterns: &'g ChildPatterns,
        context: PatternView<'g>,
        pattern_index: &PatternId,
        sub_index: usize,
    ) -> bool {
        D::pattern_head(context)
            .and_then(|next_sub| {
                D::index_next(sub_index).and_then(|i| {
                    child_patterns
                        .get(pattern_index)
                        .and_then(|pattern| pattern.get(i).map(|next_sup| next_sub == next_sup))
                })
            })
            .unwrap_or(false)
    }
    /// try to find child pattern with context matching sub_context
    pub(crate) fn find_best_child_pattern(
        child_patterns: &'g ChildPatterns,
        candidates: impl Iterator<Item = (usize, PatternId)>,
        sub_context: PatternView<'g>,
    ) -> Result<(PatternId, usize), NotFound> {
        candidates.find_or_first(|(pattern_index, sub_index)| {
            Self::compare_next_index_in_child_pattern(
                child_patterns,
                sub_context,
                pattern_index,
                *sub_index,
            )
        })
        .ok_or(NotFound::NoChildPatterns)
    }
}
