use crate::{
    vertex::*,
    Hypergraph,
};
use std::{
    collections::BTreeSet,
    num::NonZeroUsize,
    ops::Bound,
};
mod index_splitter;
pub use index_splitter::*;

impl<'t, 'a, T> Hypergraph<T>
where
    T: Tokenize + 't,
{
    /// Split an index the specified position
    pub fn split_index(&mut self, root: impl Indexed, pos: NonZeroUsize) -> SingleSplitResult {
        IndexSplitter::new(self).split_index(root, pos)
    }
    // create index from token position range in index
    pub fn index_subrange(
        &mut self,
        root: impl Indexed + Clone,
        range: impl PatternRangeIndex,
    ) -> RangeSplitResult {
        IndexSplitter::new(self).index_subrange(root, range)
    }
    pub fn index_prefix(&mut self, root: impl Indexed, pos: NonZeroUsize) -> (Child, SplitSegment) {
        IndexSplitter::new(self).index_prefix(root, pos)
    }
    pub fn index_postfix(
        &mut self,
        root: impl Indexed,
        pos: NonZeroUsize,
    ) -> (SplitSegment, Child) {
        IndexSplitter::new(self).index_postfix(root, pos)
    }
}
impl<'t, 'a, T> Hypergraph<T>
where
    T: Tokenize + 't + std::fmt::Display,
{
    fn pattern_split_string_width(&self, split: &PatternSplit) -> usize {
        let left = self.pattern_string_with_separator(&split.prefix, ".");
        let right = self.pattern_string_with_separator(&split.postfix, ".");
        left.len() + self.index_split_string_width(&split.inner) + right.len()
    }
    fn index_split_string_width(&self, split: &IndexSplit) -> usize {
        split
            .splits
            .first()
            .map(|s| self.pattern_split_string_width(s))
            .unwrap_or(1)
    }
    fn pattern_split_string_with_offset_and_width(
        &self,
        split: &PatternSplit,
        offset: usize,
        width: usize,
    ) -> String {
        let left = self.pattern_string_with_separator(&split.prefix, ".");
        let right = self.pattern_string_with_separator(&split.postfix, ".");
        let next_width = (width - left.len()) - right.len();
        let next_offset = offset + left.len();

        let inner = split.inner.splits.iter().fold(String::new(), |acc, s| {
            format!(
                "{}{}\n",
                acc,
                self.pattern_split_string_with_offset_and_width(s, next_offset, next_width,),
            )
        });
        format!(
            "{}\n{}",
            std::iter::repeat(' ')
                .take(offset)
                .chain(left.chars())
                .chain(std::iter::repeat(' ').take(next_width + 1))
                .chain(right.chars())
                .collect::<String>(),
            inner,
        )
    }
    fn index_split_string(&self, split: &IndexSplit) -> String {
        let width = self.index_split_string_width(split);
        split.splits.iter().fold(String::new(), |acc, split| {
            format!(
                "{}{}\n",
                acc,
                self.pattern_split_string_with_offset_and_width(split, 0, width),
            )
        })
    }
    pub fn print_index_split(&self, split: &IndexSplit) {
        print!("{}", self.index_split_string(split));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        r#match::*,
        search::*,
    };
    use maplit::hashset;
    use pretty_assertions::assert_eq;
    use std::collections::HashSet;
    #[test]
    fn build_child_splits_1() {
        let mut graph = Hypergraph::default();
        if let [a, b, c, d] = graph.insert_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('c'),
            Token::Element('d'),
        ])[..]
        {
            // wxabyzabbyxabyz
            let ab = graph.insert_pattern([a, b]);
            let bc = graph.insert_pattern([b, c]);
            let cd = graph.insert_pattern([c, d]);
            let abc = graph.insert_patterns([vec![ab, c], vec![a, bc]]);
            let bcd = graph.insert_patterns([vec![bc, d], vec![b, cd]]);
            let abcd = graph.insert_patterns([vec![abc, d], vec![a, bcd]]);

            let expleft = hashset![
                (vec![a], SplitSegment::Child(b)),
                (vec![], SplitSegment::Child(ab)),
            ];
            let expright = hashset![
                (vec![d], SplitSegment::Child(c)),
                (vec![], SplitSegment::Child(cd)),
            ];
            let mut splitter = IndexSplitter::new(&mut graph);
            let (ps, child_splits) =
                splitter.separate_perfect_split(abcd, NonZeroUsize::new(2).unwrap());
            assert_eq!(ps, None);
            let (left, right) = splitter.build_child_splits(child_splits);
            let (left, right): (HashSet<_>, HashSet<_>) =
                (left.into_iter().collect(), right.into_iter().collect());
            assert_eq!(left, expleft, "left");
            assert_eq!(right, expright, "right");
        } else {
            panic!();
        }
    }
    #[test]
    fn build_child_splits_2() {
        let mut graph = Hypergraph::default();
        if let [a, b, _w, x, y, z] = graph.insert_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('w'),
            Token::Element('x'),
            Token::Element('y'),
            Token::Element('z'),
        ])[..]
        {
            // wxabyzabbyxabyz
            let ab = graph.insert_pattern([a, b]);
            let by = graph.insert_pattern([b, y]);
            let yz = graph.insert_pattern([y, z]);
            let xab = graph.insert_pattern([x, ab]);
            let xaby = graph.insert_patterns([vec![xab, y], vec![x, a, by]]);
            let xabyz = graph.insert_patterns([vec![xaby, z], vec![xab, yz]]);

            let mut splitter = IndexSplitter::new(&mut graph);
            let (ps, child_splits) =
                splitter.separate_perfect_split(xabyz, NonZeroUsize::new(2).unwrap());
            assert_eq!(ps, None);
            let (left, right) = splitter.build_child_splits(child_splits);
            let xa_found = graph.find_pattern(vec![x, a]);
            let xa = if let SearchFound {
                index: xa,
                parent_match:
                    ParentMatch {
                        parent_range: FoundRange::Complete,
                        ..
                    },
                ..
            } = xa_found.expect("xa not found")
            {
                Some(xa)
            } else {
                None
            }
            .unwrap();

            let expleft = hashset![(vec![], SplitSegment::Child(xa)),];
            let expright = hashset![
                (vec![yz], SplitSegment::Child(b)),
                (vec![z], SplitSegment::Child(by)),
            ];

            let (left, right): (HashSet<_>, HashSet<_>) =
                (left.into_iter().collect(), right.into_iter().collect());
            assert_eq!(left, expleft, "left");
            assert_eq!(right, expright, "right");
        } else {
            panic!();
        }
    }
}
