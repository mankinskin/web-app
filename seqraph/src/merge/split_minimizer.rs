use crate::{
    merge::merge_direction::*,
    search::*,
    split::*,
    vertex::*,
    Child,
    Hypergraph,
};

use std::collections::HashSet;
#[derive(Debug)]
pub struct SplitMinimizer<'g, T: Tokenize> {
    graph: &'g mut Hypergraph<T>,
}
impl<'g, T: Tokenize> SplitMinimizer<'g, T> {
    pub fn new(graph: &'g mut Hypergraph<T>) -> Self {
        Self { graph }
    }
    /// minimize a pattern which has been merged at pos
    pub fn resolve_common_parent(&mut self, left: Child, right: Child) -> Result<Child, Pattern> {
        //println!("pos: {}, len: {}", pos, pattern.len());
        let p = vec![left, right];
        // find pattern over merge position
        self.graph
            .find_pattern(p.as_pattern_view())
            .map(
                |SearchFound {
                     index: found,
                     pattern_id,
                     sub_index,
                     parent_match,
                 }| {
                    match parent_match.parent_range {
                        FoundRange::Postfix(_)
                        | FoundRange::Prefix(_)
                        | FoundRange::Infix(_, _) => {
                            // create new index and replace in parent
                            let partial = self.graph.insert_pattern(p.clone());
                            self.graph.replace_in_pattern(
                                found,
                                pattern_id,
                                sub_index..sub_index + 2,
                                [partial],
                            );
                            partial
                        }
                        FoundRange::Complete => found,
                    }
                },
            )
            .map_err(|_| p.into_pattern())
    }
    /// minimal means:
    /// - no two indicies are adjacient more than once
    /// - no two patterns of the same index share an index border
    pub fn merge_left_splits(&mut self, splits: Vec<(Pattern, SplitSegment)>) -> SplitSegment {
        MergeLeft::merge_splits(self.graph, splits)
    }
    pub fn merge_right_splits(&mut self, splits: Vec<(Pattern, SplitSegment)>) -> SplitSegment {
        MergeRight::merge_splits(self.graph, splits)
    }
    pub fn merge_left_optional_splits(
        &mut self,
        splits: Vec<(Pattern, Option<SplitSegment>)>,
    ) -> SplitSegment {
        MergeLeft::merge_optional_splits(self.graph, splits)
    }
    pub fn merge_right_optional_splits(
        &mut self,
        splits: Vec<(Pattern, Option<SplitSegment>)>,
    ) -> SplitSegment {
        MergeRight::merge_optional_splits(self.graph, splits)
    }
    /// returns minimal patterns of pattern split
    /// i.e. no duplicate subsequences with respect to entire index
    pub fn merge_inner_optional_splits(
        &mut self,
        splits: Vec<(Option<SplitSegment>, SplitSegment, Option<SplitSegment>)>,
    ) -> Child {
        match splits
            .into_iter()
            .try_fold(HashSet::new(), |mut acc, (left, infix, right)| {
                match (left, right) {
                    (Some(left), Some(right)) => self.add_inner_split(acc, left, infix, right),
                    (Some(left), None) => {
                        match MergeRight::minimize_split(self.graph, infix, left) {
                            SplitSegment::Child(c) => Err(c),
                            SplitSegment::Pattern(pat) => {
                                acc.insert(pat);
                                Ok(acc)
                            }
                        }
                    }
                    (None, Some(right)) => {
                        match MergeLeft::minimize_split(self.graph, infix, right) {
                            SplitSegment::Child(c) => Err(c),
                            SplitSegment::Pattern(pat) => {
                                acc.insert(pat);
                                Ok(acc)
                            }
                        }
                    }
                    (None, None) => match infix.len() {
                        1 => Err(infix.unwrap_child()),
                        0 => panic!("Empty inner pattern in merge patterns"),
                        _ => {
                            acc.insert(infix.unwrap_pattern());
                            Ok(acc)
                        }
                    },
                }
            }) {
            Ok(patterns) => {
                self.graph.insert_patterns(patterns)
                //println!(
                //    "created {} from [\n{}]",
                //    hypergraph.index_string(c),
                //    patterns.into_iter().fold(String::new(), |acc, p| {
                //        format!("{}{},\n", acc, hypergraph.pattern_string(p))
                //    })
                //);
            }
            Err(child) => child,
        }
    }
    fn add_inner_split(
        &mut self,
        mut acc: HashSet<Pattern>,
        left: SplitSegment,
        infix: SplitSegment,
        right: SplitSegment,
    ) -> Result<HashSet<Pattern>, Child> {
        match infix.len() {
            0 => {
                let (l, _ltail) = MergeLeft::split_context_head(left).unwrap();
                let (r, _rtail) = MergeRight::split_context_head(right).unwrap();
                match self.resolve_common_parent(l, r).into() {
                    SplitSegment::Child(c) => Err(c),
                    SplitSegment::Pattern(pat) => {
                        acc.insert(pat);
                        Ok(acc)
                    }
                }
            }
            1 => {
                let (l, _) = MergeLeft::split_context_head(left.clone()).unwrap();
                let (i, _) = MergeRight::split_context_head(infix).unwrap();
                let (r, _) = MergeRight::split_context_head(right.clone()).unwrap();
                match self.resolve_common_parent(l, i).into() {
                    SplitSegment::Child(lc) => match self.resolve_common_parent(lc, r).into() {
                        SplitSegment::Child(c) => Err(c),
                        SplitSegment::Pattern(_) => {
                            match self.resolve_common_parent(i, r).into() {
                                SplitSegment::Child(rc) => {
                                    acc.insert(lc.into_iter().chain(right).collect());
                                    acc.insert(left.into_iter().chain(rc).collect());
                                }
                                SplitSegment::Pattern(_) => {
                                    acc.insert(lc.into_iter().chain(right).collect());
                                }
                            }
                            Ok(acc)
                        }
                    },
                    SplitSegment::Pattern(_) => {
                        match self.resolve_common_parent(i, r).into() {
                            SplitSegment::Child(c) => {
                                acc.insert(left.into_iter().chain(c).collect());
                            }
                            SplitSegment::Pattern(_) => {
                                acc.insert(left.into_iter().chain(i).chain(right).collect());
                            }
                        };
                        Ok(acc)
                    }
                }
            }
            _ => {
                let left = MergeRight::minimize_split(self.graph, infix, left);
                let right = MergeLeft::minimize_split(self.graph, left, right).unwrap_pattern();
                acc.insert(right);
                Ok(acc)
            }
        }
    }
}
