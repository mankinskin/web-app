use hypergraph::HyperedgeIndex;

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
    },
    token::Tokenize,
};

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
impl<'g, T: Tokenize, D: MatchDirection> Searcher<'g, T, D> {
    pub fn new(graph: &'g Hypergraph<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    pub(crate) fn matcher(&self) -> Matcher<T, D> {
        Matcher::new(self.graph)
    }
    pub(crate) fn find_parent_matching_postfix(
        &self,
        index: impl Indexed,
        postfix: Pattern,
    ) -> Option<(Child, (PatternId, usize), FoundRange)> {
        let vertex = self.graph.expect_vertex_data(index);
        let width = pattern_width(&postfix) + vertex.width;
        let (index, pattern_id, found_range) = self.matcher()
            .find_parent_matching_context_below_width(&postfix[..], vertex, Some(width + 1))
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
    pub fn find_sequence(
        &self,
        pattern: impl IntoIterator<Item = impl Into<T>>,
    ) -> Option<(Child, (PatternId, usize), FoundRange)> {
        let pattern = T::tokenize(pattern.into_iter());
        let pattern = self.graph.to_token_children(pattern)?;
        self.find_pattern(pattern)
    }
}
