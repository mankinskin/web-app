use crate::hypergraph::*;
use std::{
    borrow::Borrow,
    fmt::Debug,
    ops::RangeBounds,
    slice::SliceIndex,
};
pub type Pattern = Vec<Child>;
pub type PatternView<'a> = &'a [Child];
pub trait PatternRangeIndex:
    SliceIndex<[Child], Output = [Child]> + RangeBounds<usize> + Iterator<Item = usize> + Debug
{
}
impl<
        T: SliceIndex<[Child], Output = [Child]> + RangeBounds<usize> + Iterator<Item = usize> + Debug,
    > PatternRangeIndex for T
{
}
pub trait IntoPattern: IntoIterator + Sized
    where <Self as IntoIterator>::Item: Into<Child>,
{
    fn into_pattern(self) -> Pattern {
        self.into_iter().map(Into::into).collect()
    }
    fn as_pattern_view(&'_ self) -> &'_ [Child];
    fn is_empty(&self) -> bool;
}
impl IntoPattern for Pattern {
    fn as_pattern_view(&'_ self) -> &'_ [Child] {
        self.as_slice()
    }
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}
impl IntoPattern for &'_ Pattern {
    fn as_pattern_view(&'_ self) -> &'_ [Child] {
        self.as_slice()
    }
    fn is_empty(&self) -> bool {
        (*self).is_empty()
    }
}
impl IntoPattern for PatternView<'_> {
    fn as_pattern_view(&'_ self) -> &'_ [Child] {
        self
    }
    fn is_empty(&self) -> bool {
        (*self).is_empty()
    }
}

pub fn pattern_width<T: Borrow<Child>>(pat: impl IntoIterator<Item = T>) -> TokenPosition {
    pat.into_iter()
        .fold(0, |acc, child| acc + child.borrow().get_width())
}
pub fn prefix(pattern: PatternView<'_>, index: PatternId) -> Pattern {
    pattern.get(..index).unwrap_or(pattern).to_vec()
}
pub fn infix(pattern: PatternView<'_>, start: PatternId, end: PatternId) -> Pattern {
    pattern.get(start..end).unwrap_or(&[]).to_vec()
}
pub fn postfix(pattern: PatternView<'_>, index: PatternId) -> Pattern {
    pattern.get(index..).unwrap_or(&[]).to_vec()
}
pub fn replace_in_pattern(
    pattern: impl IntoIterator<Item = Child>,
    range: impl PatternRangeIndex,
    replace: impl IntoIterator<Item = impl Into<Child>>,
) -> Pattern {
    let mut pattern: Pattern = pattern.into_iter().collect();
    pattern.splice(range, replace.into_iter().map(Into::into));
    pattern
}
pub fn single_child_patterns(halves: Vec<Pattern>) -> Result<Child, Vec<Pattern>> {
    match (halves.len(), halves.first()) {
        (1, Some(first)) => single_child_pattern(first.clone()).map_err(|_| halves),
        _ => Err(halves),
    }
}
pub fn single_child_pattern(half: Pattern) -> Result<Child, Pattern> {
    match (half.len(), half.first()) {
        (1, Some(first)) => Ok(*first),
        _ => Err(half),
    }
}
/// Split a pattern before the specified index
pub fn split_pattern_at_index(pattern: PatternView<'_>, index: PatternId) -> (Pattern, Pattern) {
    (prefix(pattern, index), postfix(pattern, index))
}
pub fn split_context(pattern: PatternView<'_>, index: PatternId) -> (Pattern, Pattern) {
    (prefix(pattern, index), postfix(pattern, index + 1))
}
pub fn double_split_context(
    pattern: PatternView<'_>,
    left_index: PatternId,
    right_index: PatternId,
) -> (Pattern, Pattern, Pattern) {
    let (prefix, rem) = split_context(pattern, left_index);
    if left_index < right_index {
        let (infix, postfix) = split_context(&rem, right_index - (left_index + 1));
        (prefix, infix, postfix)
    } else {
        (prefix, vec![], rem)
    }
}
