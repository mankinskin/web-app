use crate::{
    r#match::*,
    search::*,
    Hypergraph,
};
//use tokio_stream::{
//    Stream,
//    StreamExt,
//};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SearchFound {
    pub index: Child,
    pub pattern_id: PatternId,
    pub sub_index: usize,
    pub parent_match: ParentMatch,
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
    pub fn embed_in_super(self, other: Self) -> Self {
        match (self, other) {
            (Self::Complete, outer) => outer,
            (inner, Self::Complete) => inner,
            (Self::Prefix(inner), Self::Postfix(outer)) => Self::Infix(outer, inner),
            (Self::Prefix(inner), Self::Prefix(outer)) => Self::Prefix([inner, outer].concat()),
            (Self::Prefix(inner), Self::Infix(louter, router)) => {
                Self::Infix(louter, [inner, router].concat())
            }
            (Self::Postfix(inner), Self::Prefix(outer)) => Self::Infix(inner, outer),
            (Self::Postfix(inner), Self::Postfix(outer)) => Self::Postfix([outer, inner].concat()),
            (Self::Postfix(inner), Self::Infix(louter, router)) => {
                Self::Infix([louter, inner].concat(), router)
            }
            (Self::Infix(linner, rinner), Self::Prefix(outer)) => {
                Self::Infix(linner, [rinner, outer].concat())
            }
            (Self::Infix(linner, rinner), Self::Postfix(outer)) => {
                Self::Infix([outer, linner].concat(), rinner)
            }
            (Self::Infix(linner, rinner), Self::Infix(louter, router)) => {
                Self::Infix([louter, linner].concat(), [rinner, router].concat())
            }
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
        pattern: impl IntoIterator<Item = Result<impl Into<Child> + Tokenize, NotFound>>,
    ) -> SearchResult {
        let pattern: Pattern = pattern
            .into_iter()
            .map(|r| r.map(Into::into))
            .collect::<Result<Pattern, NotFound>>()?;
        self.find_pattern(pattern)
    }
    pub(crate) fn find_pattern(
        &self,
        pattern: impl IntoPattern<Item = impl Into<Child> + Tokenize>,
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
        context: impl IntoPattern<Item = impl Into<Child> + Tokenize>,
    ) -> SearchResult {
        self.find_largest_matching_parent_below_width(index, context, None)
    }
    pub fn find_largest_matching_parent_below_width(
        &self,
        index: impl Indexed,
        context: impl IntoPattern<Item = impl Into<Child> + Tokenize>,
        width_ceiling: Option<TokenPosition>,
    ) -> SearchResult {
        let vertex_index = *index.index();
        let vertex = self.expect_vertex_data(index);
        let parents = vertex.get_parents_below_width(width_ceiling);
        if let Some((&index, child_patterns, parent, pattern_id, sub_index)) =
            self.find_direct_matching_parent(parents.clone(), context.as_pattern_view())
        {
            // direct matching parent
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
            // no direct matching parent
            // compare all parent's children
            self.find_indirect_matching_parent(parents, context)
                .ok_or(NotFound::NoMatchingParent(vertex_index))
        }
        .and_then(|search_found| {
            if let Some(rem) = &search_found.parent_match.remainder {
                self.find_largest_matching_parent(search_found.index, rem)
                    .map(|super_found| SearchFound {
                        parent_match: search_found
                            .parent_match
                            .embed_in_super(super_found.parent_match),
                        index: super_found.index,
                        sub_index: super_found.sub_index,
                        pattern_id: super_found.pattern_id,
                    })
            } else {
                Ok(search_found)
            }
        })
    }
    /// find parent with a child pattern matching context
    pub fn find_direct_matching_parent(
        &'g self,
        mut parents: impl Iterator<Item = (&'g VertexIndex, &'g Parent)>,
        context: impl IntoPattern<Item = impl Into<Child> + Tokenize>,
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
    /// find parent with a child pattern matching context
    pub fn find_indirect_matching_parent(
        &'g self,
        mut parents: impl Iterator<Item = (&'g VertexIndex, &'g Parent)>,
        context: impl IntoPattern<Item = impl Into<Child> + Tokenize>,
    ) -> Option<SearchFound> {
        let context = context.as_pattern_view();
        let mut first = None;
        parents
            .find_map(|(index, parent)| {
                first.get_or_insert((index, parent));
                self.matcher()
                    .match_indirect_parent(*index, parent, context)
            })
            .or_else(|| {
                first.and_then(|(index, parent)| {
                    parent
                        .pattern_indices
                        .iter()
                        .next()
                        .cloned()
                        .ok_or(PatternMismatch::NoParents)
                        .and_then(|(pattern_index, sub_index)| {
                            let vert = self.expect_vertex_data(index);
                            let child_patterns = vert.get_children();
                            self.matcher()
                                .compare_child_pattern_at_offset(
                                    child_patterns,
                                    context,
                                    pattern_index,
                                    sub_index,
                                )
                                .map(|parent_match| (parent_match, pattern_index, sub_index))
                        })
                        .map(|(parent_match, pattern_id, sub_index)| SearchFound {
                            index: Child::new(index, parent.width),
                            pattern_id,
                            sub_index,
                            parent_match,
                        })
                        .ok()
                })
            })
    }
}
