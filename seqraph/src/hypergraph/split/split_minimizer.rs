use crate::{
    hypergraph::{
        search::*,
        split::*,
        Child,
        Hypergraph,
    },
    token::Tokenize,
};
use std::{
    collections::HashSet,
    fmt::Display,
};

trait MergeDirection {
    fn split_context_head(context: Pattern) -> Option<(Child, Pattern)>;
    fn concat_inner_and_context(inner: Pattern, context: Pattern) -> Pattern;
    fn merge_order(inner: Child, head: Child) -> (Child, Child);
    fn minimize_split<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        context: Pattern,
        inner: Child,
    ) -> Result<Child, Pattern> {
        if let Some((head, tail)) = Self::split_context_head(context) {
            let (left, right) = Self::merge_order(inner, head);
            // try to find parent matching both
            SplitMinimizer::resolve_common_parent(hypergraph, left, right)
                // if no remaining context, return found inner
                .and_then(|inner| {
                    if tail.is_empty() {
                        Ok(inner)
                    } else {
                        Err(vec![inner])
                    }
                })
                // return not found or found with tail
                .map_err(|pat| Self::concat_inner_and_context(pat, tail))
        } else {
            // context empty
            Ok(inner)
        }
    }
    /// returns minimal patterns of pattern split
    /// i.e. no duplicate subsequences with respect to entire index
    fn merge_splits<T: Tokenize + Display>(
        hypergraph: &mut Hypergraph<T>,
        splits: Vec<(Pattern, Child)>,
    ) -> Child {
        Self::merge_optional_splits(hypergraph, splits.into_iter().map(|(p, c)| (p, Some(c))))
    }
    /// returns minimal patterns of pattern split
    /// i.e. no duplicate subsequences with respect to entire index
    fn merge_optional_splits<T: Tokenize + Display>(
        hypergraph: &mut Hypergraph<T>,
        splits: impl IntoIterator<Item = (Pattern, Option<Child>)>,
    ) -> Child {
        match splits
            .into_iter()
            .try_fold(HashSet::new(), |mut acc, (context, inner)| {
                if let Some(inner) = inner {
                    match Self::minimize_split(hypergraph, context, inner) {
                        Ok(c) => Err(c),
                        Err(pat) => {
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
                child
            }
            Ok(patterns) => hypergraph.insert_patterns(patterns),
        }
    }
}
// context left, inner right
struct MergeLeft;
impl MergeDirection for MergeLeft {
    fn split_context_head(context: Pattern) -> Option<(Child, Pattern)> {
        let mut context = context;
        let last = context.pop();
        last.map(|last| (last, context))
    }
    fn merge_order(inner: Child, head: Child) -> (Child, Child) {
        (head, inner)
    }
    fn concat_inner_and_context(inner: Pattern, context: Pattern) -> Pattern {
        [context, inner].concat()
    }
}
// context right, inner left
struct MergeRight;
impl MergeDirection for MergeRight {
    fn split_context_head(context: Pattern) -> Option<(Child, Pattern)> {
        let mut context = context.into_iter();
        let first = context.next();
        first.map(|first| (first, context.collect()))
    }
    fn merge_order(inner: Child, head: Child) -> (Child, Child) {
        (inner, head)
    }
    fn concat_inner_and_context(inner: Pattern, context: Pattern) -> Pattern {
        [inner, context].concat()
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct SplitMinimizer;
impl SplitMinimizer {
    /// minimize a pattern which has been merged at pos
    fn resolve_common_parent<T: Tokenize>(
        hypergraph: &mut Hypergraph<T>,
        left: Child,
        right: Child,
    ) -> Result<Child, Pattern> {
        //println!("pos: {}, len: {}", pos, pattern.len());
        let p = &[left, right];
        // find pattern over merge position
        hypergraph
            .find_pattern(p)
            .map(|SearchFound(found, pattern_id, sub_index, found_range)| {
                match found_range {
                    FoundRange::Postfix(_) | FoundRange::Prefix(_) | FoundRange::Infix(_, _) => {
                        // create new index and replace in parent
                        let partial = hypergraph.insert_pattern(p);
                        hypergraph.replace_in_pattern(
                            found,
                            pattern_id,
                            sub_index..sub_index + 2,
                            [partial],
                        );
                        partial
                    }
                    FoundRange::Complete => found,
                }
            })
            .map_err(|_| p.to_vec())
    }
    /// minimal means:
    /// - no two indicies are adjacient more than once
    /// - no two patterns of the same index share an index border
    /// returns minimal patterns on each side of index split
    pub fn merge_left_splits<T: Tokenize + Display>(
        hypergraph: &mut Hypergraph<T>,
        splits: Vec<(Pattern, Child)>,
    ) -> Child {
        MergeLeft::merge_splits(hypergraph, splits)
    }
    pub fn merge_right_splits<T: Tokenize + Display>(
        hypergraph: &mut Hypergraph<T>,
        splits: Vec<(Pattern, Child)>,
    ) -> Child {
        MergeRight::merge_splits(hypergraph, splits)
    }
    pub fn merge_left_optional_splits<T: Tokenize + Display>(
        hypergraph: &mut Hypergraph<T>,
        splits: Vec<(Pattern, Option<Child>)>,
    ) -> Child {
        MergeLeft::merge_optional_splits(hypergraph, splits)
    }
    pub fn merge_right_optional_splits<T: Tokenize + Display>(
        hypergraph: &mut Hypergraph<T>,
        splits: Vec<(Pattern, Option<Child>)>,
    ) -> Child {
        MergeRight::merge_optional_splits(hypergraph, splits)
    }
    /// returns minimal patterns of pattern split
    /// i.e. no duplicate subsequences with respect to entire index
    pub fn merge_inner_optional_splits<T: Tokenize + Display>(
        hypergraph: &mut Hypergraph<T>,
        splits: Vec<(Option<Child>, Pattern, Option<Child>)>,
    ) -> Child {
        match splits
            .into_iter()
            .try_fold(HashSet::new(), |mut acc, (left, infix, right)| {
                match (left, right) {
                    (Some(left), Some(right)) => match infix.len() {
                        0 => match Self::resolve_common_parent(hypergraph, left, right) {
                            Ok(c) => Err(c),
                            Err(pat) => {
                                acc.insert(pat);
                                Ok(acc)
                            }
                        },
                        1 => {
                            let inner = *infix.first().unwrap();
                            match Self::resolve_common_parent(hypergraph, left, inner) {
                                Ok(lc) => {
                                    match Self::resolve_common_parent(hypergraph, lc, right) {
                                        Ok(c) => Err(c),
                                        Err(_) => {
                                            match Self::resolve_common_parent(
                                                hypergraph, inner, right,
                                            ) {
                                                Ok(rc) => {
                                                    acc.insert(vec![lc, right]);
                                                    acc.insert(vec![left, rc]);
                                                }
                                                Err(_) => {
                                                    acc.insert(vec![lc, right]);
                                                }
                                            }
                                            Ok(acc)
                                        }
                                    }
                                }
                                Err(_) => {
                                    match Self::resolve_common_parent(hypergraph, inner, right) {
                                        Ok(c) => {
                                            acc.insert(vec![left, c]);
                                        }
                                        Err(_) => {
                                            acc.insert(vec![left, inner, right]);
                                        }
                                    };
                                    Ok(acc)
                                }
                            }
                        }
                        _ => {
                            let left =
                                MergeRight::minimize_split(hypergraph, infix, left).unwrap_err();
                            let right =
                                MergeLeft::minimize_split(hypergraph, left, right).unwrap_err();
                            acc.insert(right);
                            Ok(acc)
                        }
                    },
                    (Some(left), None) => {
                        match MergeRight::minimize_split(hypergraph, infix, left) {
                            Ok(c) => Err(c),
                            Err(pat) => {
                                acc.insert(pat);
                                Ok(acc)
                            }
                        }
                    }
                    (None, Some(right)) => {
                        match MergeLeft::minimize_split(hypergraph, infix, right) {
                            Ok(c) => Err(c),
                            Err(pat) => {
                                acc.insert(pat);
                                Ok(acc)
                            }
                        }
                    }
                    (None, None) => match infix.len() {
                        1 => Err(*infix.first().unwrap()),
                        0 => panic!("Empty inner pattern in merge patterns"),
                        _ => {
                            acc.insert(infix);
                            Ok(acc)
                        }
                    },
                }
            }) {
            Ok(patterns) => {
                let c = hypergraph.insert_patterns(patterns.clone());
                println!(
                    "created {} from [\n{}]",
                    hypergraph.index_string(c),
                    patterns.into_iter().fold(String::new(), |acc, p| {
                        format!("{}{},\n", acc, hypergraph.pattern_string(p))
                    })
                );
                c
            }
            Err(child) => child,
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
