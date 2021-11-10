use crate::{
    split::*,
    token::Tokenize,
    Hypergraph,
    Indexed,
};
use std::{
    borrow::Borrow,
    num::NonZeroUsize,
};
type ChildSplits = (Vec<(Pattern, SplitSegment)>, Vec<(Pattern, SplitSegment)>);
impl IndexSplitter {
    pub(crate) fn split_index_with_pid<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        root: impl Indexed,
        pos: NonZeroUsize,
    ) -> (Option<PatternId>, SingleSplitResult) {
        let root = root.index();
        //println!("splitting {} at {}", hypergraph.index_string(root), pos);
        let (perfect_split, remaining_splits) = hypergraph.separate_perfect_split(root, pos);
        Self::single_split_from_indices(hypergraph, root, perfect_split, remaining_splits)
    }
    pub(crate) fn split_index<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        root: impl Indexed,
        pos: NonZeroUsize,
    ) -> SingleSplitResult {
        Self::split_index_with_pid(hypergraph, root, pos).1
    }
    pub fn index_prefix<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        root: impl Indexed,
        pos: NonZeroUsize,
    ) -> (Child, SplitSegment) {
        let (pid, (l, r)) = Self::split_index_with_pid(hypergraph, root.index(), pos);
        match l {
            SplitSegment::Child(c) => (c, r),
            SplitSegment::Pattern(p) => {
                let len = p.len();
                let c = hypergraph.insert_pattern(p);
                if let Some(pid) = pid {
                    hypergraph.replace_in_pattern(root, pid, 0..len, c);
                }
                (c, r)
            }
        }
    }
    pub fn index_postfix<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        root: impl Indexed,
        pos: NonZeroUsize,
    ) -> (SplitSegment, Child) {
        let (pid, (l, r)) = Self::split_index_with_pid(hypergraph, root.index(), pos);
        match r {
            SplitSegment::Child(c) => (l, c),
            SplitSegment::Pattern(p) => {
                let c = hypergraph.insert_pattern(p);
                if let Some(pid) = pid {
                    hypergraph.replace_in_pattern(root, pid, l.len().., c);
                }
                (l, c)
            }
        }
    }
    /// Find single split indicies and positions of multiple patterns
    pub(crate) fn find_single_split_indices(
        patterns: impl IntoIterator<Item = (PatternId, impl IntoIterator<Item = Child>)>,
        pos: NonZeroUsize,
    ) -> SingleSplitIndices {
        patterns
            .into_iter()
            .map(move |(i, pattern)| {
                let split =
                    Self::find_pattern_split_index(pattern, pos).expect("Split not in pattern");
                (i, split)
            })
            .collect()
    }
    pub(crate) fn process_single_splits<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        vertex: &VertexData,
        root: impl Indexed + Clone,
        single: impl IntoIterator<Item = (PatternId, SplitIndex)>,
    ) -> RangeSplitResult {
        let (perfect_split, remaining_splits) = Self::separate_single_split_indices(vertex, single);
        let (_, (left, right)) =
            Self::single_split_from_indices(hypergraph, root, perfect_split, remaining_splits);
        RangeSplitResult::Single(left, right)
    }
    pub(crate) fn separate_single_split_indices(
        current_node: &VertexData,
        split_indices: impl IntoIterator<Item = (PatternId, SplitIndex)>,
    ) -> (Option<(Split, IndexInParent)>, Vec<SplitContext>) {
        let len = current_node.get_children().len();
        Self::perfect_split_search(current_node, split_indices)
            .into_iter()
            .fold((None, Vec::with_capacity(len)), |(pa, mut sa), r| match r {
                Ok(s) => {
                    sa.push(s);
                    (pa, sa)
                }
                Err(s) => (Some(s), sa),
            })
    }
    /// find split position in index in pattern
    pub(crate) fn find_pattern_split_index(
        pattern: impl IntoIterator<Item = impl Borrow<Child>>,
        pos: NonZeroUsize,
    ) -> Option<SplitIndex> {
        let mut skipped = 0;
        let pos: TokenPosition = pos.into();
        // find child overlapping with cut pos or
        pattern.into_iter().enumerate().find_map(|(i, child)| {
            let child = child.borrow();
            if skipped + child.get_width() <= pos {
                skipped += child.get_width();
                None
            } else {
                Some(SplitIndex {
                    index_pos: i,
                    pos: pos - skipped,
                    index: child.index,
                })
            }
        })
    }
    /// search for a perfect split in the split indices (offset = 0)
    pub(crate) fn perfect_split_search<'a>(
        current_node: &'a VertexData,
        split_indices: impl IntoIterator<Item = (PatternId, SplitIndex)> + 'a,
    ) -> impl IntoIterator<Item = Result<SplitContext, (Split, IndexInParent)>> + 'a {
        split_indices
            .into_iter()
            .map(move |(pattern_index, split_index)| {
                let pattern = current_node.get_child_pattern(&pattern_index).unwrap();
                Self::separate_pattern_split(pattern_index, split_index)
                    .map(
                        move |(
                            key,
                            IndexInParent {
                                replaced_index: split_index,
                                ..
                            },
                        )| {
                            let (prefix, postfix) = split_context(pattern, split_index);
                            SplitContext {
                                prefix,
                                key,
                                postfix,
                            }
                        },
                    )
                    .map_err(
                        |ind @ IndexInParent {
                             replaced_index: split_index,
                             ..
                         }| {
                            (split_pattern_at_index(pattern, split_index), ind)
                        },
                    )
            })
    }
    /// search for a perfect split in the split indices (offset = 0)
    pub(crate) fn separate_pattern_split(
        pattern_index: PatternId,
        split_index: SplitIndex,
    ) -> Result<(SplitKey, IndexInParent), IndexInParent> {
        let SplitIndex {
            index_pos,
            pos,
            index,
        } = split_index;
        let index_in_parent = IndexInParent {
            pattern_index,
            replaced_index: index_pos,
        };
        NonZeroUsize::new(pos)
            .map(|offset| (SplitKey::new(index, offset), index_in_parent.clone()))
            .ok_or(index_in_parent)
    }
    pub(crate) fn build_child_splits<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        child_splits: Vec<SplitContext>,
    ) -> ChildSplits {
        child_splits
            .into_iter()
            .map(
                |SplitContext {
                     prefix,
                     key,
                     postfix,
                 }| {
                    // recurse
                    let (l, r) = Self::split_index(hypergraph, key.index, key.offset);
                    ((prefix, l), (postfix, r))
                },
            )
            .unzip()
    }
    fn single_split_from_indices<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        root: impl Indexed,
        perfect_split: Option<(Split, IndexInParent)>,
        remaining_splits: Vec<SplitContext>,
    ) -> (Option<PatternId>, SingleSplitResult) {
        if let Some(ps) = perfect_split {
            let (pid, (left, right)) = Self::perform_perfect_split(hypergraph, ps, root);
            (Some(pid), (left, right))
        } else {
            // split all children and resolve
            let (pid, (left, right)) = Self::perform_child_splits(hypergraph, remaining_splits);
            hypergraph.add_pattern_to_node(root, left.clone().into_iter().chain(right.clone()));
            (pid, (left, right))
        }
    }
    fn perform_child_splits<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        child_splits: Vec<SplitContext>,
    ) -> (Option<PatternId>, SingleSplitResult) {
        // for every child split
        let (left, right) = Self::build_child_splits(hypergraph, child_splits);
        let mut minimizer = SplitMinimizer::new(hypergraph);
        let left = minimizer.merge_left_splits(left);
        let right = minimizer.merge_right_splits(right);
        (None, (left, right))
    }
    fn perform_perfect_split<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        ((pl, pr), ind): (Split, IndexInParent),
        parent: impl Indexed,
    ) -> (PatternId, SingleSplitResult) {
        // if other patterns can't add any more overlapping splits
        let parent = parent.index();
        (
            ind.pattern_index,
            (
                Self::resolve_perfect_split_range(
                    hypergraph,
                    pl,
                    parent,
                    ind.pattern_index,
                    0..ind.replaced_index,
                ),
                Self::resolve_perfect_split_range(
                    hypergraph,
                    pr,
                    parent,
                    ind.pattern_index,
                    ind.replaced_index..,
                ),
            ),
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        r#match::*,
        search::*,
        tests::*,
        token::*,
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
            _ef,
            _cdef,
            _efghi,
            _abab,
            _ababab,
            _ababababcdefghi,
        ) = &mut *context_mut();
        let (left, right) = graph.split_index(*abc, NonZeroUsize::new(2).unwrap());
        assert_eq!(left, SplitSegment::Child(Child::new(ab, 2)), "left");
        assert_eq!(right, SplitSegment::Child(Child::new(c, 1)), "right");
    }
    #[test]
    fn split_index_2() {
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
            _ef,
            _cdef,
            _efghi,
            _abab,
            _ababab,
            _ababababcdefghi,
        ) = &mut *context_mut();
        let (left, right) = graph.split_index(*abcd, NonZeroUsize::new(3).unwrap());
        assert_eq!(left, SplitSegment::Child(Child::new(abc, 3)), "left");
        assert_eq!(right, SplitSegment::Child(Child::new(d, 1)), "right");
    }
    fn split_index_3_impl() {
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

            let (left, right) = graph.split_index(abcd, NonZeroUsize::new(2).unwrap());
            assert_eq!(left, SplitSegment::Child(ab), "left");
            assert_eq!(right, SplitSegment::Child(cd), "right");
        } else {
            panic!();
        }
    }
    #[test]
    fn split_index_3() {
        split_index_3_impl()
    }
    #[test]
    fn split_index_4() {
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

            let (left, right) = graph.split_index(xabyz, NonZeroUsize::new(2).unwrap());
            println!("{:#?}", graph);
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
            .expect("xa");

            let byz_found = graph.find_pattern(vec![b, y, z]);
            let byz = if let SearchFound {
                index: byz,
                parent_match:
                    ParentMatch {
                        parent_range: FoundRange::Complete,
                        ..
                    },
                ..
            } = byz_found.expect("byz not found")
            {
                Some(byz)
            } else {
                None
            }
            .expect("byz");
            assert_eq!(left, SplitSegment::Child(xa), "left");
            assert_eq!(right, SplitSegment::Child(byz), "left");
        } else {
            panic!();
        }
    }
    #[test]
    fn split_index_5() {
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
            let xa = graph.insert_pattern([x, a]);
            let xab = graph.insert_patterns([vec![x, ab], vec![xa, b]]);
            let xaby = graph.insert_patterns([vec![xab, y], vec![xa, by]]);
            let xabyz = graph.insert_patterns([vec![xaby, z], vec![xab, yz]]);
            let wxabyzabbyxabyz = graph.insert_pattern([w, xabyz, ab, by, xabyz]);

            // split wxabyzabbyxabyz at 3
            let (left, right) = graph.split_index(wxabyzabbyxabyz, NonZeroUsize::new(3).unwrap());

            let xa_found = graph.find_pattern(vec![w, xa]);
            let wxa = if let SearchFound {
                index: wxa,
                parent_match:
                    ParentMatch {
                        parent_range: FoundRange::Complete,
                        ..
                    },
                ..
            } = xa_found.expect("wxa not found")
            {
                Some(wxa)
            } else {
                None
            }
            .unwrap();
            assert_eq!(left, SplitSegment::Child(wxa), "left");

            let byzabbyxabyz_found = graph.find_pattern(vec![by, z, ab, by, xabyz]);
            let byzabbyxabyz = if let SearchFound {
                index: byzabbyxabyz,
                parent_match:
                    ParentMatch {
                        parent_range: FoundRange::Complete,
                        ..
                    },
                ..
            } = byzabbyxabyz_found.expect("byzabbyxabyz not found")
            {
                Some(byzabbyxabyz)
            } else {
                None
            }
            .expect("byzabbyxabyz");

            assert_eq!(right, SplitSegment::Child(byzabbyxabyz), "left");
        } else {
            panic!();
        }
    }
    fn split_index_6_impl() {
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

            let (left, right) = graph.split_index(wxabyz, NonZeroUsize::new(3).unwrap());
            let wxa_found = graph.find_pattern(vec![w, x, a]);
            let wxa = if let SearchFound {
                index: wxa,
                parent_match:
                    ParentMatch {
                        parent_range: FoundRange::Complete,
                        ..
                    },
                ..
            } = wxa_found.expect("wxa not found")
            {
                Some(wxa)
            } else {
                None
            }
            .unwrap();
            let byz_found = graph.find_pattern(vec![b, y, z]);
            println!("{:#?}", byz_found);
            let byz = if let SearchFound {
                index: byz,
                parent_match:
                    ParentMatch {
                        parent_range: FoundRange::Complete,
                        ..
                    },
                ..
            } = byz_found.expect("byz not found")
            {
                println!("byz = {}", graph.index_string(byz));
                Some(byz)
            } else {
                None
            }
            .unwrap();

            assert_eq!(left, SplitSegment::Child(wxa), "left");
            assert_eq!(right, SplitSegment::Child(byz), "right");
        } else {
            panic!();
        }
    }
    #[test]
    fn split_index_6() {
        split_index_6_impl()
    }
    //#[bench]
    //fn bench_split_child_patterns_6(b: &mut test::Bencher) {
    //    b.iter(split_child_patterns_6_impl)
    //}
    //#[bench]
    //fn bench_split_child_patterns_3(b: &mut test::Bencher) {
    //    b.iter(split_child_patterns_3_impl)
    //}
}
