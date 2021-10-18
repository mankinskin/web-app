use std::collections::VecDeque;

use crate::{
    hypergraph::{
        r#match::*,
        search::*,
        *,
    },
    token::*,
};
use std::{
    num::NonZeroUsize,
};

#[derive(Debug)]
struct ReaderCache {
    pub(crate) index: Child,
    pub(crate) pattern_id: Option<PatternId>,
}
impl ReaderCache {
    fn new<T: Tokenize + std::fmt::Display>(graph: &'_ mut Hypergraph<T>, new: impl IntoIterator<Item=Child>) -> Self {
        let (index, pattern_id) = graph.insert_pattern_with_id(new);
        Self {
            index,
            pattern_id,
        }
    }
    fn update_index<T: Tokenize + std::fmt::Display>(&mut self, graph: &'_ mut Hypergraph<T>, new: impl IntoIterator<Item=Child>) {
        if let Some(pid) = &self.pattern_id {
            graph.append_to_pattern(self.index, *pid, new);
        } else {
            let (index, pattern_id) = graph.insert_pattern_with_id(std::iter::once(self.index).chain(new));
            self.index = index;
            self.pattern_id = pattern_id;
        }
    }
}
#[derive(Debug)]
pub struct Reader<'a, T: Tokenize, D: MatchDirection> {
    graph: &'a mut Hypergraph<T>,
    cache: Option<ReaderCache>,
    _ty: std::marker::PhantomData<D>,
}
impl<'a, T: Tokenize, D: MatchDirection> std::ops::Deref for Reader<'a, T, D> {
    type Target = Hypergraph<T>;
    fn deref(&self) -> &Self::Target {
        self.graph
    }
}
impl<'a, T: Tokenize, D: MatchDirection> std::ops::DerefMut for Reader<'a, T, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.graph
    }
}
impl<'a, T: Tokenize + std::fmt::Display, D: MatchDirection> Reader<'a, T, D> {
    pub(crate) fn new(graph: &'a mut Hypergraph<T>) -> Self {
        Self {
            graph,
            cache: None,
            _ty: Default::default(),
        }
    }
    pub(crate) fn right_searcher(&self) -> Searcher<T, MatchRight> {
        Searcher::new(self.graph)
    }
    fn index_tokens(&mut self, sequence: impl IntoIterator<Item=T>) -> NewTokenIndices {
        sequence.into_iter()
            .map(|t| Token::Element(t))
            .map(|t|
                match self.get_token_index(&t) {
                    Ok(i) => NewTokenIndex::Known(i),
                    Err(_) => {
                        let i = self.insert_token(t);
                        NewTokenIndex::New(i.index)
                    },
                }
            )
            .collect()
    }
    fn take_block<I, J: Iterator<Item=I> + itertools::PeekingNext>(iter: &mut J, f: impl FnMut(&I) -> bool) -> Pattern
        where Child: From<I>
    {
        iter.peeking_take_while(f).map(Child::from).collect()
    }
    fn find_known_block(&mut self, sequence: NewTokenIndices) -> (Pattern, Pattern, NewTokenIndices) {
        let mut seq_iter = sequence.into_iter().peekable();
        let cache = Self::take_block(&mut seq_iter, |t| t.is_new());
        let known = Self::take_block(&mut seq_iter, |t| t.is_known());
        (cache, known, seq_iter.collect())
    }
    fn update_cache_index(&mut self, new: impl IntoIterator<Item=Child>) {
        if let Some(cache) = &mut self.cache {
            cache.update_index(self.graph, new)
        } else {
            self.cache = Some(ReaderCache::new(self.graph, new));
        }
        println!("Cache index contains: {:?}", self.cache.as_ref()
            .map(|c| self.graph.index_string(c.index))
            .unwrap_or_default()
        );
    }
    pub(crate) fn read_sequence(&mut self, sequence: impl IntoIterator<Item=T>) -> Child {
        let sequence: NewTokenIndices = self.index_tokens(sequence);
        self.try_read_sequence(sequence).expect("Empty sequence")
    }
    fn try_read_sequence(&mut self, sequence: NewTokenIndices) -> Option<Child> {
        if sequence.is_empty() {
            return None;
        }
        let (cache, known, new) = self.find_known_block(sequence);
        self.update_cache_index(cache);
        let known_str = self.graph.pattern_string(&known);
        let new_str = self.graph.pattern_string(&new);
        //println!("cache: \"{}\"", self.graph.pattern_string(&self.cache));
        println!("known: \"{}\"\nnew: \"{}\"", known_str, new_str);
        let res = match known.len() {
            0 => None,
            1 => Some(*known.first().unwrap()),
            _ => Some(match self.find_pattern(&known) {
                Ok(SearchFound {
                    index,
                    parent_match,
                    ..
                }) => match parent_match.parent_range {
                    FoundRange::Complete => {
                        println!("Found full index");
                        index
                    },
                    FoundRange::Prefix(post) => {
                        println!("Found prefix");
                        let width = index.width - pattern_width(post);
                        let width =
                            NonZeroUsize::new(width).expect("returned full length postfix remainder");
                        let (c, _) = self.split_index(index, width);
                        c
                    }
                    FoundRange::Postfix(pre) => {
                        println!("Found postfix");
                        let width = pattern_width(pre);
                        let width =
                            NonZeroUsize::new(width).expect("returned zero length prefix remainder");
                        let (_, c) = self.split_index(index, width);
                        c
                    }
                    FoundRange::Infix(pre, post) => {
                        println!("Found infix");
                        let pre_width = pattern_width(pre);
                        let post_width = pattern_width(post);
                        match self.index_subrange(index, pre_width..index.width - post_width) {
                            RangeSplitResult::Full(c) => c,
                            RangeSplitResult::Single(l, r) => {
                                if pre_width == 0 {
                                    l
                                } else {
                                    r
                                }
                            }
                            RangeSplitResult::Double(_, c, _) => c,
                            RangeSplitResult::None => panic!("range not in index"),
                        }
                    }
                },
                Err(not_found) => match not_found {
                    NotFound::NoMatchingParent => {
                        // create new index for this known block
                        println!("No matching parents for {}", known_str);
                        println!("Inserting new pattern");
                        self.graph.insert_pattern(known)
                    },
                    _ => panic!("Not found {:?}", not_found),
                }
            })
        };
        self.update_cache_index(res);
        let res = self.try_read_sequence(new);
        if res.is_none() {
            self.cache.as_ref().map(|c| c.index)
        } else {
            res
        }
        //Child::INVALID
    }
}
