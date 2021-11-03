use crate::{
    hypergraph::{
        pattern::*,
        search::*,
        *,
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

fn to_matching_iterator<'a, I: Indexed + 'a, J: Indexed + 'a>(
    a: impl Iterator<Item = &'a I>,
    b: impl Iterator<Item = &'a J>,
) -> impl Iterator<Item = (usize, EitherOrBoth<&'a I, &'a J>)> {
    a.zip_longest(b)
        .enumerate()
        .skip_while(|(_, eob)| match eob {
            EitherOrBoth::Both(a, b) => a.index() == b.index(),
            _ => false,
        })
}
pub trait MatchDirection {
    /// get the parent where vertex is at the relevant position
    fn get_match_parent(vertex: &VertexData, sup: impl Indexed) -> Option<&'_ Parent>;
    fn skip_equal_indices<'a, I: Indexed, J: Indexed>(
        a: impl DoubleEndedIterator<Item = &'a I>,
        b: impl DoubleEndedIterator<Item = &'a J>,
    ) -> Option<(TokenPosition, EitherOrBoth<&'a I, &'a J>)>;
    fn split_head_tail<T: Tokenize>(pattern: &'_ [T]) -> Option<(T, &'_ [T])> {
        Self::pattern_head(pattern).map(|head| (*head, Self::pattern_tail(pattern)))
    }
    /// get remaining pattern in matching direction including index
    fn split_end<T: Tokenize>(pattern: &'_[T], index: PatternId) -> Vec<T>;
    fn split_end_normalized<T: Tokenize>(pattern: &'_[T], index: PatternId) -> Vec<T> {
        Self::split_end(pattern, Self::normalize_index(pattern, index))
    }
    /// get remaining pattern in matching direction excluding index
    fn front_context<T: Tokenize>(pattern: &'_[T], index: PatternId) -> Vec<T>;
    fn front_context_normalized<T: Tokenize>(pattern: &'_[T], index: PatternId) -> Vec<T> {
        Self::front_context(pattern, Self::normalize_index(pattern, index))
    }
    /// get remaining pattern agains matching direction excluding index
    fn back_context<T: Tokenize>(pattern: &'_[T], index: PatternId) -> Vec<T>;
    fn back_context_normalized<T: Tokenize>(pattern: &'_[T], index: PatternId) -> Vec<T> {
        Self::back_context(pattern, Self::normalize_index(pattern, index))
    }
    fn pattern_tail<T: Tokenize>(pattern: &'_[T]) -> &'_[T];
    fn pattern_head<T: Tokenize>(pattern: &'_[T]) -> Option<&T>;
    fn normalize_index<T: Tokenize>(pattern: &'_[T], index: usize) -> usize;
    fn merge_remainder_with_context<
        T: Into<Child> + Tokenize,
        A: IntoPattern<Item=impl Into<Child> + Tokenize, Token=T>,
        B: IntoPattern<Item=impl Into<Child> + Tokenize, Token=T>,
        >(rem: A, context: B) -> Vec<T>;
    fn index_next(index: usize) -> Option<usize>;
    /// filter pattern indices of parent relation by child patterns and matching direction
    fn filter_parent_pattern_indices(
        parent: &Parent,
        child_patterns: &HashMap<PatternId, Pattern>,
    ) -> HashSet<(PatternId, usize)>;
    fn to_found_range(p: Option<Pattern>, context: Pattern) -> FoundRange;
    fn found_at_start(fr: FoundRange) -> bool;
    fn get_remainder(found_range: FoundRange) -> Option<Pattern>;
    fn directed_pattern_split<T: Tokenize>(pattern: &'_[T], index: usize) -> (Vec<T>, Vec<T>) {
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
        vertex.get_parent_at_prefix_of(sup).ok()
    }
    fn skip_equal_indices<'a, I: Indexed, J: Indexed>(
        a: impl DoubleEndedIterator<Item = &'a I>,
        b: impl DoubleEndedIterator<Item = &'a J>,
    ) -> Option<(TokenPosition, EitherOrBoth<&'a I, &'a J>)> {
        to_matching_iterator(a, b).next()
    }
    fn split_end<T: Tokenize>(pattern: &'_[T], index: PatternId) -> Vec<T> {
        postfix(pattern, index)
    }
    fn front_context<T: Tokenize>(pattern: &'_[T], index: PatternId) -> Vec<T> {
        postfix(pattern, index + 1)
    }
    fn back_context<T: Tokenize>(pattern: &'_[T], index: PatternId) -> Vec<T> {
        prefix(pattern, index)
    }
    fn pattern_tail<T: Tokenize>(pattern: &'_[T]) -> &'_[T] {
        pattern.get(1..).unwrap_or(&[])
    }
    fn pattern_head<T: Tokenize>(pattern: &'_[T]) -> Option<&T> {
        pattern.first()
    }
    fn index_next(index: usize) -> Option<usize> {
        index.checked_add(1)
    }
    fn normalize_index<T: Tokenize>(_pattern: &'_[T], index: usize) -> usize {
        index
    }
    fn merge_remainder_with_context<
        T: Into<Child> + Tokenize,
        A: IntoPattern<Item=impl Into<Child> + Tokenize, Token=T>,
        B: IntoPattern<Item=impl Into<Child> + Tokenize, Token=T>,
        >(rem: A, context: B) -> Vec<T> {
        [rem.as_pattern_view(), context.as_pattern_view()].concat()
    }
    fn filter_parent_pattern_indices(
        parent: &Parent,
        _patterns: &HashMap<PatternId, Pattern>,
    ) -> HashSet<(PatternId, usize)> {
        parent
            .filter_pattern_indicies_at_prefix()
            .cloned()
            .collect()
    }
    fn to_found_range(p: Option<Pattern>, context: Pattern) -> FoundRange {
        match (context.is_empty(), p) {
            (false, Some(rem)) => FoundRange::Infix(context, rem),
            (true, Some(rem)) => FoundRange::Prefix(rem),
            (false, None) => FoundRange::Postfix(context),
            (true, None) => FoundRange::Complete,
        }
    }
    fn get_remainder(found_range: FoundRange) -> Option<Pattern> {
        match found_range {
            FoundRange::Prefix(rem) => Some(rem),
            _ => None,
        }
    }
    fn found_at_start(fr: FoundRange) -> bool {
        matches!(fr, FoundRange::Prefix(_) | FoundRange::Complete)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MatchLeft;
impl MatchDirection for MatchLeft {
    fn get_match_parent(vertex: &VertexData, sup: impl Indexed) -> Option<&'_ Parent> {
        vertex.get_parent_at_postfix_of(sup).ok()
    }
    fn skip_equal_indices<'a, I: Indexed, J: Indexed>(
        a: impl DoubleEndedIterator<Item = &'a I>,
        b: impl DoubleEndedIterator<Item = &'a J>,
    ) -> Option<(TokenPosition, EitherOrBoth<&'a I, &'a J>)> {
        to_matching_iterator(a.rev(), b.rev()).next()
    }
    fn split_end<T: Tokenize>(pattern: &'_[T], index: PatternId) -> Vec<T> {
        prefix(pattern, index + 1)
    }
    fn front_context<T: Tokenize>(pattern: &'_[T], index: PatternId) -> Vec<T> {
        prefix(pattern, index)
    }
    fn back_context<T: Tokenize>(pattern: &'_[T], index: PatternId) -> Vec<T> {
        postfix(pattern, index + 1)
    }
    fn pattern_tail<T: Tokenize>(pattern: &'_[T]) -> &'_[T] {
        pattern.split_last().map(|(_last, pre)| pre).unwrap_or(&[])
    }
    fn pattern_head<T: Tokenize>(pattern: &'_[T]) -> Option<&T> {
        pattern.last()
    }
    fn index_next(index: usize) -> Option<usize> {
        index.checked_sub(1)
    }
    fn normalize_index<T: Tokenize>(pattern: &'_[T], index: usize) -> usize {
        pattern.len() - index - 1
    }
    fn merge_remainder_with_context<
        T: Into<Child> + Tokenize,
        A: IntoPattern<Item=impl Into<Child> + Tokenize, Token=T>,
        B: IntoPattern<Item=impl Into<Child> + Tokenize, Token=T>,
        >(rem: A, context: B) -> Vec<T> {
        [context.as_pattern_view(), rem.as_pattern_view()].concat()
    }
    fn filter_parent_pattern_indices(
        parent: &Parent,
        child_patterns: &HashMap<PatternId, Pattern>,
    ) -> HashSet<(PatternId, usize)> {
        parent
            .filter_pattern_indicies_at_end_in_patterns(child_patterns)
            .cloned()
            .collect()
    }
    fn to_found_range(p: Option<Pattern>, context: Pattern) -> FoundRange {
        match (context.is_empty(), p) {
            (false, Some(rem)) => FoundRange::Infix(rem, context),
            (true, Some(rem)) => FoundRange::Postfix(rem),
            (false, None) => FoundRange::Prefix(context),
            (true, None) => FoundRange::Complete,
        }
    }
    fn get_remainder(found_range: FoundRange) -> Option<Pattern> {
        match found_range {
            FoundRange::Postfix(rem) => Some(rem),
            _ => None,
        }
    }
    fn found_at_start(fr: FoundRange) -> bool {
        matches!(fr, FoundRange::Postfix(_) | FoundRange::Complete)
    }
}
