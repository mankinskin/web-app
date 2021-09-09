use crate::hypergraph::*;
use std::{
    borrow::Borrow,
    slice::SliceIndex,
    ops::RangeBounds,
};
pub type Pattern = Vec<Child>;
pub type PatternView<'a> = &'a[Child];
pub trait PatternRangeIndex: SliceIndex<[Child], Output=[Child]> + RangeBounds<usize> + Iterator<Item=usize> {}
impl<T: SliceIndex<[Child], Output=[Child]> + RangeBounds<usize> + Iterator<Item=usize>> PatternRangeIndex for T {}

pub fn pattern_width<T:Borrow<Child>>(pat: impl IntoIterator<Item=T>) -> TokenPosition {
    pat.into_iter().fold(0, |acc, child| acc + child.borrow().get_width())
}
pub fn prefix(
    pattern: PatternView<'_>,
    index: PatternId,
    ) -> Pattern {
    pattern.get(..index)
        .unwrap_or(pattern)
        .to_vec()
}
pub fn postfix(
    pattern: PatternView<'_>,
    index: PatternId,
    ) -> Pattern {
    pattern.get(index..)
        .unwrap_or(&[])
        .to_vec()
}
pub fn replace_in_pattern(
    pattern: impl IntoIterator<Item=Child>,
    range: impl PatternRangeIndex,
    replace: impl IntoIterator<Item=impl Into<Child>>,
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