use crate::{
    hypergraph::{
        VertexIndex,
        Pattern,
        Hypergraph,
        split::Split,
        pattern_width,
    },
    token::Tokenize,
};
use indexmap::{
    IndexMap,
};
use std::{
    num::NonZeroUsize,
    collections::{
        BTreeSet,
    },
    cmp::PartialEq,
    borrow::Borrow,
};

use super::split_minimizer::SplitMinimizer;

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
/// refers to an index in a hypergraph node
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct IndexInParent {
    pub pattern_index: usize, // index of pattern in parent
    pub replaced_index: usize, // replaced index in pattern
}
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct SplitContext {
    pub context: (Pattern, Pattern),
    pub key: SplitKey,
}
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct SplitKey {
    pub index: VertexIndex, // index in hypergraph
    pub offset: NonZeroUsize,
}
impl SplitKey {
    pub fn new(index: VertexIndex, offset: NonZeroUsize) -> Self {
        Self {
            index,
            offset,
        }
    }
}
#[derive(Debug, Eq, Clone, Ord, PartialOrd)]
pub struct SplitHalf {
    context: Pattern,
    inner: Vec<SplitHalf>,
}
impl SplitHalf {
    pub fn new<T: Into<SplitHalf>, I: IntoIterator<Item=T>>(context: Pattern, inner: I) -> Self {
        Self {
            context,
            inner: inner.into_iter().map(Into::into).collect(),
        }
    }
}
impl<T: Into<Pattern>> From<T> for SplitHalf {
    fn from(pattern: T) -> Self {
        Self {
            context: pattern.into(),
            inner: Default::default(),
        }
    }
}
impl std::hash::Hash for SplitHalf {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        self.context.hash(h);
        let set: BTreeSet<_> = self.inner.iter().collect();
        set.hash(h);
    }
}
impl PartialEq for SplitHalf {
    fn eq(&self, other: &Self) -> bool {
        self.context == other.context && {
            let a: BTreeSet<_> = self.inner.iter().collect();
            let b: BTreeSet<_> = other.inner.iter().collect();
            a == b
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub struct PatternSplit {
    left: SplitHalf,
    right: SplitHalf,
}
impl PatternSplit {
    pub fn new(left: SplitHalf, right: SplitHalf) -> Self {
        Self {
            left,
            right,
        }
    }
}
impl<T: Into<SplitHalf>> From<(T, T)> for PatternSplit {
    fn from((left, right): (T, T)) -> Self {
        Self {
            left: left.into(),
            right: right.into(),
        }
    }
}
#[derive(Debug, Clone, Eq)]
pub struct IndexSplit {
    pattern_splits: Vec<PatternSplit>
}
impl IndexSplit {
    fn add_split<T: Into<PatternSplit>>(&mut self, split: T) {
        self.pattern_splits.push(split.into());
    }
    fn into_split_halves(self) -> (Vec<SplitHalf>, Vec<SplitHalf>) {
        self.pattern_splits
            .into_iter()
            .map(|pattern_split| (pattern_split.left, pattern_split.right))
            .unzip()
    }
}
impl PartialEq for IndexSplit {
    fn eq(&self, other: &Self) -> bool {
        let a: BTreeSet<_> = self.pattern_splits.iter().collect();
        let b: BTreeSet<_> = other.pattern_splits.iter().collect();
        a == b
    }
}
impl From<Split> for IndexSplit {
    fn from(split: Split) -> Self {
        Self {
            pattern_splits: vec![PatternSplit::from(split)],
        }
    }
}
impl<T: Into<PatternSplit>> From<Vec<T>> for IndexSplit {
    fn from(splits: Vec<T>) -> Self {
        Self {
            pattern_splits: splits.into_iter().map(Into::into).collect(),
        }
    }
}
impl<T: Into<SplitHalf>> From<(Vec<T>, Vec<T>)> for IndexSplit {
    fn from((left, right): (Vec<T>, Vec<T>)) -> Self {
        Self {
            pattern_splits: left.into_iter().zip(right).map(Into::into).collect(),
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct IndexSplitter {
    pub cache: IndexMap<SplitKey, IndexSplit>,
}
impl IndexSplitter {
    pub fn split<T: Tokenize + std::fmt::Display>(hypergraph: &mut Hypergraph<T>, root: impl Borrow<VertexIndex>, pos: NonZeroUsize) -> IndexSplit  {
        let index_split = Self::build_index_split(hypergraph, root, pos);
        SplitMinimizer::minimize_index_split(hypergraph, index_split)
    }
    pub fn build_index_split<T: Tokenize + std::fmt::Display>(hypergraph: &Hypergraph<T>, root: impl Borrow<VertexIndex>, pos: NonZeroUsize) -> IndexSplit  {
        let mut s = Self::default();
        hypergraph.try_perfect_split(root, pos)
            .unwrap_or_else(|child_splits| s.perform_child_splits(hypergraph, child_splits))
    }
    fn perform_child_splits<T: Tokenize + std::fmt::Display>(&mut self, hypergraph: &Hypergraph<T>, child_splits: Vec<SplitContext>) -> IndexSplit {
        let splits: Vec<PatternSplit> = child_splits.into_iter().map(|SplitContext {
            context: (left_context, right_context),
            key,
        }| {
            // recurse
            let index_split = self.get_index_split(hypergraph, key);
            let (left, right) = index_split.into_split_halves();
            PatternSplit::new(
                SplitHalf::new(left_context, left),
                SplitHalf::new(right_context, right),
            )
        }).collect();
        IndexSplit::from(splits)
    }
    fn create_new_index_split<T: Tokenize + std::fmt::Display>(&mut self, hypergraph: &Hypergraph<T>, key: SplitKey) -> IndexSplit {
        // todo: insert remaining patterns if perfect split has more than one index on one side
        let (perfect_split, remaining_splits) = hypergraph.separate_perfect_split(key.index, key.offset);

        let index_split = if let Some((pl, pr)) = perfect_split {
            if (pl.len() <= 1 || pattern_width(&pl) <= 2) && (pr.len() <= 1 || pattern_width(&pr) <= 2) {
                IndexSplit::from((pl, pr))
            } else { 
                let mut index_split = self.perform_child_splits(hypergraph, remaining_splits);
                index_split.add_split((pl, pr));
                index_split
            }
        } else {
            self.perform_child_splits(hypergraph, remaining_splits)
        };
        //let (left, right) = Self::minimize_split_patterns(hypergraph, (left, right));
        //println!("Split: {} =>", name);
        //println!("left:\n\t{}", hypergraph.separated_pattern_string(&left));
        //println!("right:\n\t{}", hypergraph.separated_pattern_string(&right));
        self.cache_index_split(key, index_split.clone());
        index_split
    }
    fn get_index_split<T: Tokenize + std::fmt::Display>(&mut self, hypergraph: &Hypergraph<T>, key: SplitKey) -> IndexSplit {
        //let name = hypergraph.index_string(key.index);
        // don't merge existing indices again
        self.cache.get(&key).cloned()
            //.map(|r| {
            //    println!("got cached split for: {}", name);
            //    r
            //})
            .unwrap_or_else(|| self.create_new_index_split(hypergraph, key))
    }
    fn cache_index_split(&mut self, key: SplitKey, split: IndexSplit) {
        self.cache.insert(key, split);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::hypergraph::{
        tests::context_mut,
    };
    use pretty_assertions::assert_eq;
    #[test]
    fn split_index_1() {
        let (
            graph,
            _a,
            _b,
            c,
            _d,
            _e,
            _f,
            _g,
            _h,
            _i,
            ab,
            _bc,
            _cd,
            _bcd,
            abc,
            _abcd,
            _cdef,
            _efghi,
            _abab,
            _ababab,
            _ababababcdefghi,
            ) = &mut *context_mut();
        let index_split = IndexSplitter::build_index_split(graph, abc, NonZeroUsize::new(2).unwrap());
        assert_eq!(index_split, IndexSplit::from((
            vec![*ab],
            vec![*c]
        )));
    }
    #[test]
    fn split_child_patterns_2() {
        let (
            graph,
            _a,
            _b,
            _c,
            d,
            _e,
            _f,
            _g,
            _h,
            _i,
            _ab,
            _bc,
            _cd,
            _bcd,
            abc,
            abcd,
            _cdef,
            _efghi,
            _abab,
            _ababab,
            _ababababcdefghi,
            ) = &mut *context_mut();
        let index_split = IndexSplitter::build_index_split(graph, abcd, NonZeroUsize::new(3).unwrap());
        assert_eq!(index_split, IndexSplit::from((
            vec![*abc],
            vec![*d]
        )));
    }
    use crate::token::*;
    fn split_child_patterns_3_impl() {
        let mut graph = Hypergraph::default();
        if let [a, b, c, d] = graph.insert_tokens(
            [
                Token::Element('a'),
                Token::Element('b'),
                Token::Element('c'),
                Token::Element('d'),
            ])[..] {
            // wxabyzabbyxabyz
            let ab = graph.insert_pattern([a, b]);
            let bc = graph.insert_pattern([b, c]);
            let cd = graph.insert_pattern([c, d]);
            let abc = graph.insert_patterns([
                vec![ab, c],
                vec![a, bc]
            ]);
            let bcd = graph.insert_patterns([
                vec![bc, d],
                vec![b, cd]
            ]);
            let abcd = graph.insert_patterns([
                vec![abc, d],
                vec![a, bcd]
            ]);
            let ab_pattern = vec![ab];
            let a_pattern = vec![a];
            let b_pattern = vec![b];
            let c_pattern = vec![c];
            let d_pattern = vec![d];
            let cd_pattern = vec![cd];

            //let (left, right) = graph.split_index_at_pos(abcd, NonZeroUsize::new(2).unwrap());
            let index_split = IndexSplitter::build_index_split(&graph, abcd, NonZeroUsize::new(2).unwrap());
            //assert_eq!(left, ab_pattern, "left");
            //assert_eq!(right, cd_pattern, "right");
            assert_eq!(index_split, IndexSplit::from(vec![
                PatternSplit::new(
                    SplitHalf::new(vec![], vec![ab_pattern]),
                    SplitHalf::new(d_pattern, vec![c_pattern]),
                ),
                PatternSplit::new(
                    SplitHalf::new(a_pattern, vec![b_pattern]),
                    SplitHalf::new(vec![], vec![cd_pattern]),
                ),
            ]));
        } else {
            panic!();
        }
    }
    #[test]
    fn split_child_patterns_3() {
        split_child_patterns_3_impl()
    }
    #[test]
    fn split_child_patterns_4() {
        let mut graph = Hypergraph::default();
        if let [a, b, _w, x, y, z] = graph.insert_tokens(
            [
                Token::Element('a'),
                Token::Element('b'),
                Token::Element('w'),
                Token::Element('x'),
                Token::Element('y'),
                Token::Element('z'),
            ])[..] {
            // wxabyzabbyxabyz
            let ab = graph.insert_pattern([a, b]);
            let by = graph.insert_pattern([b, y]);
            let yz = graph.insert_pattern([y, z]);
            let xab = graph.insert_pattern([x, ab]);
            let xaby = graph.insert_patterns([
                vec![xab, y],
                vec![x, a, by]
            ]);
            let xabyz = graph.insert_patterns([
                vec![xaby, z],
                vec![xab, yz]
            ]);
            //let xa_graph = ChildStrings::from_node(
            //    "xa",
            //    vec![
            //        vec!["x", "a"],
            //    ]
            //);
            //let byz_graph = ChildStrings::from_node(
            //    "byz",
            //    vec![
            //        vec!["by", "z"],
            //        vec!["b", "yz"],
            //    ]
            //);
            let z_pattern = vec![z];
            let x_pattern = vec![x];
            let a_pattern = vec![a];
            let b_pattern = vec![b];
            let x_a_pattern = vec![x, a];
            let by_pattern = vec![by];
            let yz_pattern = vec![yz];

            let index_split = IndexSplitter::build_index_split(&graph, xabyz, NonZeroUsize::new(2).unwrap());
            assert_eq!(index_split, IndexSplit::from(vec![
                PatternSplit::new(
                    SplitHalf::new(vec![], vec![
                        x_a_pattern
                    ]),
                    SplitHalf::new(z_pattern, vec![
                        by_pattern
                    ]),
                ),
                PatternSplit::new(
                    SplitHalf::new(vec![], vec![
                        SplitHalf::new(x_pattern, vec![
                            a_pattern
                        ]),
                    ]),
                    SplitHalf::new(yz_pattern, vec![
                        SplitHalf::new(vec![], vec![
                            b_pattern
                        ]),
                    ]),
                ),
            ]));
            //let left = graph.pattern_child_strings(left);
            //let right = graph.pattern_child_strings(right);
            //assert_eq!(left, xa_graph, "left");
            //assert_eq!(right, byz_graph, "right");
        } else {
            panic!();
        }
    }
    #[test]
    fn split_child_patterns_5() {
        let mut graph = Hypergraph::default();
        if let [a, b, w, x, y, z] = graph.insert_tokens(
            [
                Token::Element('a'),
                Token::Element('b'),
                Token::Element('w'),
                Token::Element('x'),
                Token::Element('y'),
                Token::Element('z'),
            ])[..] {
            // wxabyzabbyxabyz
            let ab = graph.insert_pattern([a, b]);
            let by = graph.insert_pattern([b, y]);
            let yz = graph.insert_pattern([y, z]);
            let xa = graph.insert_pattern([x, a]);
            let xab = graph.insert_patterns([
                vec![x, ab],
                vec![xa, b],
            ]);
            let xaby = graph.insert_patterns([
                vec![xab, y],
                vec![xa, by]
            ]);
            let xabyz = graph.insert_patterns([
                vec![xaby, z],
                vec![xab, yz]
            ]);
            let wxabyzabbyxabyz = graph.insert_pattern([w, xabyz, ab, by, xabyz]);

            let w_pattern = vec![w];
            let ab_by_xabyz_pattern = vec![ab, by, xabyz];
            let z_pattern = vec![z];
            let b_pattern = vec![b];
            let xa_pattern = vec![xa];
            let by_pattern = vec![by];
            let yz_pattern = vec![yz];
            let index_split = IndexSplitter::build_index_split(&graph, wxabyzabbyxabyz, NonZeroUsize::new(3).unwrap());
            assert_eq!(index_split, IndexSplit::from(vec![
                PatternSplit::new(
                    SplitHalf::new(w_pattern, vec![
                        SplitHalf::new(vec![], vec![
                            xa_pattern.clone(),
                        ]),
                        SplitHalf::new(vec![], vec![
                            xa_pattern,
                        ]),
                    ]),
                    SplitHalf::new(ab_by_xabyz_pattern, vec![
                        SplitHalf::new(z_pattern, vec![
                            by_pattern
                        ]),
                        SplitHalf::new(yz_pattern, vec![
                            b_pattern
                        ]),
                    ]),
                ),
            ]));
            //let left = graph.pattern_child_strings(left);
            //let right = graph.pattern_child_strings(right);
            //assert_eq!(left, wxa_graph, "left");
            //assert_eq!(right, byzabbyxabyz_graph, "right");
        } else {
            panic!();
        }
    }
    fn split_child_patterns_6_impl() {
        let mut graph = Hypergraph::default();
        if let [a, b, w, x, y, z] = graph.insert_tokens(
            [
                Token::Element('a'),
                Token::Element('b'),
                Token::Element('w'),
                Token::Element('x'),
                Token::Element('y'),
                Token::Element('z'),
            ])[..] {
            // wxabyzabbyxabyz
            let ab = graph.insert_pattern([a, b]);
            let by = graph.insert_pattern([b, y]);
            let yz = graph.insert_pattern([y, z]);
            let wx = graph.insert_pattern([w, x]);
            let xab = graph.insert_pattern([x, ab]);
            let xaby = graph.insert_patterns([
                vec![xab, y],
                vec![x, a, by]
            ]);
            let wxab = graph.insert_patterns([
                vec![wx, ab],
                vec![w, xab]
            ]);
            let wxaby = graph.insert_patterns([
                vec![w, xaby],
                vec![wx, a, by],
                vec![wxab, y]
            ]);
            let xabyz = graph.insert_patterns([
                vec![xaby, z],
                vec![xab, yz]
            ]);
            let wxabyz = graph.insert_patterns([
                vec![w, xabyz],
                vec![wxaby, z],
                vec![wx, ab, yz]
            ]);
            let w_pattern = vec![w];
            let x_pattern = vec![x];
            let y_pattern = vec![y];
            let a_pattern = vec![a];
            let wx_pattern = vec![wx];
            let wx_a_pattern = vec![wx, a];
            let z_pattern = vec![z];
            let b_pattern = vec![b];
            let x_a_pattern = vec![x, a];
            let by_pattern = vec![by];
            let yz_pattern = vec![yz];
            let index_split = IndexSplitter::build_index_split(&graph, wxabyz, NonZeroUsize::new(3).unwrap());
            assert_eq!(index_split, IndexSplit::from(vec![
                PatternSplit::new(
                    SplitHalf::new(vec![], vec![
                        SplitHalf::new(w_pattern.clone(), vec![
                            SplitHalf::from(x_a_pattern.clone()),
                        ]),
                        SplitHalf::new(vec![], vec![
                            SplitHalf::new(wx_pattern.clone(), vec![
                                a_pattern.clone(),
                            ]),
                            SplitHalf::new(w_pattern.clone(), vec![
                                SplitHalf::new(x_pattern.clone(), vec![
                                    a_pattern.clone(),
                                ]),
                            ]),
                        ]),
                        SplitHalf::from(wx_a_pattern),
                    ]),
                    SplitHalf::new(z_pattern.clone(), vec![
                        SplitHalf::new(vec![], vec![
                            SplitHalf::from(by_pattern.clone()),
                        ]),
                        SplitHalf::new(y_pattern, vec![
                            SplitHalf::new(vec![], vec![
                                SplitHalf::new(vec![], vec![
                                    b_pattern.clone(),
                                ]),
                            ]),
                            SplitHalf::new(vec![], vec![
                                b_pattern.clone(),
                            ]),
                        ]),
                        SplitHalf::from(by_pattern.clone()),
                    ]),
                ),
                PatternSplit::new(
                    SplitHalf::new(w_pattern, vec![
                        SplitHalf::new(vec![], vec![
                            SplitHalf::new(x_pattern, vec![
                                a_pattern.clone(),
                            ]),
                        ]),
                        SplitHalf::new(vec![], vec![
                            x_a_pattern,
                        ]),
                    ]),
                    SplitHalf::new(vec![], vec![
                        SplitHalf::new(yz_pattern.clone(), vec![
                            SplitHalf::new(vec![], vec![
                                b_pattern.clone(),
                            ]),
                        ]),
                        SplitHalf::new(z_pattern, vec![
                            by_pattern,
                        ]),
                    ]),
                ),
                PatternSplit::new(
                    SplitHalf::new(wx_pattern, vec![
                        a_pattern,
                    ]),
                    SplitHalf::new(yz_pattern, vec![
                        b_pattern,
                    ]),
                ),
            ]));
            //let left = graph.pattern_child_strings(left);
            //let right = graph.pattern_child_strings(right);
            //assert_eq!(left, wxa_graph, "left");
            //assert_eq!(right, byz_graph, "right");
        } else {
            panic!();
        }
    }
    #[test]
    fn split_child_patterns_6() {
        split_child_patterns_6_impl()
    }
    //#[bench]
    //fn bench_split_child_patterns_6(_bencher: &mut test::Bencher) {
    //    split_child_patterns_6_impl()
    //}
    //#[bench]
    //fn bench_split_child_patterns_3(b: &mut test::Bencher) {
    //    b.iter(|| split_child_patterns_3_impl())
    //}
}