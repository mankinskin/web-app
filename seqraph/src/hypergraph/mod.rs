use std::fmt::Debug;
use crate::{
    token::{
        Token,
        Tokenize,
    },
};
use std::borrow::Borrow;
use itertools::{
    Itertools,
};

mod search;
mod r#match;
mod split;
mod path_tree;
mod insert;
mod vertex;
mod getters;

pub use vertex::*;
pub use getters::*;


#[derive(Debug)]
pub struct Hypergraph<T: Tokenize> {
    graph: indexmap::IndexMap<VertexKey<T>, VertexData>,
}
impl<'t, 'a, T> Hypergraph<T>
    where T: Tokenize + 't,
{
    pub fn new() -> Self {
        Self {
            graph: indexmap::IndexMap::new(),
        }
    }
    pub fn pattern_width(pat: PatternView<'a>) -> TokenPosition {
        pat.into_iter().fold(0, |acc, child| acc + child.get_width())
    }
    pub fn vertex_count(&self) -> usize {
        self.graph.len()
    }
    //pub fn index_sequence<N: Into<T>, I: IntoIterator<Item = N>>(&mut self, seq: I) -> VertexIndex {
    //    let seq = seq.into_iter();
    //    let tokens = T::tokenize(seq);
    //    let pattern = self.to_token_children(tokens);
    //    self.index_pattern(&pattern[..])
    //}
}
impl<'t, 'a, T> Hypergraph<T>
    where T: Tokenize + 't + std::fmt::Display,
{
    pub fn pattern_string(&'a self, pattern: impl IntoIterator<Item=&'a Child>) -> String {
        pattern.into_iter().map(|child| self.index_string(child.get_index())).join("")
    }
    pub fn key_data_string(&self, key: &VertexKey<T>, data: &VertexData) -> String {
        match key {
            VertexKey::Token(token) => token.to_string(),
            VertexKey::Pattern(_) =>
                self.pattern_string(data.expect_any_pattern()),
        }
    }
    pub fn index_string(&self, index: VertexIndex) -> String {
        let (key, data) = self.expect_vertex(index);
        self.key_data_string(key, data)
    }
    pub fn key_string(&self, key: &VertexKey<T>) -> String {
        let data = self.expect_vertex_data_by_key(key);
        self.key_data_string(key, data)
    }
}

#[cfg(test)]
#[macro_use]
mod tests {
    use super::*;
    use crate::token::*;
    lazy_static::lazy_static! {
        pub static ref 
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
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                VertexIndex,
                ) = {
            let mut graph = Hypergraph::new();
            if let [a, b, c, d, e, f, g, h, i] = graph.insert_tokens(
                [
                    Token::Element('a'),
                    Token::Element('b'),
                    Token::Element('c'),
                    Token::Element('d'),
                    Token::Element('e'),
                    Token::Element('f'),
                    Token::Element('g'),
                    Token::Element('h'),
                    Token::Element('i'),
                ])[..] {
                // abcdefghi
                // ababababcdbcdefdefcdefefghefghghi
                // ->
                // abab ab abcdbcdefdefcdefefghefghghi
                // ab abab abcdbcdefdefcdefefghefghghi

                // abcdbcdef def cdef efgh efgh ghi

                // abcd b cdef
                // abcd bcd ef

                // ab cd
                // abc d
                // a bcd

                let ab = graph.insert_pattern([a, b]);
                let bc = graph.insert_pattern([b, c]);
                let ef = graph.insert_pattern([e, f]);
                let def = graph.insert_pattern([d, ef]);
                let cdef = graph.insert_pattern([c, def]);
                let gh = graph.insert_pattern([g, h]);
                let efgh = graph.insert_pattern([ef, gh]);
                let ghi = graph.insert_pattern([gh, i]);
                let abc = graph.insert_patterns([
                    [ab, c],
                    [a, bc],
                ]);
                let cd = graph.insert_pattern([c, d]);
                let bcd = graph.insert_patterns([
                    [bc, d],
                    [b, cd],
                ]);
                //let abcd = graph.insert_pattern(&[abc, d]);
                //graph.insert_to_pattern(abcd, &[a, bcd]);
                let abcd = graph.insert_patterns([
                    [abc, d],
                    [a, bcd],
                ]);
                let efghi = graph.insert_patterns([
                    [efgh, i],
                    [ef, ghi],
                ]);
                let abcdefghi = graph.insert_pattern([abcd, efghi]);
                let aba = graph.insert_pattern([ab, a]);
                let abab = graph.insert_patterns([
                    [aba, b],
                    [ab, ab],
                ]);
                let ababab = graph.insert_patterns([
                    [abab, ab],
                    [ab, abab],
                ]);
                let ababcd = graph.insert_patterns([
                    [ab, abcd],
                    [aba, bcd],
                    [abab, cd],
                ]);
                let ababababcd = graph.insert_patterns([
                    [ababab, abcd],
                    [abab, ababcd],
                ]);
                let ababcdefghi = graph.insert_patterns([
                    [ab, abcdefghi],
                    [ababcd, efghi],
                ]);
                let ababababcdefghi = graph.insert_patterns([
                    [ababababcd, efghi],
                    [abab, ababcdefghi],
                ]);
                let longer_pattern = graph.insert_pattern([ababab, abcdefghi]);
                (
                    graph,
                    a,
                    b,
                    c,
                    d,
                    e,
                    f,
                    g,
                    h,
                    i,
                    ab,
                    bc,
                    cd,
                    bcd,
                    abc,
                    abcd,
                    cdef,
                    efghi,
                    abab,
                    ababab,
                    ababababcdefghi,
                )
            } else {
                panic!();
            }
        };
    }
}
