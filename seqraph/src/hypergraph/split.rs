use crate::{
    hypergraph::{
        Hypergraph,
        VertexIndex,
        PatternIndex,
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
        index: PatternIndex,
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
        patterns: impl Iterator<Item=(&'a PatternIndex, impl Iterator<Item=&'a Child> + 'a)> + 'a,
        pos: NonZeroUsize,
        ) -> impl Iterator<Item=(PatternIndex, (IndexPosition, TokenPosition))> + 'a {
        patterns.filter_map(move |(i, pattern)|
            Self::find_pattern_split_index(pattern, pos).map(|split| (*i, split))
        )
    }
    fn get_split_neighbors_and_next_parent(&self, split: &Split, tree: &PathTree, half: Either<(), ()>) -> (&[Child], Option<TreeParent>) {
        if let Some(tree_parent) = &split.parent {
            let (next_parent, index) = tree.get_parents().get(tree_parent.tree_node).unwrap();
            let IndexInParent {
                pattern_index,
                replaced_index,
            } = tree_parent.index_in_parent;
            let parent_node = self.get_vertex_data(index).unwrap();
            let neighbors = match half {
                Either::Left(_) => parent_node.get_child_pattern_range(&pattern_index, ..replaced_index).unwrap(),
                Either::Right(_) => parent_node.get_child_pattern_range(&pattern_index, replaced_index+1..).unwrap(),
            };
            (neighbors, next_parent.clone())
        } else {
            (&[], None)
        }
    }
    fn build_split_halves_from_tree(&self, mut splits: Vec<Split>, tree: &PathTree, half: Either<(), ()>) -> Vec<Pattern> {
        if splits.is_empty() {
            return Vec::new();
        }
        loop {
            let largest = splits.iter().max_by(|a, b| a.width.cmp(&b.width)).unwrap();
            let (neighbors, next_parent) = self.get_split_neighbors_and_next_parent(largest, tree, half);
            // if at end
            if neighbors.is_empty() && next_parent.is_none() {
                return vec![largest.pattern.clone()];
            } else {
                for split in splits.iter_mut() {
                    let (neighbors, next_parent) = self.get_split_neighbors_and_next_parent(split, tree, half);
                    let width = Self::pattern_width(neighbors);
                    split.width += width;
                    split.pattern = match half {
                        Either::Left(_) => [neighbors, &split.pattern].concat(),
                        Either::Right(_) => [neighbors, &split.pattern].concat(),
                    };
                    split.parent = next_parent;
                }
            }
            if next_parent.is_none() {
                break;
            }
        }
        splits.into_iter().map(|split| split.pattern).collect()
    }
    fn build_splits_from_tree(&self, mut splits: Vec<(Split, Split)>, tree: PathTree) -> (Vec<Pattern>, Vec<Pattern>) {
        let (left, right): (Vec<_>, Vec<_>) = splits.into_iter().unzip();
        (
            self.build_split_halves_from_tree(left, &tree, Either::Left(())),
            self.build_split_halves_from_tree(right, &tree, Either::Right(()))
        )
    }
    pub fn split_index_at_pos(&self, root: VertexIndex, pos: NonZeroUsize) -> (Vec<Pattern>, Vec<Pattern>) {
        let mut queue = VecDeque::from_iter(std::iter::once(vec![IndexPositionDescriptor {
                node: root,
                offset: pos,
                parent: None,
            }]));
        let mut path_tree = PathTree::new();
        loop {
            let mut splits = Vec::new();
            let mut next_children = Vec::new();
            let children = queue.pop_front().unwrap();
            for IndexPositionDescriptor {
                    node: current_index,
                    offset,
                    parent,
                 } in children.into_iter() {
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
                    Ok(mut split_indices) => {
                        // no perfect split
                        // add current node to path tree
                        let tree_node = path_tree.add_element(parent, current_index);
                        split_indices.sort_unstable_by(|(ind_a, _), (ind_b, _)| 
                            current_node.get_child_pattern_position(&ind_a.pattern_index, ind_a.replaced_index).unwrap().get_width()
                                .cmp(&current_node.get_child_pattern_position(&ind_b.pattern_index, ind_b.replaced_index).unwrap().get_width())
                        );
                        next_children.extend(split_indices.into_iter()
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
            if splits.is_empty() {
                queue.push_back(next_children);
            } else {
                return self.build_splits_from_tree(splits, path_tree);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::hypergraph::tests::CONTEXT;
    use pretty_assertions::assert_eq;
    use maplit::hashset;
    use std::collections::HashSet;
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
            cd,
            _bcd,
            abc,
            _abcd,
            _cdef,
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
            cd,
            _bcd,
            abc,
            abcd,
            _cdef,
            ) = &*CONTEXT;
        let (left, right) = graph.split_index_at_pos(*abcd, NonZeroUsize::new(3).unwrap());
        assert_eq!(left, vec![vec![Child::new(*abc, 3)]], "left");
        assert_eq!(right, vec![vec![Child::new(*d, 1)]], "right");
    }
    #[test]
    fn split_child_patterns_3() {
        let (
            graph,
            a,
            _b,
            c,
            d,
            _e,
            _f,
            _g,
            _h,
            _i,
            ab,
            _bc,
            cd,
            bcd,
            abc,
            abcd,
            _cdef,
            ) = &*CONTEXT;
        let ab_pattern = vec![Child::new(ab, 2)];
        let cd_pattern = vec![Child::new(cd, 2)];

        let abcd_patterns = graph.expect_vertex_data(abcd).get_children().clone();
        assert_eq!(abcd_patterns.into_values().collect::<HashSet<_>>(), hashset![
            [Child::new(abc, 3), Child::new(d, 1)].iter().cloned().collect::<Vec<_>>(),
            [Child::new(a, 1), Child::new(bcd, 3)].iter().cloned().collect::<Vec<_>>(),
        ]);

        let (left, right) = graph.split_index_at_pos(*abcd, NonZeroUsize::new(2).unwrap());
        assert_eq!(left, vec![ab_pattern], "left");
        assert_eq!(right, vec![cd_pattern], "right");
    }
}