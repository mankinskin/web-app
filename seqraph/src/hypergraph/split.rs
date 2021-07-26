use crate::{
    hypergraph::{
        Hypergraph,
        VertexIndex,
        PatternId,
        Pattern,
        PatternView,
        TokenPosition,
        Child,
        ChildPatterns,
        path_tree::{
            IndexInParent,
            TreeParent,
            PathTree,
            IndexPositionDescriptor,
        },
        vertex::*,
    },
    token::{
        Tokenize,
    },
};
use std::num::NonZeroUsize;
use std::collections::{
    VecDeque,
};
use std::iter::FromIterator;
use either::Either;

#[derive(Debug, PartialEq, Eq, Clone)]
enum PatternSplit {
    Multiple(Pattern, ChildPatterns, ChildPatterns, Pattern),
    Single(Pattern, Pattern),
}
impl From<(Pattern, Pattern)> for PatternSplit {
    fn from((l, r): (Pattern, Pattern)) -> Self {
        PatternSplit::Single(l, r)
    }
}
struct Split {
    pattern: Pattern,
    parent: Option<TreeParent>,
    width: TokenPosition,
}

impl<'t, 'a, T> Hypergraph<T>
    where T: Tokenize + 't,
{
    pub fn split_pattern_at_index(
        pattern: PatternView<'a>,
        index: PatternId,
        ) -> (Pattern, Pattern) {
        let prefix = &pattern[..index];
        let postfix = &pattern[index..];
        //let prefix_str = self.sub_pattern_string(prefix);
        //let postfix_str = self.sub_pattern_string(postfix);
        (
            prefix.into_iter().cloned().collect(),
            postfix.into_iter().cloned().collect()
        )
    }
    pub fn find_pattern_split_index(
        pattern: impl Iterator<Item=&'a Child>,
        pos: NonZeroUsize,
        ) -> Option<(IndexPosition, TokenPosition)> {
        let mut skipped = 0;
        let pos: TokenPosition = pos.into();
        // find child overlapping with cut pos or 
        pattern.enumerate()
            .find_map(|(i, child)| {
                if skipped + child.get_width() <= pos {
                    skipped += child.get_width();
                    None
                } else {
                    Some((i, pos - skipped))
                }
            })
    }
    pub fn find_child_pattern_split_indices(
        patterns: impl Iterator<Item=(&'a PatternId, impl Iterator<Item=&'a Child> + 'a)> + 'a,
        pos: NonZeroUsize,
        ) -> impl Iterator<Item=(PatternId, (IndexPosition, TokenPosition))> + 'a {
        patterns.filter_map(move |(i, pattern)|
            Self::find_pattern_split_index(pattern, pos).map(|split| (*i, split))
        )
    }
    fn get_split_neighbors_and_next_parent(&self, tree_parent: &TreeParent, tree: &PathTree, half: Either<(), ()>) -> (&[Child], Option<TreeParent>) {
        let IndexInParent {
            pattern_index,
            replaced_index,
        } = tree_parent.index_in_parent;
        let (next_parent, index) = tree.get_parents().get(tree_parent.tree_node).unwrap();
        let parent_node = self.get_vertex_data(index).unwrap();
        let neighbors = match half {
            Either::Left(_) => parent_node.get_child_pattern_range(&pattern_index, ..replaced_index).unwrap(),
            Either::Right(_) => parent_node.get_child_pattern_range(&pattern_index, replaced_index+1..).unwrap(),
        };
        (neighbors, next_parent.clone())
    }
    fn build_split_halves_from_tree(&self, mut splits: Vec<Split>, tree: &PathTree, half: Either<(), ()>) -> Vec<Pattern> {
        if splits.is_empty() {
            return Vec::new();
        }
        // todo: reuse this later
        let largest = splits.iter().map(|s| s.width).max().unwrap();
        //let (neighbors, next_parent) = largest.parent.as_ref().map(|parent|
        //    self.get_split_neighbors_and_next_parent(parent, tree, half)
        //).unwrap_or((&[], None));
        //// if at end
        //if neighbors.is_empty() && next_parent.is_none() {
        //    return vec![largest.pattern.clone()];
        //}
        for split in splits.iter_mut() {
            while let Some(tree_parent) = &split.parent {
                let (neighbors, next_parent) = self.get_split_neighbors_and_next_parent(&tree_parent, tree, half);
                if !neighbors.is_empty() {
                    let width = Self::pattern_width(neighbors);
                    split.width += width;
                    if split.width == largest {
                        split.width = 0; // exclude at end
                        break;
                    }
                    split.pattern = match half {
                        Either::Left(_) => [neighbors, &split.pattern].concat(),
                        Either::Right(_) => [&split.pattern, neighbors].concat(),
                    };
                }
                split.parent = next_parent;
            }
        }
        splits.into_iter().filter(|s| s.width > 0).map(|s| s.pattern).collect()
    }
    fn build_splits_from_tree(&self, splits: Vec<(Split, Split)>, tree: PathTree) -> (Vec<Pattern>, Vec<Pattern>) {
        let (left, right): (Vec<_>, Vec<_>) = splits.into_iter().unzip();
        (
            self.build_split_halves_from_tree(left, &tree, Either::Left(())),
            self.build_split_halves_from_tree(right, &tree, Either::Right(()))
        )
    }
    pub fn split_index_at_pos(&self, root: VertexIndex, pos: NonZeroUsize) -> (Vec<Pattern>, Vec<Pattern>) {
        let mut queue = VecDeque::from_iter(std::iter::once(IndexPositionDescriptor {
                node: root,
                offset: pos,
                parent: None,
            }));
        let mut path_tree = PathTree::new();
        let mut splits = Vec::new();
        while let Some(IndexPositionDescriptor {
                node: current_index,
                offset,
                parent,
             }) = queue.pop_front() {
            let current_node = self.get_vertex_data(current_index).unwrap();
            let children = current_node.get_children().clone();
            let child_slices = children.iter().map(|(i, p)| (i, p.iter()));
            let split_indices = Hypergraph::<T>::find_child_pattern_split_indices(child_slices, offset);
            let perfect_split = split_indices
                .map(|(pattern_index, (split_index, offset))| {
                    let index_in_parent = IndexInParent {
                        pattern_index,
                        replaced_index: split_index,
                    };
                    NonZeroUsize::new(offset)
                        .ok_or(index_in_parent.clone())
                        .map(|offset| (index_in_parent, offset))
                })
                .collect::<Result<Vec<_>, _>>();
            match perfect_split {
                Err(IndexInParent {
                        pattern_index,
                        replaced_index: split_index,
                    }) => {
                    // perfect split found
                    let (left_split, right_split) = Hypergraph::<T>::split_pattern_at_index(&current_node.get_child_pattern(&pattern_index).unwrap(), split_index);
                    splits.push((Split {
                        width: Self::pattern_width(&left_split),
                        pattern: left_split,
                        parent: parent.clone(),
                    }, Split {
                        width: Self::pattern_width(&right_split),
                        pattern: right_split,
                        parent,
                    }));
                },
                Ok(split_indices) => {
                    // no perfect split
                    // add current node to path tree
                    let tree_node = path_tree.add_element(parent, current_index);
                    queue.extend(split_indices.into_iter()
                        .map(|(index_in_parent, offset)| {
                        let IndexInParent { pattern_index, replaced_index } = index_in_parent;
                        IndexPositionDescriptor {
                            node: current_node.get_child_pattern_position(&pattern_index, replaced_index).unwrap().get_index(),
                            offset,
                            parent: Some(TreeParent {
                                tree_node,
                                index_in_parent,
                            }),
                        }
                    }));
                }
            };
        }
        return self.build_splits_from_tree(splits, path_tree);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::hypergraph::tests::CONTEXT;
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
            ) = &*CONTEXT;
        let (left, right) = graph.split_index_at_pos(*abc, NonZeroUsize::new(2).unwrap());
        assert_eq!(left, vec![vec![Child::new(*ab, 2)]], "left");
        assert_eq!(right, vec![vec![Child::new(*c, 1)]], "right");
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
            ) = &*CONTEXT;
        let (left, right) = graph.split_index_at_pos(*abcd, NonZeroUsize::new(3).unwrap());
        assert_eq!(left, vec![vec![Child::new(*abc, 3)]], "left");
        assert_eq!(right, vec![vec![Child::new(*d, 1)]], "right");
    }
    use crate::token::*;
    #[test]
    fn split_child_patterns_3() {
        let mut graph = Hypergraph::new();
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
            let ab_pattern = vec![Child::new(ab, 2)];
            let cd_pattern = vec![Child::new(cd, 2)];

            let (left, right) = graph.split_index_at_pos(abcd, NonZeroUsize::new(2).unwrap());
            assert_eq!(left, vec![ab_pattern], "left");
            assert_eq!(right, vec![cd_pattern], "right");
        } else {
            panic!();
        }
    }
    #[test]
    fn split_child_patterns_4() {
        let mut graph = Hypergraph::new();
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
            let xab = graph.insert_pattern([x, ab]);
            let xaby = graph.insert_patterns([
                vec![xab, y],
                vec![x, a, by]
            ]);
            let xabyz = graph.insert_patterns([
                vec![xaby, z],
                vec![xab, yz]
            ]);
            // split xabyz at 2
            let xa_pattern = vec![
                vec![Child::new(x, 1), Child::new(a, 1)],
            ];
            let byz_pattern = vec![
                vec![Child::new(by, 2), Child::new(z, 1)],
                vec![Child::new(b, 1), Child::new(yz, 2)],
            ];

            let (left, right) = graph.split_index_at_pos(xabyz, NonZeroUsize::new(2).unwrap());
            assert_eq!(left, xa_pattern, "left");
            assert_eq!(right, byz_pattern, "right");
        } else {
            panic!();
        }
    }
    #[test]
    fn split_child_patterns_5() {
        let mut graph = Hypergraph::new();
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
            let xab = graph.insert_pattern([x, ab]);
            let xaby = graph.insert_patterns([
                vec![xab, y],
                vec![x, a, by]
            ]);
            let xabyz = graph.insert_patterns([
                vec![xaby, z],
                vec![xab, yz]
            ]);
            let wxabyzabbyxabyz = graph.insert_pattern([w, xabyz, ab, by, xabyz]);

            // split wxabyzabbyxabyz at 3
            let wxa_pattern = vec![
                vec![Child::new(w, 1), Child::new(x, 1), Child::new(a, 1)],
            ];
            let byzabbyxabyz_pattern = vec![
                vec![Child::new(by, 2), Child::new(z, 1), Child::new(ab, 2), Child::new(by, 2), Child::new(xabyz, 5)],
                vec![Child::new(b, 1), Child::new(yz, 2), Child::new(ab, 2), Child::new(by, 2), Child::new(xabyz, 5)],
            ];
            let (left, right) = graph.split_index_at_pos(wxabyzabbyxabyz, NonZeroUsize::new(3).unwrap());
            assert_eq!(left, wxa_pattern, "left");
            assert_eq!(right, byzabbyxabyz_pattern, "right");
        } else {
            panic!();
        }
    }
    #[test]
    fn split_child_patterns_6() {
        let mut graph = Hypergraph::new();
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
            // split wxabyz at 3
            let wxa_pattern = vec![
                vec![Child::new(wx, 2), Child::new(a, 1)],
            ];
            let byz_pattern = vec![
                vec![Child::new(by, 2), Child::new(z, 1)],
                vec![Child::new(b, 1), Child::new(yz, 2)],
            ];

            let (left, right) = graph.split_index_at_pos(wxabyz, NonZeroUsize::new(3).unwrap());
            assert_eq!(left, wxa_pattern, "left");
            assert_eq!(right, byz_pattern, "right");
        } else {
            panic!();
        }
    }
}