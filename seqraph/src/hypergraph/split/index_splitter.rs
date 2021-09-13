use crate::{
    hypergraph::{
        VertexIndex,
        Hypergraph,
        split::*,
        Indexed,
    },
    token::Tokenize,
};
use std::{
    num::NonZeroUsize,
    cmp::PartialEq,
};

#[derive(Debug, PartialEq, Eq, Clone, Hash, Copy)]
pub struct SplitKey {
    pub index: VertexIndex, // index in hypergraph
    pub offset: NonZeroUsize,
}
impl SplitKey {
    pub fn new(index: impl Indexed, offset: NonZeroUsize) -> Self {
        Self {
            index: *index.borrow(),
            offset,
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct IndexSplitter;

impl IndexSplitter {
    pub(crate) fn split_index_complete<T: Tokenize>(hypergraph: &mut Hypergraph<T>, root: impl Indexed, pos: NonZeroUsize) -> (Child, Child) {
        let root = root.index();
        let (perfect_split, remaining_splits) = hypergraph.separate_perfect_split(root, pos);
        if let Some(ps) = perfect_split {
            Self::perform_perfect_split(hypergraph, ps, root)
        } else {
            // split all children and resolve
            let (left, right) = Self::perform_child_splits(hypergraph, remaining_splits);
            hypergraph.add_pattern_to_node(root, [left, right]);
            (left, right)
        }
    }
    fn perform_perfect_split<T: Tokenize>(hypergraph: &mut Hypergraph<T>, ((pl, pr), ind): (Split, IndexInParent), parent: impl Indexed) -> (Child, Child) {
        // if other patterns can't add any more overlapping splits
        let parent = parent.index();
        (
            Self::resolve_perfect_split_half(hypergraph, pl, parent, ind.pattern_index, 0..ind.replaced_index),
            Self::resolve_perfect_split_half(hypergraph, pr, parent, ind.pattern_index, ind.replaced_index..)
        )
    }
    fn resolve_perfect_split_half<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        half: Pattern,
        parent: impl Indexed,
        pattern_index: PatternId,
        range: impl PatternRangeIndex + Clone,
        ) -> Child {
        if half.len() <= 1 {
            *half.first().expect("Empty perfect split half!")
        } else {
            let c = hypergraph.insert_pattern(half);
            hypergraph.replace_in_pattern(parent, pattern_index, range, [c]);
            c
        }
    }
    fn perform_child_splits<T: Tokenize>(hypergraph: &mut Hypergraph<T>, child_splits: Vec<SplitContext>) -> (Child, Child) {
        // for every child split
        let (left, right) = child_splits.into_iter().map(|SplitContext {
            prefix,
            key,
            postfix,
        }| {
            // recurse
            let (l, r) = Self::split_index_complete(hypergraph, key.index, key.offset);
            ((prefix, l), (postfix, r))
        }).unzip();
        let left = SplitMinimizer::merge_left_splits(hypergraph, left);
        let right = SplitMinimizer::merge_right_splits(hypergraph, right);
        (left, right)
    }
}
#[cfg(test)]
mod tests {
    //use super::*;
    //use crate::hypergraph::{
    //    tests::context_mut,
    //};
    //use pretty_assertions::assert_eq;
    //#[test]
    //fn split_index_1() {
    //    let (
    //        graph,
    //        _a,
    //        _b,
    //        c,
    //        _d,
    //        _e,
    //        _f,
    //        _g,
    //        _h,
    //        _i,
    //        ab,
    //        _bc,
    //        _cd,
    //        _bcd,
    //        abc,
    //        _abcd,
    //        _cdef,
    //        _efghi,
    //        _abab,
    //        _ababab,
    //        _ababababcdefghi,
    //        ) = &mut *context_mut();
    //    let index_split = IndexSplitter::build_index_split(graph, abc, NonZeroUsize::new(2).unwrap());
    //    assert_eq!(index_split, IndexSplit::from((
    //        vec![*ab],
    //        vec![*c]
    //    )));
    //}
    //#[test]
    //fn split_child_patterns_2() {
    //    let (
    //        graph,
    //        _a,
    //        _b,
    //        _c,
    //        d,
    //        _e,
    //        _f,
    //        _g,
    //        _h,
    //        _i,
    //        _ab,
    //        _bc,
    //        _cd,
    //        _bcd,
    //        abc,
    //        abcd,
    //        _cdef,
    //        _efghi,
    //        _abab,
    //        _ababab,
    //        _ababababcdefghi,
    //        ) = &mut *context_mut();
    //    let index_split = IndexSplitter::build_index_split(graph, abcd, NonZeroUsize::new(3).unwrap());
    //    assert_eq!(index_split, IndexSplit::from((
    //        vec![*abc],
    //        vec![*d]
    //    )));
    //}
    //use crate::token::*;
    //fn split_child_patterns_3_impl() {
    //    let mut graph = Hypergraph::default();
    //    if let [a, b, c, d] = graph.insert_tokens(
    //        [
    //            Token::Element('a'),
    //            Token::Element('b'),
    //            Token::Element('c'),
    //            Token::Element('d'),
    //        ])[..] {
    //        // wxabyzabbyxabyz
    //        let ab = graph.insert_pattern([a, b]);
    //        let bc = graph.insert_pattern([b, c]);
    //        let cd = graph.insert_pattern([c, d]);
    //        let abc = graph.insert_patterns([
    //            vec![ab, c],
    //            vec![a, bc]
    //        ]);
    //        let bcd = graph.insert_patterns([
    //            vec![bc, d],
    //            vec![b, cd]
    //        ]);
    //        let abcd = graph.insert_patterns([
    //            vec![abc, d],
    //            vec![a, bcd]
    //        ]);
    //        let ab_pattern = vec![ab];
    //        let a_pattern = vec![a];
    //        let b_pattern = vec![b];
    //        let c_pattern = vec![c];
    //        let d_pattern = vec![d];
    //        let cd_pattern = vec![cd];

    //        let index_split = IndexSplitter::build_index_split(&graph, abcd, NonZeroUsize::new(2).unwrap());
    //        assert_eq!(index_split, IndexSplit::from(vec![
    //            PatternSplit::new(
    //                vec![],
    //                vec![(ab_pattern, c_pattern)],
    //                d_pattern,
    //            ),
    //            PatternSplit::new(
    //                a_pattern,
    //                vec![(b_pattern, cd_pattern)],
    //                vec![],
    //            ),
    //        ]));
    //    } else {
    //        panic!();
    //    }
    //}
    //#[test]
    //fn split_child_patterns_3() {
    //    split_child_patterns_3_impl()
    //}
    //#[test]
    //fn split_child_patterns_4() {
    //    let mut graph = Hypergraph::default();
    //    if let [a, b, _w, x, y, z] = graph.insert_tokens(
    //        [
    //            Token::Element('a'),
    //            Token::Element('b'),
    //            Token::Element('w'),
    //            Token::Element('x'),
    //            Token::Element('y'),
    //            Token::Element('z'),
    //        ])[..] {
    //        // wxabyzabbyxabyz
    //        let ab = graph.insert_pattern([a, b]);
    //        let by = graph.insert_pattern([b, y]);
    //        let yz = graph.insert_pattern([y, z]);
    //        let xab = graph.insert_pattern([x, ab]);
    //        let xaby = graph.insert_patterns([
    //            vec![xab, y],
    //            vec![x, a, by]
    //        ]);
    //        let xabyz = graph.insert_patterns([
    //            vec![xaby, z],
    //            vec![xab, yz]
    //        ]);
    //        let z_pattern = vec![z];
    //        let x_pattern = vec![x];
    //        let a_pattern = vec![a];
    //        let b_pattern = vec![b];
    //        let x_a_pattern = vec![x, a];
    //        let by_pattern = vec![by];
    //        let yz_pattern = vec![yz];

    //        let index_split = IndexSplitter::build_index_split(&graph, xabyz, NonZeroUsize::new(2).unwrap());
    //        assert_eq!(index_split, IndexSplit::from(vec![
    //            PatternSplit::new(
    //                vec![],
    //                vec![(x_a_pattern, by_pattern)],
    //                z_pattern,
    //            ),
    //            PatternSplit::new(
    //                vec![],
    //                vec![
    //                    PatternSplit::new(
    //                        x_pattern,
    //                        vec![(a_pattern, b_pattern)],
    //                        vec![],
    //                    ),
    //                ],
    //                yz_pattern,
    //            ),
    //        ]));
    //    } else {
    //        panic!();
    //    }
    //}
    //#[test]
    //fn split_child_patterns_5() {
    //    let mut graph = Hypergraph::default();
    //    if let [a, b, w, x, y, z] = graph.insert_tokens(
    //        [
    //            Token::Element('a'),
    //            Token::Element('b'),
    //            Token::Element('w'),
    //            Token::Element('x'),
    //            Token::Element('y'),
    //            Token::Element('z'),
    //        ])[..] {
    //        // wxabyzabbyxabyz
    //        let ab = graph.insert_pattern([a, b]);
    //        let by = graph.insert_pattern([b, y]);
    //        let yz = graph.insert_pattern([y, z]);
    //        let xa = graph.insert_pattern([x, a]);
    //        let xab = graph.insert_patterns([
    //            vec![x, ab],
    //            vec![xa, b],
    //        ]);
    //        let xaby = graph.insert_patterns([
    //            vec![xab, y],
    //            vec![xa, by]
    //        ]);
    //        let xabyz = graph.insert_patterns([
    //            vec![xaby, z],
    //            vec![xab, yz]
    //        ]);
    //        let wxabyzabbyxabyz = graph.insert_pattern([w, xabyz, ab, by, xabyz]);

    //        let w_pattern = vec![w];
    //        let ab_by_xabyz_pattern = vec![ab, by, xabyz];
    //        let z_pattern = vec![z];
    //        let b_pattern = vec![b];
    //        let xa_pattern = vec![xa];
    //        let by_pattern = vec![by];
    //        let yz_pattern = vec![yz];
    //        let index_split = IndexSplitter::build_index_split(&graph, wxabyzabbyxabyz, NonZeroUsize::new(3).unwrap());
    //        assert_eq!(index_split, IndexSplit::from((
    //            w_pattern,
    //            vec![
    //                PatternSplit::new(
    //                    vec![],
    //                    vec![
    //                        PatternSplit::from((
    //                            xa_pattern.clone(),
    //                            b_pattern,
    //                        )),
    //                    ],
    //                    yz_pattern,
    //                ),
    //                PatternSplit::new(
    //                    vec![],
    //                    vec![
    //                        PatternSplit::from((
    //                            xa_pattern,
    //                            by_pattern,
    //                        )),
    //                    ],
    //                    z_pattern,
    //                ),
    //            ],
    //            ab_by_xabyz_pattern,
    //        )));
    //    } else {
    //        panic!();
    //    }
    //}
    //fn split_child_patterns_6_impl() {
    //    let mut graph = Hypergraph::default();
    //    if let [a, b, w, x, y, z] = graph.insert_tokens(
    //        [
    //            Token::Element('a'),
    //            Token::Element('b'),
    //            Token::Element('w'),
    //            Token::Element('x'),
    //            Token::Element('y'),
    //            Token::Element('z'),
    //        ])[..] {
    //        // wxabyzabbyxabyz
    //        let ab = graph.insert_pattern([a, b]);
    //        let by = graph.insert_pattern([b, y]);
    //        let yz = graph.insert_pattern([y, z]);
    //        let wx = graph.insert_pattern([w, x]);
    //        let xab = graph.insert_pattern([x, ab]);
    //        let xaby = graph.insert_patterns([
    //            vec![xab, y],
    //            vec![x, a, by]
    //        ]);
    //        let wxab = graph.insert_patterns([
    //            vec![wx, ab],
    //            vec![w, xab]
    //        ]);
    //        let wxaby = graph.insert_patterns([
    //            vec![w, xaby],
    //            vec![wx, a, by],
    //            vec![wxab, y]
    //        ]);
    //        let xabyz = graph.insert_patterns([
    //            vec![xaby, z],
    //            vec![xab, yz]
    //        ]);
    //        let wxabyz = graph.insert_patterns([
    //            vec![w, xabyz],
    //            vec![wxaby, z],
    //            vec![wx, ab, yz]
    //        ]);
    //        let w_pattern = vec![w];
    //        let x_pattern = vec![x];
    //        let y_pattern = vec![y];
    //        let a_pattern = vec![a];
    //        let wx_pattern = vec![wx];
    //        let wx_a_pattern = vec![wx, a];
    //        let z_pattern = vec![z];
    //        let b_pattern = vec![b];
    //        let x_a_pattern = vec![x, a];
    //        let by_pattern = vec![by];
    //        let yz_pattern = vec![yz];
    //        let wxabyz_split = IndexSplitter::build_index_split(&graph, wxabyz, NonZeroUsize::new(3).unwrap());
    //        assert_eq!(wxabyz_split, IndexSplit::from(vec![
    //            PatternSplit::new(
    //                wx_pattern.clone(),
    //                (a_pattern.clone(), b_pattern.clone()),
    //                yz_pattern.clone(),
    //            ),
    //            PatternSplit::new(
    //                vec![],
    //                vec![
    //                    PatternSplit::new(
    //                        w_pattern.clone(),
    //                        (x_a_pattern.clone(), by_pattern.clone()),
    //                        vec![],
    //                    ),
    //                    PatternSplit::from((
    //                        wx_a_pattern,
    //                        by_pattern.clone(),
    //                    )),
    //                    PatternSplit::new(
    //                        vec![],
    //                        vec![
    //                            PatternSplit::from((
    //                                wx_pattern,
    //                                PatternSplit::from((
    //                                    a_pattern.clone(),
    //                                    b_pattern.clone(),
    //                                )),
    //                                vec![],
    //                            )),
    //                            PatternSplit::from((
    //                                w_pattern.clone(),
    //                                PatternSplit::from((
    //                                    x_pattern.clone(),
    //                                    (a_pattern.clone(), b_pattern.clone()),
    //                                    vec![],
    //                                )),
    //                                vec![],
    //                            )),
    //                        ],
    //                        y_pattern,
    //                    ),
    //                ],
    //                z_pattern.clone(),
    //            ),
    //            PatternSplit::new(
    //                w_pattern,
    //                vec![
    //                    PatternSplit::new(
    //                        vec![],
    //                        PatternSplit::new(
    //                            x_pattern,
    //                            (a_pattern, b_pattern),
    //                            vec![]
    //                        ),
    //                        yz_pattern,
    //                    ),
    //                    PatternSplit::new(
    //                        vec![],
    //                        (x_a_pattern, by_pattern),
    //                        z_pattern,
    //                    ),
    //                ],
    //                vec![]
    //            ),
    //        ]), "wxabyz");
    //    } else {
    //        panic!();
    //    }
    //}
    //#[test]
    //fn split_child_patterns_6() {
    //    split_child_patterns_6_impl()
    //}
    //#[test]
    //fn split_child_patterns_7() {
    //    let mut graph = Hypergraph::default();
    //    if let [a, b, w, x, y, z] = graph.insert_tokens(
    //        [
    //            Token::Element('a'),
    //            Token::Element('b'),
    //            Token::Element('w'),
    //            Token::Element('x'),
    //            Token::Element('y'),
    //            Token::Element('z'),
    //        ])[..] {
    //        // wxabyzabbyxabyz
    //        let ab = graph.insert_pattern([a, b]);
    //        let by = graph.insert_pattern([b, y]);
    //        let yz = graph.insert_pattern([y, z]);
    //        let wx = graph.insert_pattern([w, x]);
    //        let xab = graph.insert_pattern([x, ab]);
    //        let xaby = graph.insert_patterns([
    //            vec![xab, y],
    //            vec![x, a, by]
    //        ]);
    //        let wxab = graph.insert_patterns([
    //            vec![wx, ab],
    //            vec![w, xab]
    //        ]);
    //        let wxaby = graph.insert_patterns([
    //            vec![w, xaby],
    //            vec![wx, a, by],
    //            vec![wxab, y]
    //        ]);
    //        let xabyz = graph.insert_patterns([
    //            vec![xaby, z],
    //            vec![xab, yz]
    //        ]);
    //        let wxabyz = graph.insert_patterns([
    //            vec![w, xabyz],
    //            vec![wxaby, z],
    //            vec![wx, ab, yz]
    //        ]);
    //        let w_pattern = vec![w];
    //        let x_pattern = vec![x];
    //        let y_pattern = vec![y];
    //        let a_pattern = vec![a];
    //        let wx_pattern = vec![wx];
    //        let wx_a_pattern = vec![wx, a];
    //        let z_pattern = vec![z];
    //        let b_pattern = vec![b];
    //        let x_a_pattern = vec![x, a];
    //        let by_pattern = vec![by];
    //        let yz_pattern = vec![yz];
    //        let wxabyz_split = IndexSplitter::build_index_split(&graph, wxabyz, NonZeroUsize::new(3).unwrap());
    //        let wxaby_split = IndexSplitter::build_index_split_complete(&graph, wxaby, NonZeroUsize::new(3).unwrap());
    //        let xabyz_split = IndexSplitter::build_index_split_complete(&graph, xabyz, NonZeroUsize::new(2).unwrap());
    //        let wxab_split = IndexSplitter::build_index_split_complete(&graph, wxab, NonZeroUsize::new(3).unwrap());
    //        let xaby_split = IndexSplitter::build_index_split_complete(&graph, xaby, NonZeroUsize::new(2).unwrap());
    //        let xab_split = IndexSplitter::build_index_split_complete(&graph, xab, NonZeroUsize::new(2).unwrap());
    //        assert_eq!(xab_split, IndexSplit::from((
    //            x_pattern.clone(),
    //            (
    //                a_pattern.clone(),
    //                b_pattern.clone(),
    //            ),
    //            vec![]
    //        )), "xab");
    //        //graph.print_index_split(&wxabyz_split);
    //        assert_eq!(xaby_split, IndexSplit::from((
    //            x_a_pattern.clone(),
    //            by_pattern.clone(),
    //        )), "xaby");
    //        assert_eq!(wxab_split, IndexSplit::from(vec![
    //            PatternSplit::new(
    //                wx_pattern.clone(),
    //                (
    //                    a_pattern.clone(),
    //                    b_pattern.clone(),
    //                ),
    //                vec![],
    //            ),
    //            PatternSplit::new(
    //                w_pattern.clone(),
    //                PatternSplit::new(
    //                    x_pattern.clone(),
    //                    (
    //                        a_pattern.clone(),
    //                        b_pattern.clone(),
    //                    ),
    //                    vec![],
    //                ),
    //                vec![],
    //            ),
    //        ]), "wxab");
    //        assert_eq!(wxaby_split, IndexSplit::from(vec![
    //            PatternSplit::new(
    //                w_pattern.clone(),
    //                (
    //                    x_a_pattern.clone(),
    //                    by_pattern.clone(),
    //                ),
    //                vec![],
    //            ),
    //            PatternSplit::new(
    //                vec![],
    //                vec![
    //                    PatternSplit::new(
    //                        wx_pattern.clone(),
    //                        (
    //                            a_pattern.clone(),
    //                            b_pattern.clone(),
    //                        ),
    //                        vec![],
    //                    ),
    //                    PatternSplit::new(
    //                        w_pattern.clone(),
    //                        PatternSplit::new(
    //                            x_pattern.clone(),
    //                            (
    //                                a_pattern.clone(),
    //                                b_pattern.clone(),
    //                            ),
    //                            vec![],
    //                        ),
    //                        vec![],
    //                    ),
    //                ],
    //                y_pattern
    //            ),
    //            PatternSplit::from(
    //                (wx_a_pattern, by_pattern.clone())
    //            ),
    //        ]), "wxaby");
    //        assert_eq!(xabyz_split, IndexSplit::from(vec![
    //            PatternSplit::new(
    //                vec![],
    //                PatternSplit::new(
    //                    x_pattern,
    //                    (
    //                        a_pattern.clone(),
    //                        b_pattern.clone(),
    //                    ),
    //                    vec![],
    //                ),
    //                yz_pattern.clone(),
    //            ),
    //            PatternSplit::new(
    //                vec![],
    //                (
    //                    x_a_pattern,
    //                    by_pattern,
    //                ),
    //                z_pattern.clone(),
    //            ),
    //        ]), "xabyz");
    //        assert_eq!(wxabyz_split, IndexSplit::from(vec![
    //            PatternSplit::new(
    //                vec![],
    //                wxaby_split,
    //                z_pattern
    //            ),
    //            PatternSplit::new(
    //                w_pattern,
    //                xabyz_split,
    //                vec![],
    //            ),
    //            PatternSplit::new(
    //                wx_pattern,
    //                (a_pattern, b_pattern),
    //                yz_pattern
    //            ),
    //        ]), "wxabyz");
    //    } else {
    //        panic!();
    //    }
    //}
    //#[bench]
    //fn bench_split_child_patterns_6(_bencher: &mut test::Bencher) {
    //    split_child_patterns_6_impl()
    //}
    //#[bench]
    //fn bench_split_child_patterns_3(b: &mut test::Bencher) {
    //    b.iter(|| split_child_patterns_3_impl())
    //}
}