use crate::{
    hypergraph::{
        r#match::*,
        search::*,
        //read::*,
        Child,
        ChildPatterns,
        Hypergraph,
        Indexed,
        Parent,
        Pattern,
        PatternId,
        TokenPosition,
        VertexIndex,
    },
};
//use tokio_stream::{
//    Stream,
//    StreamExt,
//};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SearchFound {
    pub index: Child,
    pub parent_match: ParentMatch,
    pub pattern_id: PatternId,
    pub sub_index: usize,
}
// found range of search pattern in vertex at index
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FoundRange {
    Complete,                // Full index found
    Prefix(Pattern),         // found prefix (remainder)
    Postfix(Pattern),        // found postfix (remainder)
    Infix(Pattern, Pattern), // found infix
}
impl FoundRange {
    pub fn prepend_prefix(self, pattern: Pattern) -> Self {
        if pattern.is_empty() {
            return self;
        }
        match self {
            FoundRange::Complete => FoundRange::Prefix(pattern),
            FoundRange::Prefix(post) => FoundRange::Infix(pattern, post),
            FoundRange::Infix(pre, post) => {
                FoundRange::Infix([&pattern[..], &pre[..]].concat(), post)
            }
            FoundRange::Postfix(pre) => FoundRange::Postfix([&pattern[..], &pre[..]].concat()),
        }
    }
    pub fn is_matching(&self) -> bool {
        self == &FoundRange::Complete
    }
    pub fn reverse(self) -> Self {
        match self {
            Self::Complete => Self::Complete,
            Self::Prefix(post) => Self::Postfix(post),
            Self::Postfix(pre) => Self::Prefix(pre),
            Self::Infix(pre, post) => Self::Infix(post, pre),
        }
    }
}
pub type SearchResult = Result<SearchFound, NotFound>;

pub struct Searcher<'g, T: Tokenize, D: MatchDirection> {
    graph: &'g Hypergraph<T>,
    _ty: std::marker::PhantomData<D>,
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
    pub fn find_sequence(&self, pattern: impl IntoIterator<Item = impl Into<T>>) -> SearchResult {
        let iter = tokenizing_iter(pattern.into_iter());
        let pattern = self.to_token_children(iter)?;
        self.find_pattern(pattern)
    }
    #[allow(unused)]
    pub(crate) fn find_pattern_iter(
        &self,
        pattern: impl IntoIterator<Item=Result<impl Into<Child> + Tokenize, NotFound>>,
    ) -> SearchResult {
        let pattern: Pattern = pattern.into_iter().map(|r| r.map(Into::into)).collect::<Result<Pattern, NotFound>>()?;
        self.find_pattern(pattern)
    }
    pub(crate) fn find_pattern(
        &self,
        pattern: impl IntoPattern<Item=impl Into<Child> + Tokenize>,
    ) -> SearchResult {
        let pattern: Pattern = pattern.into_iter().map(Into::into).collect();
        MatchRight::split_head_tail(&pattern)
            .ok_or(NotFound::EmptyPatterns)
            .and_then(|(head, tail)| {
                if tail.is_empty() {
                    // single index is not a pattern
                    Err(NotFound::SingleIndex)
                } else {
                    self.find_largest_matching_parent(head, tail.to_vec())
                }
            })
    }
    pub fn find_largest_matching_parent(
        &self,
        index: impl Indexed,
        context: impl IntoPattern<Item=impl Into<Child> + Tokenize>,
    ) -> SearchResult {
        self.find_largest_matching_parent_below_width(index, context, None)
    }
    pub fn find_largest_matching_parent_below_width(
        &self,
        index: impl Indexed,
        context: impl IntoPattern<Item=impl Into<Child> + Tokenize>,
        width_ceiling: Option<TokenPosition>,
    ) -> SearchResult {
        let vertex = self.expect_vertex_data(index);
        let parents = vertex.get_parents_below_width(width_ceiling);
        if let Some((&index, child_patterns, parent, pattern_id, sub_index)) =
            self.find_parent_with_matching_children(parents.clone(), context.as_pattern_view())
        {
            self.matcher()
                .compare_child_pattern_at_offset(child_patterns, context, pattern_id, sub_index)
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
                .find_map(|(index, parent)| {
                    self.matcher()
                        .compare_with_parent_children(context.as_pattern_view(), index, parent)
                        .map(|(parent_match, pattern_id, sub_index)| SearchFound {
                            index: Child::new(index, parent.width),
                            pattern_id,
                            sub_index,
                            parent_match,
                        })
                        .ok()
                })
                .ok_or(NotFound::NoMatchingParent)
        }
        .and_then(|search_found| match search_found.parent_match.remainder {
            Some(post) => {
                self.find_largest_matching_parent(search_found.index, post[..].to_vec())
            }
            // todo: match prefixes
            _ => Ok(search_found),
        })
    }
    /// find parent with a child pattern matching context
    pub fn find_parent_with_matching_children(
        &'g self,
        mut parents: impl Iterator<Item = (&'g VertexIndex, &'g Parent)>,
        context: impl IntoPattern<Item=impl Into<Child> + Tokenize>,
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
            let candidates = D::filter_parent_pattern_indices(parent, child_patterns);
            candidates
                .into_iter()
                .find(|(pattern_index, sub_index)| {
                    // find pattern with same next index
                    Matcher::<'g, T, D>::compare_next_index_in_child_pattern(
                        child_patterns,
                        context.as_pattern_view(),
                        pattern_index,
                        *sub_index,
                    )
                })
                .map(|(pattern_index, sub_index)| {
                    (index, child_patterns, parent, pattern_index, sub_index)
                })
        })
    }
}
