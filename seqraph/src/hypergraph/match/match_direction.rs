use crate::{
    hypergraph::{
        pattern::*,
        r#match::*,
        search::*,
        Child,
        Indexed,
        Parent,
        PatternId,
        TokenPosition,
        VertexData,
    },
};
use itertools::{
    EitherOrBoth,
    Itertools,
};
use std::collections::{
    HashMap,
    HashSet,
};

fn to_matching_iterator<'a>(
    a: impl Iterator<Item = &'a Child>,
    b: impl Iterator<Item = &'a Child>,
) -> impl Iterator<Item = (usize, EitherOrBoth<&'a Child, &'a Child>)> {
    a.zip_longest(b)
        .enumerate()
        .skip_while(|(_, eob)| match eob {
            EitherOrBoth::Both(a, b) => a == b,
            _ => false,
        })
}
pub trait MatchDirection {
    // get the parent where vertex is at the relevant position
    fn get_match_parent(vertex: &VertexData, sup: impl Indexed) -> Option<&'_ Parent>;
    fn skip_equal_indices<'a>(
        a: impl DoubleEndedIterator<Item = &'a Child>,
        b: impl DoubleEndedIterator<Item = &'a Child>,
    ) -> Option<(TokenPosition, EitherOrBoth<&'a Child, &'a Child>)>;
    fn split_head_tail(pattern: PatternView<'_>) -> Option<(Child, PatternView<'_>)> {
        Self::pattern_head(pattern).map(|head| (*head, Self::pattern_tail(pattern)))
    }
    // get remaining pattern in matching direction including index
    fn split_end(pattern: PatternView<'_>, index: PatternId) -> Pattern;
    fn split_end_normalized(pattern: PatternView<'_>, index: PatternId) -> Pattern {
        Self::split_end(pattern, Self::normalize_index(pattern, index))
    }
    // get remaining pattern in matching direction excluding index
    fn front_context(pattern: PatternView<'_>, index: PatternId) -> Pattern;
    fn front_context_normalized(pattern: PatternView<'_>, index: PatternId) -> Pattern {
        Self::front_context(pattern, Self::normalize_index(pattern, index))
    }
    // get remaining pattern agains matching direction excluding index
    fn back_context(pattern: PatternView<'_>, index: PatternId) -> Pattern;
    fn back_context_normalized(pattern: PatternView<'_>, index: PatternId) -> Pattern {
        Self::back_context(pattern, Self::normalize_index(pattern, index))
    }
    fn pattern_tail(pattern: PatternView<'_>) -> PatternView<'_>;
    fn pattern_head(pattern: PatternView<'_>) -> Option<&Child>;
    fn index_next(index: usize) -> Option<usize>;
    fn normalize_index(pattern: PatternView<'_>, index: usize) -> usize;
    fn merge_remainder_with_context<'a>(rem: PatternView<'a>, context: PatternView<'a>) -> Pattern;
    /// filter child patterns by parent relation and matching direction
    fn candidate_parent_pattern_indices(
        parent: &Parent,
        child_patterns: &HashMap<PatternId, Pattern>,
    ) -> HashSet<(PatternId, usize)>;
    fn to_found_range(p: PatternMatch, context: Pattern) -> FoundRange;
    fn directed_pattern_split(pattern: PatternView<'_>, index: usize) -> (Pattern, Pattern) {
        (
            Self::back_context(pattern, index),
            Self::split_end(pattern, index),
        )
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MatchRight;
impl MatchDirection for MatchRight {
    fn get_match_parent(vertex: &VertexData, sup: impl Indexed) -> Option<&'_ Parent> {
        vertex.get_parent_at_prefix_of(sup)
    }
    fn skip_equal_indices<'a>(
        a: impl DoubleEndedIterator<Item = &'a Child>,
        b: impl DoubleEndedIterator<Item = &'a Child>,
    ) -> Option<(TokenPosition, EitherOrBoth<&'a Child, &'a Child>)> {
        to_matching_iterator(a, b).next()
    }
    fn split_end(pattern: PatternView<'_>, index: PatternId) -> Pattern {
        postfix(pattern, index)
    }
    fn front_context(pattern: PatternView<'_>, index: PatternId) -> Pattern {
        postfix(pattern, index + 1)
    }
    fn back_context(pattern: PatternView<'_>, index: PatternId) -> Pattern {
        prefix(pattern, index)
    }
    fn pattern_tail(pattern: PatternView<'_>) -> PatternView<'_> {
        pattern.get(1..).unwrap_or(&[])
    }
    fn pattern_head(pattern: PatternView<'_>) -> Option<&Child> {
        pattern.first()
    }
    fn index_next(index: usize) -> Option<usize> {
        index.checked_add(1)
    }
    fn normalize_index(_pattern: PatternView<'_>, index: usize) -> usize {
        index
    }
    fn merge_remainder_with_context<'a>(rem: PatternView<'a>, context: PatternView<'a>) -> Pattern {
        [rem, context].concat()
    }
    fn candidate_parent_pattern_indices(
        parent: &Parent,
        _: &HashMap<PatternId, Pattern>,
    ) -> HashSet<(PatternId, usize)> {
        parent
            .filter_pattern_indicies_at_prefix()
            .cloned()
            .collect()
    }
    fn to_found_range(p: PatternMatch, context: Pattern) -> FoundRange {
        p.prepend_prefix(context)
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MatchLeft;
impl MatchDirection for MatchLeft {
    fn get_match_parent(vertex: &VertexData, sup: impl Indexed) -> Option<&'_ Parent> {
        vertex.get_parent_at_postfix_of(sup)
    }
    fn skip_equal_indices<'a>(
        a: impl DoubleEndedIterator<Item = &'a Child>,
        b: impl DoubleEndedIterator<Item = &'a Child>,
    ) -> Option<(TokenPosition, EitherOrBoth<&'a Child, &'a Child>)> {
        to_matching_iterator(a.rev(), b.rev()).next()
    }
    fn split_end(pattern: PatternView<'_>, index: PatternId) -> Pattern {
        prefix(pattern, index + 1)
    }
    fn front_context(pattern: PatternView<'_>, index: PatternId) -> Pattern {
        prefix(pattern, index)
    }
    fn back_context(pattern: PatternView<'_>, index: PatternId) -> Pattern {
        postfix(pattern, index + 1)
    }
    fn pattern_tail(pattern: PatternView<'_>) -> PatternView<'_> {
        pattern.split_last().map(|(_last, pre)| pre).unwrap_or(&[])
    }
    fn pattern_head(pattern: PatternView<'_>) -> Option<&Child> {
        pattern.last()
    }
    fn index_next(index: usize) -> Option<usize> {
        index.checked_sub(1)
    }
    fn normalize_index(pattern: PatternView<'_>, index: usize) -> usize {
        pattern.len() - index - 1
    }
    fn merge_remainder_with_context(rem: PatternView<'_>, context: PatternView<'_>) -> Pattern {
        [context, rem].concat()
    }
    fn candidate_parent_pattern_indices(
        parent: &Parent,
        child_patterns: &HashMap<PatternId, Pattern>,
    ) -> HashSet<(PatternId, usize)> {
        parent
            .filter_pattern_indicies_at_end_in_patterns(child_patterns)
            .cloned()
            .collect()
    }
    fn to_found_range(p: PatternMatch, context: Pattern) -> FoundRange {
        p.prepend_prefix(context).reverse()
    }
}