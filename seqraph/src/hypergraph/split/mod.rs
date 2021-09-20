use crate::{
    hypergraph::{
        pattern::*,
        vertex::*,
        Child,
        Hypergraph,
        PatternId,
        TokenPosition,
        VertexIndex,
    },
    token::Tokenize,
};
use std::{
    cmp::PartialEq,
    collections::BTreeSet,
    num::NonZeroUsize,
    ops::Bound,
};
mod index_splitter;
mod split_minimizer;
pub use {
    index_splitter::*,
    split_minimizer::SplitMinimizer,
};

pub type Split = (Pattern, Pattern);
/// refers to position of child in parent
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ContextChild {
    pub child: usize, // child node in the sub graph
    pub index_in_parent: IndexInParent,
}
/// refers to position of child in parent
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ContextParent {
    pub parent: usize, // parent node in the sub graph
    pub index_in_parent: IndexInParent,
}
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct SplitContext {
    pub prefix: Pattern,
    pub key: SplitKey,
    pub postfix: Pattern,
}
/// refers to an index in a hypergraph node
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct IndexInParent {
    pub pattern_index: usize,  // index of pattern in parent
    pub replaced_index: usize, // replaced index in pattern
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SplitIndex {
    pos: TokenPosition,
    index: VertexIndex,
    index_pos: IndexPosition,
}
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct PatternSplit {
    pub(crate) prefix: Pattern,
    pub(crate) inner: IndexSplit,
    pub(crate) postfix: Pattern,
}
impl PatternSplit {
    pub fn new(prefix: Pattern, inner: impl Into<IndexSplit>, postfix: Pattern) -> Self {
        Self {
            prefix,
            inner: inner.into(),
            postfix,
        }
    }
}
#[derive(Debug, Clone, Eq, Ord, PartialOrd, Default)]
pub struct IndexSplit {
    pub(crate) splits: Vec<PatternSplit>,
}
impl IndexSplit {
    pub fn new(inner: impl IntoIterator<Item = impl Into<PatternSplit>>) -> Self {
        Self {
            splits: inner.into_iter().map(Into::into).collect(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.splits.is_empty()
    }
    pub fn add_split<T: Into<PatternSplit>>(&mut self, split: T) {
        self.splits.push(split.into());
    }
}
impl PartialEq for IndexSplit {
    fn eq(&self, other: &Self) -> bool {
        let a: BTreeSet<_> = self.splits.iter().collect();
        let b: BTreeSet<_> = other.splits.iter().collect();
        a == b
    }
}
impl From<Split> for PatternSplit {
    fn from((prefix, postfix): Split) -> Self {
        Self {
            prefix,
            inner: Default::default(),
            postfix,
        }
    }
}
impl<T: Into<IndexSplit>> From<(Pattern, T, Pattern)> for PatternSplit {
    fn from((prefix, inner, postfix): (Pattern, T, Pattern)) -> Self {
        Self::new(prefix, inner, postfix)
    }
}
impl<T: Into<PatternSplit>> From<Vec<T>> for IndexSplit {
    fn from(splits: Vec<T>) -> Self {
        Self {
            splits: splits.into_iter().map(Into::into).collect(),
        }
    }
}
impl<T: Into<PatternSplit>> From<T> for IndexSplit {
    fn from(split: T) -> Self {
        Self::from(vec![split])
    }
}
impl<'t, 'a, T> Hypergraph<T>
where
    T: Tokenize + 't,
{
    /// Get perfect split if it exists and remaining pattern split contexts
    pub(crate) fn separate_perfect_split(
        &self,
        root: impl Indexed,
        pos: NonZeroUsize,
    ) -> (Option<(Split, IndexInParent)>, Vec<SplitContext>) {
        let current_node = self.expect_vertex_data(root);
        let children = current_node.get_children().clone();
        let child_slices = children.into_iter().map(|(i, p)| (i, p.into_iter()));
        let split_indices = IndexSplitter::find_single_split_indices(child_slices, pos);
        IndexSplitter::separate_single_split_indices(current_node, split_indices)
    }
    /// Get perfect split or pattern split contexts
    #[allow(unused)]
    pub(crate) fn try_perfect_split(
        &self,
        root: impl Indexed,
        pos: NonZeroUsize,
    ) -> Result<(Split, IndexInParent), Vec<SplitContext>> {
        let current_node = self.get_vertex_data(root).unwrap();
        let children = current_node.get_children().clone();
        let child_slices = children.into_iter().map(|(i, p)| (i, p.into_iter()));
        let split_indices = IndexSplitter::find_single_split_indices(child_slices, pos);
        match IndexSplitter::perfect_split_search(current_node, split_indices)
            .into_iter()
            .collect()
        {
            Ok(s) => Err(s),
            Err(s) => Ok(s),
        }
    }
}
impl<'t, 'a, T> Hypergraph<T>
where
    T: Tokenize + 't + std::fmt::Display,
{
    /// Split an index the specified position
    pub fn split_index(&mut self, root: impl Indexed + Clone, pos: NonZeroUsize) -> (Child, Child) {
        IndexSplitter::split_index(self, root, pos)
    }
    // create index from token position range in index
    pub fn index_subrange(
        &mut self,
        root: impl Indexed + Clone,
        range: impl PatternRangeIndex,
    ) -> RangeSplitResult {
        IndexSplitter::index_subrange(self, root, range)
    }

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
        hypergraph::{
            child_strings::*,
            tests::context_mut,
        },
        token::*,
    };
    use pretty_assertions::assert_eq;
    #[test]
    fn split_range_1() {
        let mut graph = Hypergraph::default();
        if let [a, b, w, x, y, z] = graph.insert_tokens([
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
            let wx = graph.insert_pattern([w, x]);
            let xab = graph.insert_pattern([x, ab]);
            let xaby = graph.insert_patterns([vec![xab, y], vec![x, a, by]]);
            let wxab = graph.insert_patterns([vec![wx, ab], vec![w, xab]]);
            let wxaby = graph.insert_patterns([vec![w, xaby], vec![wx, a, by], vec![wxab, y]]);
            let xabyz = graph.insert_patterns([vec![xaby, z], vec![xab, yz]]);
            let wxabyz = graph.insert_patterns([vec![w, xabyz], vec![wxaby, z], vec![wx, ab, yz]]);

            let _ = graph.index_subrange(wxabyz, 0..);
            let _ = graph.index_subrange(wxabyz, 1..);
            let _ = graph.index_subrange(wxabyz, 1..3);
            let _ = graph.index_subrange(wxabyz, 2..5);
            let _ = graph.index_subrange(wxabyz, 3..);
        } else {
            panic!();
        }
    }
    //    #[test]
    //    fn split_index_1() {
    //        let (
    //            graph,
    //            _a,
    //            _b,
    //            c,
    //            _d,
    //            _e,
    //            _f,
    //            _g,
    //            _h,
    //            _i,
    //            ab,
    //            _bc,
    //            _cd,
    //            _bcd,
    //            abc,
    //            _abcd,
    //            _cdef,
    //            _efghi,
    //            _abab,
    //            _ababab,
    //            _ababababcdefghi,
    //            ) = &mut *context_mut();
    //        let (left, right) = graph.split_index_at_pos(*abc, NonZeroUsize::new(2).unwrap());
    //        assert_eq!(left, vec![Child::new(*ab, 2)], "left");
    //        assert_eq!(right, vec![Child::new(*c, 1)], "right");
    //    }
    //    #[test]
    //    fn split_child_patterns_2() {
    //        let (
    //            graph,
    //            _a,
    //            _b,
    //            _c,
    //            d,
    //            _e,
    //            _f,
    //            _g,
    //            _h,
    //            _i,
    //            _ab,
    //            _bc,
    //            _cd,
    //            _bcd,
    //            abc,
    //            abcd,
    //            _cdef,
    //            _efghi,
    //            _abab,
    //            _ababab,
    //            _ababababcdefghi,
    //            ) = &mut *context_mut();
    //        let (left, right) = graph.split_index_at_pos(*abcd, NonZeroUsize::new(3).unwrap());
    //        assert_eq!(left, vec![Child::new(*abc, 3)], "left");
    //        assert_eq!(right, vec![Child::new(*d, 1)], "right");
    //    }
    //    use crate::token::*;
    //    fn split_child_patterns_3_impl() {
    //        let mut graph = Hypergraph::default();
    //        if let [a, b, c, d] = graph.insert_tokens(
    //            [
    //                Token::Element('a'),
    //                Token::Element('b'),
    //                Token::Element('c'),
    //                Token::Element('d'),
    //            ])[..] {
    //            // wxabyzabbyxabyz
    //            let ab = graph.insert_pattern([a, b]);
    //            let bc = graph.insert_pattern([b, c]);
    //            let cd = graph.insert_pattern([c, d]);
    //            let abc = graph.insert_patterns([
    //                vec![ab, c],
    //                vec![a, bc]
    //            ]);
    //            let bcd = graph.insert_patterns([
    //                vec![bc, d],
    //                vec![b, cd]
    //            ]);
    //            let abcd = graph.insert_patterns([
    //                vec![abc, d],
    //                vec![a, bcd]
    //            ]);
    //            let ab_pattern = vec![Child::new(ab, 2)];
    //            let cd_pattern = vec![Child::new(cd, 2)];
    //
    //            let (left, right) = graph.split_index_at_pos(abcd, NonZeroUsize::new(2).unwrap());
    //            assert_eq!(left, ab_pattern, "left");
    //            assert_eq!(right, cd_pattern, "right");
    //        } else {
    //            panic!();
    //        }
    //    }
    //    #[test]
    //    fn split_child_patterns_3() {
    //        split_child_patterns_3_impl()
    //    }
    //    #[test]
    //    fn split_child_patterns_4() {
    //        let mut graph = Hypergraph::default();
    //        if let [a, b, _w, x, y, z] = graph.insert_tokens(
    //            [
    //                Token::Element('a'),
    //                Token::Element('b'),
    //                Token::Element('w'),
    //                Token::Element('x'),
    //                Token::Element('y'),
    //                Token::Element('z'),
    //            ])[..] {
    //            // wxabyzabbyxabyz
    //            let ab = graph.insert_pattern([a, b]);
    //            let by = graph.insert_pattern([b, y]);
    //            let yz = graph.insert_pattern([y, z]);
    //            let xab = graph.insert_pattern([x, ab]);
    //            let xaby = graph.insert_patterns([
    //                vec![xab, y],
    //                vec![x, a, by]
    //            ]);
    //            let xabyz = graph.insert_patterns([
    //                vec![xaby, z],
    //                vec![xab, yz]
    //            ]);
    //            // split xabyz at 2
    //            let xa_graph = ChildStrings::from_node(
    //                "xa",
    //                vec![
    //                    vec!["x", "a"],
    //                ]
    //            );
    //            let byz_graph = ChildStrings::from_node(
    //                "byz",
    //                vec![
    //                    vec!["by", "z"],
    //                    vec!["b", "yz"],
    //                ]
    //            );
    //
    //            let (left, right) = graph.split_index_at_pos(xabyz, NonZeroUsize::new(2).unwrap());
    //            let left = graph.pattern_child_strings(left);
    //            let right = graph.pattern_child_strings(right);
    //            assert_eq!(left, xa_graph, "left");
    //            assert_eq!(right, byz_graph, "right");
    //        } else {
    //            panic!();
    //        }
    //    }
    //    #[test]
    //    fn split_child_patterns_5() {
    //        let mut graph = Hypergraph::default();
    //        if let [a, b, w, x, y, z] = graph.insert_tokens(
    //            [
    //                Token::Element('a'),
    //                Token::Element('b'),
    //                Token::Element('w'),
    //                Token::Element('x'),
    //                Token::Element('y'),
    //                Token::Element('z'),
    //            ])[..] {
    //            // wxabyzabbyxabyz
    //            let ab = graph.insert_pattern([a, b]);
    //            let by = graph.insert_pattern([b, y]);
    //            let yz = graph.insert_pattern([y, z]);
    //            let xa = graph.insert_pattern([x, a]);
    //            let xab = graph.insert_patterns([
    //                vec![x, ab],
    //                vec![xa, b],
    //            ]);
    //            let xaby = graph.insert_patterns([
    //                vec![xab, y],
    //                vec![xa, by]
    //            ]);
    //            let xabyz = graph.insert_patterns([
    //                vec![xaby, z],
    //                vec![xab, yz]
    //            ]);
    //            let wxabyzabbyxabyz = graph.insert_pattern([w, xabyz, ab, by, xabyz]);
    //
    //            // split wxabyzabbyxabyz at 3
    //            let wxa_graph = ChildStrings::from_node(
    //                "wxa",
    //                vec![
    //                    vec!["w", "xa"],
    //                ]
    //            );
    //            let byzabbyxabyz_graph = ChildStrings::from_node(
    //                "byzabbyxabyz",
    //                vec![
    //                    ["byz", "abbyxabyz"],
    //                ]
    //            );
    //            let (left, right) = graph.split_index_at_pos(wxabyzabbyxabyz, NonZeroUsize::new(3).unwrap());
    //            let left = graph.pattern_child_strings(left);
    //            let right = graph.pattern_child_strings(right);
    //            assert_eq!(left, wxa_graph, "left");
    //            assert_eq!(right, byzabbyxabyz_graph, "right");
    //        } else {
    //            panic!();
    //        }
    //    }
    //    fn split_child_patterns_6_impl() {
    //        let mut graph = Hypergraph::default();
    //        if let [a, b, w, x, y, z] = graph.insert_tokens(
    //            [
    //                Token::Element('a'),
    //                Token::Element('b'),
    //                Token::Element('w'),
    //                Token::Element('x'),
    //                Token::Element('y'),
    //                Token::Element('z'),
    //            ])[..] {
    //            // wxabyzabbyxabyz
    //            let ab = graph.insert_pattern([a, b]);
    //            let by = graph.insert_pattern([b, y]);
    //            let yz = graph.insert_pattern([y, z]);
    //            let wx = graph.insert_pattern([w, x]);
    //            let xab = graph.insert_pattern([x, ab]);
    //            let xaby = graph.insert_patterns([
    //                vec![xab, y],
    //                vec![x, a, by]
    //            ]);
    //            let wxab = graph.insert_patterns([
    //                vec![wx, ab],
    //                vec![w, xab]
    //            ]);
    //            let wxaby = graph.insert_patterns([
    //                vec![w, xaby],
    //                vec![wx, a, by],
    //                vec![wxab, y]
    //            ]);
    //            let xabyz = graph.insert_patterns([
    //                vec![xaby, z],
    //                vec![xab, yz]
    //            ]);
    //            let wxabyz = graph.insert_patterns([
    //                vec![w, xabyz],
    //                vec![wxaby, z],
    //                vec![wx, ab, yz]
    //            ]);
    //            let wxa_graph = ChildStrings::from_node(
    //                "wxa",
    //                vec![
    //                    vec!["wx", "a"],
    //                ]
    //            );
    //            let byz_graph = ChildStrings::from_node(
    //                "byz",
    //                vec![
    //                    vec!["by", "z"],
    //                    vec!["b", "yz"],
    //                ]
    //            );
    //
    //            let (left, right) = graph.split_index_at_pos(wxabyz, NonZeroUsize::new(3).unwrap());
    //            let left = graph.pattern_child_strings(left);
    //            let right = graph.pattern_child_strings(right);
    //            assert_eq!(left, wxa_graph, "left");
    //            assert_eq!(right, byz_graph, "right");
    //        } else {
    //            panic!();
    //        }
    //    }
    //    #[test]
    //    fn split_child_patterns_6() {
    //        split_child_patterns_6_impl()
    //    }
    //    //#[bench]
    //    //fn bench_split_child_patterns_6(_bencher: &mut test::Bencher) {
    //    //    split_child_patterns_6_impl()
    //    //}
    //    //#[bench]
    //    //fn bench_split_child_patterns_3(b: &mut test::Bencher) {
    //    //    b.iter(|| split_child_patterns_3_impl())
    //    //}
}
