use crate::{
    hypergraph::{
        search::*,
        split::*,
        Child,
        Hypergraph,
    },
    token::Tokenize,
};
use std::collections::HashSet;
trait MergeDirection {
    fn split_context_head(context: SplitSegment) -> Option<(Child, Pattern)>;
    fn split_inner_head(context: SplitSegment) -> Option<(Child, Pattern)>;
    fn concat_inner_and_context(inner_context: Pattern, inner: Pattern, outer_context: Pattern) -> Pattern;
    fn merge_order(inner: Child, head: Child) -> (Child, Child);
    fn minimize_split<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        context: SplitSegment,
        inner: SplitSegment,
    ) -> SplitSegment {

        if let Some((outer_head, outer_tail)) = Self::split_context_head(context) {
            let (inner_head, inner_tail) = Self::split_inner_head(inner).expect("Empty SplitSegment::Pattern");
            let (left, right) = Self::merge_order(inner_head, outer_head);
            // try to find parent matching both
            SplitMinimizer::new(hypergraph).resolve_common_parent(left, right)
                .map_pattern(|pat|
                    Self::concat_inner_and_context(inner_tail, pat, outer_tail)
                )
        } else {
            // context empty
            inner
        }
    }
    /// returns minimal patterns of pattern split
    /// i.e. no duplicate subsequences with respect to entire index
    fn merge_splits<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        splits: Vec<(Pattern, SplitSegment)>,
    ) -> SplitSegment {
        Self::merge_optional_splits(hypergraph, splits.into_iter().map(|(p, c)| (p, Some(c))))
    }
    /// returns minimal patterns of pattern split
    /// i.e. no duplicate subsequences with respect to entire index
    fn merge_optional_splits<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        splits: impl IntoIterator<Item = (Pattern, Option<SplitSegment>)>,
    ) -> SplitSegment {
        match splits
            .into_iter()
            .try_fold(HashSet::new(), |mut acc, (context, inner)| {
                if let Some(inner) = inner {
                    match Self::minimize_split(hypergraph, SplitSegment::Pattern(context), inner) {
                        // stop when single child is found
                        SplitSegment::Child(c) => Err(c),
                        SplitSegment::Pattern(pat) => {
                            acc.insert(pat);
                            Ok(acc)
                        }
                    }
                } else {
                    acc.insert(context);
                    Ok(acc)
                }
            }) {
            Err(child) => {
                //println!("adding [\n{}] to {}",
                //    patterns.clone().into_iter().fold(String::new(), |acc, p| {
                //        format!("{}{},\n", acc, hypergraph.pattern_string(p))
                //    }),
                //    hypergraph.index_string(child),
                //);
                //hypergraph.add_patterns_to_node(child, patterns);
                SplitSegment::Child(child)
            }
            Ok(patterns) => {
                SplitSegment::Child(if patterns.len() == 1 {
                    let pattern = patterns.into_iter().next().unwrap();
                    hypergraph.insert_pattern(pattern)
                } else {
                    hypergraph.insert_patterns(patterns)
                })
            },
        }
    }
}
// context left, inner right
struct MergeLeft;
impl MergeDirection for MergeLeft {
    fn split_context_head(context: SplitSegment) -> Option<(Child, Pattern)> {
        match context {
            SplitSegment::Pattern(mut p) => {
                let last = p.pop();
                last.map(|last| (last, p))
            }
            SplitSegment::Child(c) => Some((c, vec![])),
        }
    }
    fn split_inner_head(context: SplitSegment) -> Option<(Child, Pattern)> {
        MergeRight::split_context_head(context)
    }
    fn merge_order(inner: Child, head: Child) -> (Child, Child) {
        (head, inner)
    }
    fn concat_inner_and_context(inner_context: Pattern, inner: Pattern, outer_context: Pattern) -> Pattern {
        [outer_context, inner, inner_context].concat()
    }
}
// context right, inner left
struct MergeRight;
impl MergeDirection for MergeRight {
    fn split_context_head(context: SplitSegment) -> Option<(Child, Pattern)> {
        match context {
            SplitSegment::Pattern(p) => {
                let mut p = p.into_iter();
                let first = p.next();
                first.map(|last| (last, p.collect()))
            }
            SplitSegment::Child(c) => Some((c, vec![])),
        }
    }
    fn split_inner_head(context: SplitSegment) -> Option<(Child, Pattern)> {
        MergeLeft::split_context_head(context)
    }
    fn merge_order(inner: Child, head: Child) -> (Child, Child) {
        (inner, head)
    }
    fn concat_inner_and_context(inner_context: Pattern, inner: Pattern, outer_context: Pattern) -> Pattern {
        [inner_context, inner, outer_context].concat()
    }
}
#[derive(Debug)]
pub struct SplitMinimizer<'g, T: Tokenize> {
    graph: &'g mut Hypergraph<T>,
}
impl<'g, T: Tokenize> SplitMinimizer<'g, T> {
    pub fn new(graph: &'g mut Hypergraph<T>) -> Self {
        Self {
            graph,
        }
    }
    /// minimize a pattern which has been merged at pos
    fn resolve_common_parent(
        &mut self,
        left: Child,
        right: Child,
    ) -> SplitSegment {
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
            .into()
    }
    /// minimal means:
    /// - no two indicies are adjacient more than once
    /// - no two patterns of the same index share an index border
    /// returns minimal patterns on each side of index split
    pub fn merge_left_splits(
        &mut self,
        splits: Vec<(Pattern, SplitSegment)>,
    ) -> SplitSegment {
        MergeLeft::merge_splits(self.graph, splits)
    }
    pub fn merge_right_splits(
        &mut self,
        splits: Vec<(Pattern, SplitSegment)>,
    ) -> SplitSegment {
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
                match self.resolve_common_parent(l, r) {
                    SplitSegment::Child(c) => Err(c),
                    SplitSegment::Pattern(pat) => {
                        acc.insert(pat);
                        Ok(acc)
                    }
                }
            },
            1 => {
                let (l, _) = MergeLeft::split_context_head(left.clone()).unwrap();
                let (i, _) = MergeRight::split_context_head(infix).unwrap();
                let (r, _) = MergeRight::split_context_head(right.clone()).unwrap();
                match self.resolve_common_parent(l, i) {
                    SplitSegment::Child(lc) => {
                        match self.resolve_common_parent(lc, r) {
                            SplitSegment::Child(c) => Err(c),
                            SplitSegment::Pattern(_) => {
                                match self.resolve_common_parent(i, r) {
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
                        }
                    }
                    SplitSegment::Pattern(_) => {
                        match self.resolve_common_parent(i, r) {
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
#[cfg(test)]
mod tests {
    //use super::*;
    //use crate::{
    //    token::*,
    //    hypergraph::{
    //        split::*,
    //    },
    //};
    //use std::{
    //    num::NonZeroUsize,
    //};
    //use pretty_assertions::assert_eq;

    //#[test]
    //fn merge_split_1() {
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
    //        let cd_pattern = vec![cd];

    //        let index_split = IndexSplitter::build_index_split(&graph, abcd, NonZeroUsize::new(2).unwrap());
    //        let (left, right) = SplitMinimizer::minimize_index_split(&mut graph, index_split);
    //        assert_eq!(left, vec![ab_pattern], "left");
    //        assert_eq!(right, vec![cd_pattern], "right");
    //    } else {
    //        panic!();
    //    }
    //}
    //#[test]
    //fn merge_split_2() {
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
    //        let x_a_pattern = vec![x, a];
    //        let by_z_pattern = vec![by, z];

    //        let (left, right) = graph.split_index_at_pos(xabyz, NonZeroUsize::new(2).unwrap());
    //        let xa_found = graph.find_pattern(x_a_pattern);
    //        if let (xa, _, FoundRange::Complete) = xa_found.expect("xa not found") {
    //            assert_eq!(left, xa, "left");
    //        } else { panic!(); };
    //        let byz_found = graph.find_pattern(by_z_pattern);
    //        if let (byz, _, FoundRange::Complete) = byz_found.expect("byz not found") {
    //            assert_eq!(right, byz, "right");
    //        } else { panic!(); };
    //    } else {
    //        panic!();
    //    }
    //}
}
