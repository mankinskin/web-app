use std::fmt::Debug;
use crate::{
    //pattern::{
    //    Pattern,
    //},
    token::{
        Token,
        Tokenize,
    },
};
use std::collections::{
    HashSet,
    VecDeque,
};
use std::borrow::Borrow;
use itertools::{
    Itertools,
    EitherOrBoth,
};
use either::Either;
use std::num::NonZeroUsize;
use std::iter::FromIterator;

type VertexIndex = usize;
type VertexParents = Vec<Parent>;
type ChildPatterns = Vec<Pattern>;
type ChildPatternView<'a> = &'a[PatternView<'a>];
type Pattern = Vec<Child>;
type TokenPosition = usize;
type PatternIndex = usize;
type IndexPattern = Vec<VertexIndex>;
type VertexPattern = Vec<VertexData>;
type PatternView<'a> = &'a[Child];
type VertexPatternView<'a> = Vec<&'a VertexData>;
type VertexPatternViewMut<'a> = Vec<&'a mut VertexData>;
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum VertexKey<T: Tokenize> {
    Token(Token<T>),
    Pattern(VertexIndex)
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Parent {
    index: VertexIndex, // the parent pattern
    width: TokenPosition,
    pattern_indices: HashSet<(usize, PatternIndex)>, // positions of child in parent patterns
}
impl Parent {
    pub fn new(index: VertexIndex, width: TokenPosition) -> Self {
        Self {
            index,
            width,
            pattern_indices: Default::default(),
        }
    }
    pub fn add_pattern_index(&mut self, pattern: usize, index: PatternIndex) {
        self.pattern_indices.insert((pattern, index));
    }
}
impl Into<VertexIndex> for Parent {
    fn into(self) -> VertexIndex {
        self.index
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Child {
    index: VertexIndex, // the child index
    width: TokenPosition, // the token width
}
impl Child {
    pub fn new(index: VertexIndex, width: TokenPosition) -> Self {
        Self {
            index,
            width,
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct VertexData {
    width: TokenPosition,
    parents: VertexParents,
    children: ChildPatterns,
}
impl VertexData {
    pub fn with_width(width: TokenPosition) -> Self {
        Self {
            width,
            parents: VertexParents::new(),
            children: ChildPatterns::new(),
        }
    }
    pub fn add_pattern<'c, I: IntoIterator<Item=&'c Child>>(&mut self, pat: I) {
        // TODO: detect unmatching pattern
        self.children.push(pat.into_iter().cloned().collect())
    }
    pub fn add_parent(&mut self, vertex: VertexIndex, width: TokenPosition, pattern: usize, index: PatternIndex) {
        if let Some(parent) = self.parents
            .iter_mut()
                .find(|parent| parent.index == vertex) {
                    parent.add_pattern_index(pattern, index);
                } else {
                    let mut parent = Parent::new(vertex, width);
                    parent.add_pattern_index(pattern, index);
                    self.parents.push(parent);
                }
    }
}

#[derive(Debug)]
pub struct Hypergraph<T: Tokenize> {
    graph: indexmap::IndexMap<VertexKey<T>, VertexData>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PatternMatch {
    Remainder(Either<Pattern, Pattern>),
    Matching,
}
impl PatternMatch {
    pub fn flip_remainder(self) -> Self {
        match self {
            Self::Remainder(e) => Self::Remainder(e.flip()),
            _ => self,
        }
    }
}
//impl From<SearchFound> for PatternMatch {
//    fn from(SearchFound(range, index, offset): SearchFound) -> Self {
//        match (offset, range) {
//            (0, FoundRange::Complete) => Self::Matching,
//            (0, FoundRange::Prefix(remainder)) => Self::Remainder(Either::Left(remainder)),
//            _ => Self::Mismatch,
//        }
//    }
//}
//impl From<SearchFound> for IndexMatch {
//    fn from(SearchFound(range, index, offset): SearchFound) -> Self {
//        match (offset, range) {
//            (0, FoundRange::Complete) => Self::Matching,
//            (0, FoundRange::Prefix(remainder)) => Self::SubRemainder(remainder),
//            _ => Self::Mismatch,
//        }
//    }
//}
impl From<IndexMatch> for PatternMatch {
    fn from(r: IndexMatch) -> Self {
        match r {
            IndexMatch::SubRemainder(p) => Self::Remainder(Either::Left(p)),
            IndexMatch::SupRemainder(p) => Self::Remainder(Either::Right(p)),
            IndexMatch::Matching => Self::Matching,
        }
    }
}
impl From<PatternMatch> for IndexMatch {
    fn from(r: PatternMatch) -> Self {
        match r {
            PatternMatch::Remainder(e) => match e {
                Either::Left(p) => Self::SubRemainder(p),
                Either::Right(p) => Self::SupRemainder(p),
            },
            PatternMatch::Matching => Self::Matching,
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IndexMatch {
    SupRemainder(Pattern),
    SubRemainder(Pattern),
    Matching,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FoundRange {
    Complete, // Full index found
    Prefix(Pattern), // found prefix (remainder)
    Postfix(Pattern), // found postfix (remainder)
    Infix(Pattern, Pattern), // found infix
}
impl FoundRange {
    pub fn prepend_prefix(self, pattern: Pattern) -> Self {
        if pattern.is_empty() {
            return self;
        }
        match self {
            FoundRange::Complete => FoundRange::Prefix(pattern),
            FoundRange::Prefix(post) => FoundRange::Infix(pattern, post),
            FoundRange::Infix(pre, post) => FoundRange::Infix([&pattern[..], &pre[..]].concat(), post),
            FoundRange::Postfix(pre) => FoundRange::Postfix([&pattern[..], &pre[..]].concat()),
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SearchFound(FoundRange, VertexIndex, PatternIndex);
// found range of search pattern in vertex at index

impl SearchFound {
    //pub fn from_match_result_on_index_at_offset(result: PatternMatch, index: VertexIndex, offset: Option<PatternIndex>) -> Self {
    //    let offset = offset.unwrap_or(0);
    //    match result {
    //        PatternMatch::Matching => Self(FoundRange::Complete, index, offset),
    //        PatternMatch::Remainder(Either::Left(rem)) => Self(FoundRange::Prefix(rem), index, offset),
    //    }
    //}
    pub fn prepend_prefix(self, pattern: Pattern) -> Self {
        Self(self.0.prepend_prefix(pattern), self.1, self.2)
    }
}
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
// describes search position of child in tree
#[derive(Debug, PartialEq, Eq, Clone)]
struct IndexPositionDescriptor {
    parent: Option<TreeParent>,
    node: VertexIndex,
    offset: NonZeroUsize,
}
// refers to position of child in parent
#[derive(Debug, PartialEq, Eq, Clone)]
struct TreeParent {
    tree_node: usize,
    index_in_parent: IndexInParent,
}
#[derive(Debug, PartialEq, Eq, Clone)]
struct IndexInParent {
    pattern_index: usize,
    replaced_index: usize,
}
#[derive(Debug, PartialEq, Eq, Clone)]
struct PathDescriptor {
    index: VertexIndex,
    index_in_parent: IndexInParent
}
#[derive(Debug, PartialEq, Eq, Clone)]
struct PathTree {
    parents: Vec<(Option<TreeParent>, VertexIndex)>
}
impl PathTree {
    pub fn new() -> Self {
        Self {
            parents: Default::default(),
        }
    }
    pub fn add_element(&mut self, parent: Option<TreeParent>, index: VertexIndex) -> usize {
        let id = self.parents.len();
        self.parents.push((parent, index));
        id
    }
    pub fn add_child_of(&mut self, parent: TreeParent, index: VertexIndex) -> usize {
        self.add_element(Some(parent), index)
    }
}

impl<'t, 'a, T> Hypergraph<T>
    where T: Tokenize + 't + std::fmt::Display,
{
    fn sub_pattern_string(&'a self, pattern: impl IntoIterator<Item=&'a Child>) -> String {
        pattern.into_iter().map(|child| self.sub_index_string(child.index)).join("")
    }
    fn pattern_string(&self, pattern: PatternView<'_>) -> String {
        pattern.iter().map(|child| self.sub_index_string(child.index)).join("_")
    }
    fn sub_index_string(&self, index: VertexIndex) -> String {
        let (key, data) = self.expect_vertex(index);
        match key {
            VertexKey::Token(token) => token.to_string(),
            VertexKey::Pattern(_) => {
                self.sub_pattern_string(&data.children.get(0).expect("Pattern vertex with no children!")[..])
            },
        }
    }
    fn index_string(&self, index: VertexIndex) -> String {
        let (key, data) = self.expect_vertex(index);
        match key {
            VertexKey::Token(token) => token.to_string(),
            VertexKey::Pattern(_) => {
                self.pattern_string(&data.children.get(0).expect("Pattern vertex with no children!")[..])
            },
        }
    }
}
impl<'t, 'a, T> Hypergraph<T>
    where T: Tokenize + 't,
{
    pub fn new() -> Self {
        Self {
            graph: indexmap::IndexMap::new(),
        }
    }
    fn get_token_index(&self, token: &Token<T>) -> Option<VertexIndex> {
        self.graph.get_index_of(&VertexKey::Token(*token))
    }
    fn get_token_data(&self, token: &Token<T>) -> Option<&VertexData> {
        self.graph.get(&VertexKey::Token(*token))
    }
    fn get_token_data_mut(&mut self, token: &Token<T>) -> Option<&mut VertexData> {
        self.graph.get_mut(&VertexKey::Token(*token))
    }
    fn get_vertex_data<I: Borrow<VertexIndex>>(&self, index: I) -> Option<&VertexData> {
        self.get_vertex(index).map(|(_, v)| v)
    }
    fn expect_vertex_data<I: Borrow<VertexIndex>>(&self, index: I) -> &VertexData {
        self.expect_vertex(index).1
    }
    fn expect_vertex<I: Borrow<VertexIndex>>(&self, index: I) -> (&VertexKey<T>, &VertexData) {
        self.get_vertex(index).expect("Invalid index!")
    }
    fn get_vertex_data_mut(&mut self, index: VertexIndex) -> Option<&mut VertexData> {
        self.get_vertex_mut(index).map(|(_, v)| v)
    }
    fn get_vertex<I: Borrow<VertexIndex>>(&self, index: I) -> Option<(&VertexKey<T>, &VertexData)> {
        self.graph.get_index(*index.borrow())
    }
    fn get_vertex_mut(&mut self, index: VertexIndex) -> Option<(&mut VertexKey<T>, &mut VertexData)> {
        self.graph.get_index_mut(index)
    }
    pub fn insert_token(&mut self, token: Token<T>) -> VertexIndex {
        self.insert_vertex(VertexKey::Token(token), VertexData::with_width(1))
    }
    pub fn insert_vertex(&mut self, key: VertexKey<T>, data: VertexData) -> VertexIndex {
        self.graph.insert_full(key, data).0
    }
    fn to_token_indices(&mut self, tokens: impl IntoIterator<Item=Token<T>>) -> IndexPattern {
        tokens.into_iter()
            .map(|token|
                 self.get_token_index(&token)
                 .unwrap_or_else(|| self.insert_token(token))
                )
            .collect()
    }
    fn to_token_children(&mut self, tokens: impl IntoIterator<Item=Token<T>>) -> Pattern {
        self.to_token_indices(tokens).into_iter()
            .map(|index| Child { index, width: 1, })
            .collect()
    }
    fn expect_vertices<I: Borrow<VertexIndex>>(&self, indices: impl Iterator<Item=I>) -> VertexPatternView<'_> {
        indices
            .map(move |index| self.expect_vertex_data(index))
            .collect()
    }
    fn get_vertices<I: Borrow<VertexIndex>>(&self, indices: impl Iterator<Item=I>) -> Option<VertexPatternView<'_>> {
        indices
            .map(move |index| self.get_vertex_data(index))
            .collect()
    }
    fn get_token_indices(&mut self, tokens: impl Iterator<Item=&'t Token<T>>) -> IndexPattern {
        let mut v = IndexPattern::with_capacity(tokens.size_hint().0);
        for token in tokens {
            let index = self.get_token_index(token)
                .unwrap_or_else(|| self.insert_token(token.clone()));
            v.push(index);
        }
        v
    }
    fn pattern_width(pat: PatternView<'a>) -> TokenPosition {
        pat.into_iter().fold(0, |acc, child| acc + child.width)
    }
    fn pick_best_matching_child_pattern(
        child_patterns: &'a ChildPatterns,
        candidates: impl Iterator<Item=&'a (usize, PatternIndex)>,
        post_sub_pat: PatternView<'a>,
        ) -> Option<PatternView<'a>> {
        candidates.find_or_first(|(pattern_index, sub_index)|
            post_sub_pat.get(0).and_then(|i|
                    child_patterns[*pattern_index]
                        .get(sub_index+1)
                        .map(|b| i.index == b.index)
            ).unwrap_or(false)
        ).and_then(|&(pattern_index, sub_index)|
            child_patterns[pattern_index].get(sub_index..)
        )
    }
    /// match sub_pat against children in parent with an optional offset.
    fn compare_parent_at_offset(
        &self,
        post_pattern: PatternView<'a>,
        parent: &Parent,
        offset: Option<PatternIndex>,
        ) -> Option<IndexMatch> {
        // find pattern where sub is at offset
        println!("compare_parent_at_offset");
        let vert = self.expect_vertex_data(parent.index);
        let child_patterns = &vert.children;
        //print!("matching parent \"{}\" ", self.sub_index_string(parent.index));
        // optionally filter by sub offset
        let candidates = Self::get_pattern_index_candidates(parent, offset);
        //println!("with successors \"{}\"", self.pattern_string(post_pattern));
        // find child pattern with matching successor or pick first candidate
        let best_match = Self::pick_best_matching_child_pattern(
            &child_patterns,
            candidates,
            post_pattern,
        );
        best_match.and_then(|child_pattern|
            //println!("comparing post sub pattern with remaining children of parent");
            self.compare_pattern_prefix(
                post_pattern,
                child_pattern.get(1..).unwrap_or(&[])
                ).map(Into::into)
            // returns result of matching sub with parent's children
        )
    }
    fn get_pattern_index_candidates(
        parent: &'a Parent,
        offset: Option<PatternIndex>,
        ) -> impl Iterator<Item=&(usize, PatternIndex)> {
        if let Some(offset) = offset {
            print!("at offset = {} ", offset);
            Either::Left(parent.pattern_indices.iter()
                .filter(move |(_pattern_index, sub_index)| *sub_index == offset))
        } else {
            print!("at offset = 0");
            Either::Right(parent.pattern_indices.iter())
        }
    }
    fn get_direct_vertex_parent_with_offset(
        vertex: &'a VertexData,
        parent_index: VertexIndex,
        offset: Option<PatternIndex>,
        ) -> Option<&'a Parent> {
        vertex.parents.iter()
            .find(|Parent { index, pattern_indices, .. }|
                *index == parent_index &&
                offset.map(|offset|
                    pattern_indices.iter().any(|(_, pos)| *pos == offset)
                ).unwrap_or(true)
            )
    }
    fn get_direct_vertex_parent_at_prefix(
        vertex: &'a VertexData,
        index: VertexIndex,
        ) -> Option<&'a Parent> {
        Self::get_direct_vertex_parent_with_offset(&vertex, index, Some(0))
    }
    /// find an index at the prefix of a pattern
    fn match_sub_and_post_with_index(&self,
            sub: VertexIndex,
            post_pattern: PatternView<'a>,
            index: VertexIndex,
            width: TokenPosition,
        ) -> Option<IndexMatch> {
        println!("match_sub_pattern_to_super");
        // search parent of sub
        if sub == index {
            return if post_pattern.is_empty() {
                Some(IndexMatch::Matching)
            } else {
                Some(IndexMatch::SubRemainder(post_pattern.into()))
            }
        }
        let vertex = self.expect_vertex_data(sub);
        if vertex.parents.len() < 1 {
            return None;
        }
        let sup_parent = Self::get_direct_vertex_parent_at_prefix(&vertex, index);
        if let Some(parent) = sup_parent {
            // parents contain sup
            println!("sup found in parents");
            self.compare_parent_at_offset(post_pattern, parent, Some(0))
        } else {
            // sup is no direct parent, search upwards
            println!("matching available parents");
            // search sup in parents
            let (parent, index_match) = self.find_parent_matching_pattern_at_offset_below_width(
                post_pattern,
                &vertex,
                Some(0),
                Some(width),
            )?;
            println!("found parent matching");
            let new_post = match index_match {
                // found index for complete pattern, tr
                IndexMatch::Matching => Some(Vec::new()),
                // found matching parent larger than the pattern, not the one we were looking for
                IndexMatch::SupRemainder(_) => None,
                // found parent matching with prefix of pattern, continue
                IndexMatch::SubRemainder(rem) => Some(rem),
            }?;
            // TODO: faster way to handle empty new_post
            println!("matching on parent with remainder");
            self.match_sub_and_post_with_index(parent.index, &new_post, index, width)
        }
    }
    fn match_pattern_with_index(
        &self,
        sub_pattern: PatternView<'a>,
        index: VertexIndex,
        width: TokenPosition,
        ) -> Option<IndexMatch> {
        println!("match_sub_pattern_to_super");
        let sub = sub_pattern.get(0)?;
        let post_pattern = sub_pattern.get(1..);
        if let None = post_pattern {
            return if sub.index == index {
                Some(IndexMatch::Matching)
            } else {
                None
            };
        }
        let post_pattern = post_pattern?;
        self.match_sub_and_post_with_index(sub.index, post_pattern, index, width)
    }
    // find parent of vertex matching a pattern
    fn find_parent_matching_pattern_at_offset_below_width(
        &self,
        post_pattern: PatternView<'a>,
        vertex: &VertexData,
        offset: Option<PatternIndex>,
        width_ceiling: Option<TokenPosition>,
        ) -> Option<(Parent, IndexMatch)> {
        println!("find_parent_matching_pattern");
        let parents = &vertex.parents;
        // optionally filter parents by width
        if let Some(ceil) = width_ceiling {
            Either::Left(parents.iter().filter(move |parent| parent.width < ceil))
        } else {
            Either::Right(parents.iter())
        }
        // find matching parent
        .find_map(|parent|
            self.compare_parent_at_offset(post_pattern, parent, offset)
                .map(|m| (parent.clone(), m))
        )
    }
    pub fn find_pattern(
        &self,
        pattern: PatternView<'a>,
        ) -> Option<(VertexIndex, IndexMatch)> {
        let vertex = self.expect_vertex_data(pattern.get(0)?.index);
        if pattern.len() == 1 {
            return Some((pattern[0].index, IndexMatch::Matching));
        }
        let width = Self::pattern_width(pattern);
        //let mut pattern_iter = pattern.into_iter().cloned().enumerate();
        // accumulate prefix not found
        //let mut prefix = Vec::with_capacity(pattern_iter.size_hint().0);
        self.find_parent_matching_pattern_at_offset_below_width(&pattern[1..], vertex, Some(0), Some(width+1))
            .and_then(|(p, m)| match m {
                IndexMatch::SubRemainder(rem) =>
                    self.find_pattern(&[&[Child::new(p.index, p.width)], &rem[..]].concat())
                    .or(Some((p.index, IndexMatch::SubRemainder(rem)))),
                _ => Some((p.index, m)),
            })
    }
    fn compare_pattern_prefix(
            &self,
            pattern_a: PatternView<'a>,
            pattern_b: PatternView<'a>,
        ) -> Option<PatternMatch> {
        //println!("compare_pattern_prefix(\"{}\", \"{}\")", self.pattern_string(pattern_a), self.pattern_string(pattern_b));
        let pattern_a_iter = pattern_a.iter();
        let pattern_b_iter = pattern_b.iter();
        let mut zipped = pattern_a_iter
            .zip_longest(pattern_b_iter)
            .enumerate()
            .skip_while(|(_, eob)|
                match eob {
                    EitherOrBoth::Both(a, b) => a == b,
                    _ => false,
                }
            );
        let (pos, eob) = if let Some(next) = zipped.next() {
            next
        } else {
            return Some(PatternMatch::Matching);
        };
        Some(match eob {
            // different elements on both sides
            EitherOrBoth::Both(&Child {
                index: index_a,
                width: width_a,
            }, &Child {
                index: index_b,
                width: width_b,
            }) => {
                //println!("matching \"{}\" and \"{}\"", self.sub_index_string(index_a), self.sub_index_string(index_b));
                // Note: depending on sizes of a, b it may be differently efficient
                // to search for children or parents, large patterns have less parents,
                // small patterns have less children
                // search larger in parents of smaller
                let post_sub_pattern;
                let post_sup;
                let sub;
                let sup;
                let sup_width;
                let rotate = if width_a == width_b {
                    // relatives can not have same sizes
                    return None;
                } else if width_a < width_b {
                    println!("right super");
                    post_sub_pattern = &pattern_a[pos+1..];
                    post_sup = &pattern_b[pos+1..];
                    sub = index_a;
                    sup = index_b;
                    sup_width = width_b;
                    false
                } else {
                    println!("left super");
                    post_sub_pattern = &pattern_b[pos+1..];
                    post_sup = &pattern_a[pos+1..];
                    sub = index_b;
                    sup = index_a;
                    sup_width = width_a;
                    true
                };
                let result = self.match_sub_and_post_with_index(
                    sub,
                    post_sub_pattern,
                    sup,
                    sup_width,
                );
                // left remainder: sub remainder
                // right remainder: sup remainder
                // matching: sub & sup finished
                println!("return {:#?}", result);
                let result = match result? {
                    IndexMatch::SubRemainder(rem) =>
                        self.compare_pattern_prefix(
                            &rem,
                            post_sup,
                        )?,
                    IndexMatch::SupRemainder(rem) => PatternMatch::Remainder(Either::Right([&rem, post_sup].concat())),
                    IndexMatch::Matching => {
                        let rem: Vec<_> = post_sup.iter().cloned().collect();
                        if rem.is_empty() {
                            PatternMatch::Matching
                        } else {
                            PatternMatch::Remainder(Either::Right(rem))
                        }
                    },
                };
                if rotate {
                    result.flip_remainder()
                } else {
                    result
                }
            },
            EitherOrBoth::Left(_) => PatternMatch::Remainder(Either::Left(pattern_a[pos..].iter().cloned().collect())),
            EitherOrBoth::Right(_) => PatternMatch::Remainder(Either::Right(pattern_b[pos..].iter().cloned().collect())),
        })
    }
    pub fn index_sequence<N: Into<T>, I: IntoIterator<Item = N>>(&mut self, seq: I) -> VertexIndex {
        let seq = seq.into_iter();
        let tokens = T::tokenize(seq);
        let pattern = self.to_token_children(tokens);
        self.index_pattern(&pattern[..])
    }
    pub fn index_pattern(&mut self, pattern: PatternView<'a>) -> VertexIndex {
        self.index_prefix(pattern)
    }
    pub fn index_prefix(&mut self, pattern: PatternView<'a>) -> VertexIndex {
        unimplemented!()
    }
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
    fn build_split_from_tree(&self, mut split: (Pattern, Pattern), tree_parent: Option<TreeParent>, mut tree: PathTree) -> (Pattern, Pattern) {
        if let Some(mut parent) = tree_parent {
            //self.build_split_from_path(split, parent.index_in_parent, path)
            while let Some((next_parent, index)) = tree.parents.get(parent.tree_node) {
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
    fn split_index_at_pos(&self, root: VertexIndex, pos: NonZeroUsize) -> (Pattern, Pattern) {
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
                Ok(split_indices) => {
                    // no perfect split
                    // add current node to path tree
                    let tree_node = path_tree.add_element(parent, current_index);
                    queue.extend(split_indices.into_iter().map(|(index_in_parent, offset)| {
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
    use crate::token::*;
    lazy_static::lazy_static! {
        static ref 
            CONTEXT: (
                Hypergraph<char>,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                [Child; 2],
                [Child; 2],
                [Child; 2],
                [Child; 2],
                [Child; 2],
                [Child; 3],
                [Child; 3],
                [Child; 1],
                [Child; 1],
                [Child; 3],
                [Child; 3],
                ) = {
        let mut g = Hypergraph::new();
        let a = g.insert_token(Token::Element('a'));
        let b = g.insert_token(Token::Element('b'));
        let c = g.insert_token(Token::Element('c'));
        let d = g.insert_token(Token::Element('d'));
        let e = g.insert_token(Token::Element('e'));

        let mut ab_data = VertexData::with_width(2);
        let a_b_pattern = [Child::new(a, 1), Child::new(b, 1)];
        ab_data.add_pattern(&a_b_pattern);
        let ab = g.insert_vertex(VertexKey::Pattern(0), ab_data);
        g.get_vertex_data_mut(a).unwrap().add_parent(ab, 2, 0, 0);
        g.get_vertex_data_mut(b).unwrap().add_parent(ab, 2, 0, 1);

        let mut bc_data = VertexData::with_width(2);
        let b_c_pattern = [Child::new(b, 1), Child::new(c, 1)];
        bc_data.add_pattern(&b_c_pattern);
        let bc = g.insert_vertex(VertexKey::Pattern(1), bc_data);
        g.get_vertex_data_mut(b).unwrap().add_parent(bc, 2, 0, 0);
        g.get_vertex_data_mut(c).unwrap().add_parent(bc, 2, 0, 1);

        let a_bc_pattern = [Child::new(a, 1), Child::new(bc, 2)];
        let ab_c_pattern = [Child::new(ab, 2), Child::new(c, 1)];

        let mut abc_data = VertexData::with_width(3);
        abc_data.add_pattern(&a_bc_pattern);
        abc_data.add_pattern(&ab_c_pattern);
        let abc = g.insert_vertex(VertexKey::Pattern(2), abc_data);
        let abc_d_pattern = [Child::new(abc, 3), Child::new(d, 1)];
        let a_bc_d_pattern = [Child::new(a, 1), Child::new(bc, 2), Child::new(d, 1)];
        let ab_c_d_pattern = [Child::new(ab, 2), Child::new(c, 1), Child::new(d, 1)];
        g.get_vertex_data_mut(a).unwrap().add_parent(abc, 3, 0, 0);
        g.get_vertex_data_mut(bc).unwrap().add_parent(abc, 3, 0, 1);
        g.get_vertex_data_mut(ab).unwrap().add_parent(abc, 3, 1, 0);
        g.get_vertex_data_mut(c).unwrap().add_parent(abc, 3, 1, 1);
        let a_b_c_pattern = &[Child::new(a, 1), Child::new(b, 1), Child::new(c, 1)];


        let mut abcd_data = VertexData::with_width(4);
        abcd_data.add_pattern(&abc_d_pattern);
        let abcd = g.insert_vertex(VertexKey::Pattern(3), abcd_data);
        g.get_vertex_data_mut(abc).unwrap().add_parent(abcd, 4, 0, 0);
        g.get_vertex_data_mut(d).unwrap().add_parent(abcd, 4, 0, 1);
        let abcd_pattern = [Child::new(abcd, 4)];
        let bc_pattern = [Child::new(bc, 2)];
        let a_d_c_pattern = [Child::new(a, 1), Child::new(d, 1), Child::new(c, 1)];
        let a_b_c_pattern = [Child::new(a, 1), Child::new(b, 1), Child::new(c, 1)];
        (
            g,
            a,
            b,
            c,
            d,
            e,
            ab,
            bc,
            abc,
            abcd,
            a_b_pattern,
            b_c_pattern,
            a_bc_pattern,
            ab_c_pattern,
            abc_d_pattern,
            a_bc_d_pattern,
            ab_c_d_pattern,
            abcd_pattern,
            bc_pattern,
            a_d_c_pattern,
            a_b_c_pattern,
        )
                };
    }
    #[test]
    fn match_simple() {
        let (
            G,
            a,
            b,
            c,
            d,
            e,
            ab,
            bc,
            abc,
            abcd,
            a_b_pattern,
            b_c_pattern,
            a_bc_pattern,
            ab_c_pattern,
            abc_d_pattern,
            a_bc_d_pattern,
            ab_c_d_pattern,
            abcd_pattern,
            bc_pattern,
            a_d_c_pattern,
            a_b_c_pattern,
            ) = &*CONTEXT;
        assert_eq!(G.compare_pattern_prefix(a_bc_pattern, ab_c_pattern), Some(PatternMatch::Matching));
        assert_eq!(G.compare_pattern_prefix(ab_c_pattern, a_bc_pattern), Some(PatternMatch::Matching));
        assert_eq!(G.compare_pattern_prefix(a_b_c_pattern, a_bc_pattern), Some(PatternMatch::Matching));
        assert_eq!(G.compare_pattern_prefix(a_b_c_pattern, a_b_c_pattern), Some(PatternMatch::Matching));
        assert_eq!(G.compare_pattern_prefix(ab_c_pattern, a_b_c_pattern), Some(PatternMatch::Matching));
        assert_eq!(G.compare_pattern_prefix(a_bc_d_pattern, ab_c_d_pattern), Some(PatternMatch::Matching));

        assert_eq!(G.compare_pattern_prefix(abc_d_pattern, a_bc_d_pattern), Some(PatternMatch::Matching));
        assert_eq!(G.compare_pattern_prefix(bc_pattern, abcd_pattern), None);
        assert_eq!(G.compare_pattern_prefix(b_c_pattern, a_bc_pattern), None);
        assert_eq!(G.compare_pattern_prefix(b_c_pattern, a_d_c_pattern), None);

        assert_eq!(G.compare_pattern_prefix(a_bc_d_pattern, abc_d_pattern), Some(PatternMatch::Matching));
        assert_eq!(G.compare_pattern_prefix(a_bc_d_pattern, abcd_pattern), Some(PatternMatch::Matching));
        assert_eq!(G.compare_pattern_prefix(abcd_pattern, a_bc_d_pattern), Some(PatternMatch::Matching));

        assert_eq!(G.compare_pattern_prefix(a_b_c_pattern, abcd_pattern), Some(PatternMatch::Remainder(Either::Right(vec![Child::new(*d, 1)]))));

        assert_eq!(G.compare_pattern_prefix(ab_c_d_pattern, a_bc_pattern), Some(PatternMatch::Remainder(Either::Left(vec![Child::new(*d, 1)]))));
        assert_eq!(G.compare_pattern_prefix(a_bc_pattern, ab_c_d_pattern), Some(PatternMatch::Remainder(Either::Right(vec![Child::new(*d, 1)]))));
    }
    #[test]
    fn find_simple() {
        let (
            G,
            a,
            b,
            c,
            d,
            e,
            ab,
            bc,
            abc,
            abcd,
            a_b_pattern,
            b_c_pattern,
            a_bc_pattern,
            ab_c_pattern,
            abc_d_pattern,
            a_bc_d_pattern,
            ab_c_d_pattern,
            abcd_pattern,
            bc_pattern,
            a_d_c_pattern,
            a_b_c_pattern,
            ) = &*CONTEXT;
        assert_eq!(G.find_pattern(bc_pattern), Some((*bc, IndexMatch::Matching)));
        assert_eq!(G.find_pattern(b_c_pattern), Some((*bc, IndexMatch::Matching)));
        assert_eq!(G.find_pattern(a_bc_pattern), Some((*abc, IndexMatch::Matching)));
        assert_eq!(G.find_pattern(ab_c_pattern), Some((*abc, IndexMatch::Matching)));
        assert_eq!(G.find_pattern(a_bc_d_pattern), Some((*abcd, IndexMatch::Matching)));
        assert_eq!(G.find_pattern(a_b_c_pattern), Some((*abc, IndexMatch::Matching)));
        let a_b_c_c_pattern = &[&a_b_c_pattern[..], &[Child::new(*c, 1)]].concat();
        assert_eq!(G.find_pattern(a_b_c_c_pattern), Some((*abc, IndexMatch::SubRemainder(vec![Child::new(*c, 1)]))));
    }
    #[test]
    fn split_index_1() {
        let (
            G,
            a,
            b,
            c,
            d,
            e,
            ab,
            bc,
            abc,
            abcd,
            a_b_pattern,
            b_c_pattern,
            a_bc_pattern,
            ab_c_pattern,
            abc_d_pattern,
            a_bc_d_pattern,
            ab_c_d_pattern,
            abcd_pattern,
            bc_pattern,
            a_d_c_pattern,
            a_b_c_pattern,
            ) = &*CONTEXT;
        let (left, right) = G.split_index_at_pos(*abc, NonZeroUsize::new(2).unwrap());
        assert_eq!(left, vec![Child::new(*ab, 2)], "left");
        assert_eq!(right, vec![Child::new(*c, 1)], "right");
    }
    #[test]
    fn split_child_patterns_2() {
        let (
            G,
            a,
            b,
            c,
            d,
            e,
            ab,
            bc,
            abc,
            abcd,
            a_b_pattern,
            b_c_pattern,
            a_bc_pattern,
            ab_c_pattern,
            abc_d_pattern,
            a_bc_d_pattern,
            ab_c_d_pattern,
            abcd_pattern,
            bc_pattern,
            a_d_c_pattern,
            a_b_c_pattern,
            ) = &*CONTEXT;

        let (left, right) = G.split_index_at_pos(*abcd, NonZeroUsize::new(3).unwrap());
        assert_eq!(left, vec![Child::new(*abc, 3)], "left");
        assert_eq!(right, vec![Child::new(*d, 1)], "right");
    }
    #[test]
    fn split_child_patterns_3() {
        let (
            G,
            a,
            b,
            c,
            d,
            e,
            ab,
            bc,
            abc,
            abcd,
            a_b_pattern,
            b_c_pattern,
            a_bc_pattern,
            ab_c_pattern,
            abc_d_pattern,
            a_bc_d_pattern,
            ab_c_d_pattern,
            abcd_pattern,
            bc_pattern,
            a_d_c_pattern,
            a_b_c_pattern,
            ) = &*CONTEXT;
        let ab_pattern = [Child::new(*ab, 2)];
        let c_d_pattern = [Child::new(*c, 1), Child::new(*d, 1)];
        let abcd_data = G.expect_vertex_data(abcd).clone();
        let patterns = abcd_data.children;
        assert_eq!(patterns, vec![
            abc_d_pattern.into_iter().cloned().collect::<Vec<_>>(),
        ]);

        let (left, right) = G.split_index_at_pos(*abcd, NonZeroUsize::new(2).unwrap());
        assert_eq!(left, ab_pattern.iter().cloned().collect::<Vec<_>>(), "left");
        assert_eq!(right, c_d_pattern.iter().cloned().collect::<Vec<_>>(), "right");
    }
}
