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
    //pub fn split_by_path(
    //    &self,
    //    index: VertexIndex,
    //    path: Vec<PatternIndex>,
    //    ) -> (Pattern, Pattern) {
    //}
    pub fn find_pattern_split_index(
        pattern: impl Iterator<Item=&'a Child>,
        pos: NonZeroUsize,
        ) -> Option<(PatternIndex, TokenPosition)> {
        let mut skipped = 0;
        let pos: TokenPosition = pos.into();
        // find child overlapping with cut pos or 
        pattern.enumerate()
            .find_map(|(i, child)| {
                if skipped + child.width <= pos {
                    skipped += child.width;
                    None
                } else {
                    Some((i, pos - skipped))
                }
            })
    }
    pub fn find_child_pattern_split_indices(
        patterns: impl Iterator<Item=impl Iterator<Item=&'a Child> + 'a> + 'a,
        pos: NonZeroUsize,
        ) -> impl Iterator<Item=(PatternIndex, TokenPosition)> + 'a {
        patterns.filter_map(move |pattern|
            Self::find_pattern_split_index(pattern, pos)
        )
    }
    fn build_split_from_tree(&self, mut split: (Pattern, Pattern), tree_parent: Option<TreeParent>, tree: PathTree) -> (Pattern, Pattern) {
        if let Some(mut parent) = tree_parent {
            //self.build_split_from_path(split, parent.index_in_parent, path)
            while let Some((next_parent, index)) = tree.get_parents().get(parent.tree_node) {
                let IndexInParent {
                    pattern_index,
                    replaced_index,
                } = parent.index_in_parent;
                let current_node = self.get_vertex_data(index).unwrap();
                split = ([
                        &current_node.children[pattern_index][..replaced_index],
                        &split.0[..],
                    ].concat(),
                    [
                        &split.1[..],
                        &current_node.children[pattern_index][replaced_index+1..],
                    ].concat()
                );
                if let Some(next_parent) = next_parent {
                    parent = next_parent.clone();
                } else {
                    break;
                }
            }
        }
        split
    }
    pub fn split_index_at_pos(&self, root: VertexIndex, pos: NonZeroUsize) -> (Pattern, Pattern) {
        let mut queue = VecDeque::from_iter(std::iter::once(IndexPositionDescriptor {
                node: root,
                offset: pos,
                parent: None,
            }));
        let mut path_tree = PathTree::new();
        loop {
            let IndexPositionDescriptor {
                node: current_index,
                offset,
                parent,
             } = queue.pop_front().unwrap();
            let current_node = self.get_vertex_data(current_index).unwrap();
            let child_slices = current_node.children.iter().map(|p| p.iter());
            let split_indices = Hypergraph::<T>::find_child_pattern_split_indices(child_slices, offset);
            let perfect_split = split_indices.enumerate()
                .map(|(i, (index, offset))| {
                    let index_in_parent = IndexInParent {
                        pattern_index: i,
                        replaced_index: index,
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
                    let split = Hypergraph::<T>::split_pattern_at_index(&current_node.children[pattern_index], split_index);
                    return self.build_split_from_tree(split, parent, path_tree);
                },
                Ok(mut split_indices) => {
                    // no perfect split
                    // add current node to path tree
                    let tree_node = path_tree.add_element(parent, current_index);
                    split_indices.sort_unstable_by(|(ind_a, _), (ind_b, _)| 
                        current_node.children[ind_a.pattern_index][ind_a.replaced_index].width
                            .cmp(&current_node.children[ind_b.pattern_index][ind_b.replaced_index].width)
                    );
                    queue.extend(split_indices.into_iter()
                        .map(|(index_in_parent, offset)| {
                        let IndexInParent { pattern_index, replaced_index } = index_in_parent.clone();
                        IndexPositionDescriptor {
                            node: current_node.children[pattern_index][replaced_index].index,
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
            _bcd,
            abc,
            _abcd,
            ) = &*CONTEXT;
        let (left, right) = graph.split_index_at_pos(*abc, NonZeroUsize::new(2).unwrap());
        assert_eq!(left, vec![Child::new(*ab, 2)], "left");
        assert_eq!(right, vec![Child::new(*c, 1)], "right");
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
            _bcd,
            abc,
            abcd,
            ) = &*CONTEXT;
        let (left, right) = graph.split_index_at_pos(*abcd, NonZeroUsize::new(3).unwrap());
        assert_eq!(left, vec![Child::new(*abc, 3)], "left");
        assert_eq!(right, vec![Child::new(*d, 1)], "right");
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
            bcd,
            abc,
            abcd,
            ) = &*CONTEXT;
        let ab_pattern = &[Child::new(ab, 2)];
        let c_d_pattern = &[Child::new(c, 1), Child::new(d, 1)];

        let abcd_patterns = graph.expect_vertex_data(abcd).children.clone();
        assert_eq!(abcd_patterns.into_iter().collect::<HashSet<_>>(), hashset![
            [Child::new(abc, 3), Child::new(d, 1)].iter().cloned().collect::<Vec<_>>(),
            [Child::new(a, 1), Child::new(bcd, 3)].iter().cloned().collect::<Vec<_>>(),
        ]);

        let (left, right) = graph.split_index_at_pos(*abcd, NonZeroUsize::new(2).unwrap());
        assert_eq!(left, ab_pattern.iter().cloned().collect::<Vec<_>>(), "left");
        assert_eq!(right, c_d_pattern.iter().cloned().collect::<Vec<_>>(), "right");
    }
}