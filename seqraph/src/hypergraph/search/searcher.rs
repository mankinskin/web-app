use crate::{
    hypergraph::{
        pattern_width,
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

type FindParentResult = (
    VertexIndex,
    Parent,
    (PatternId, usize),
    Pattern,
    PatternMatch,
);
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
    ) -> Option<(Child, (PatternId, usize), FoundRange)> {
        let pattern = T::tokenize(pattern.into_iter());
        let pattern = self.to_token_children(pattern)?;
        self.find_pattern(pattern)
    }
    pub(crate) fn find_pattern(
        &self,
        pattern: impl IntoIterator<Item = impl Into<Child>>,
    ) -> Option<(Child, (PatternId, usize), FoundRange)> {
        let pattern: Pattern = pattern.into_iter().map(Into::into).collect();
        let (head, tail) = MatchRight::split_head_tail(&pattern)?;
        if tail.is_empty() {
            // single index is not a pattern
            return None;
        }
        self.find_parent_matching_postfix(head, tail.to_vec())
    }
    pub fn find_parent_matching_context(
        &self,
        context: PatternView<'g>,
        vertex: &VertexData,
    ) -> Option<FindParentResult> {
        self.find_parent_matching_context_below_width(context, vertex, None)
    }
    pub(crate) fn find_parent_matching_postfix(
        &self,
        index: impl Indexed,
        postfix: Pattern,
    ) -> Option<(Child, (PatternId, usize), FoundRange)> {
        let vertex = self.expect_vertex_data(index);
        //let width = pattern_width(&postfix) + vertex.width;
        let (index, pattern_id, found_range) = self.find_parent_matching_context_below_width(
                &postfix[..],
                vertex,
                None,
            )
            .map(|(index, parent, pattern_pos, pre, m)| {
                (
                    Child::new(index, parent.width),
                    pattern_pos,
                    m.prepend_prefix(pre),
                )
            })?;
        match found_range {
            FoundRange::Complete => Some((index, pattern_id, found_range)),
            FoundRange::Prefix(post) => self.find_parent_matching_postfix(index, post[..].to_vec()),
            // todo: match prefixes
            _ => Some((index, pattern_id, found_range)),
        }
    }
    pub fn find_parent_matching_context_below_width(
        &self,
        context: PatternView<'g>,
        vertex: &VertexData,
        width_ceiling: Option<TokenPosition>,
    ) -> Option<FindParentResult> {
        let parents = vertex.get_parents_below_width(width_ceiling);
        let best_match = self.find_parent_with_matching_children(parents.clone(), context);
        best_match
            .and_then(|(&index, child_patterns, parent, pattern_index, sub_index)| {
                self.matcher().compare_child_pattern_with_remainder(
                    child_patterns,
                    context,
                    pattern_index,
                    sub_index,
                )
                .map(|(back_context, m)| (index, parent.clone(), (pattern_index, sub_index), back_context, m))
            })
            .or_else(|| {
                // compare all parent's children
                parents.into_iter().find_map(|(&index, parent)| {
                    self.matcher().compare_context_with_child_pattern(context, index, parent)
                        .map(|(ppos, pre, m)| (index, parent.clone(), ppos, pre, m))
                })
            })
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
    ) -> Option<(PatternId, usize)> {
        candidates.find_or_first(|(pattern_index, sub_index)| {
            Self::compare_next_index_in_child_pattern(
                child_patterns,
                sub_context,
                pattern_index,
                *sub_index,
            )
        })
    }
}
