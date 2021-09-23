use std::num::NonZeroUsize;

use crate::{
    hypergraph::{
        pattern_width,
        search::*,
        split::*,
        Child,
        Hypergraph,
    },
    token::Tokenize,
};

pub(crate) struct IndexReader<'g, T: Tokenize> {
    graph: &'g mut Hypergraph<T>,
}
impl<'a, T: Tokenize> std::ops::Deref for IndexReader<'a, T> {
    type Target = Hypergraph<T>;
    fn deref(&self) -> &Self::Target {
        self.graph
    }
}
impl<'g, T: Tokenize> std::ops::DerefMut for IndexReader<'g, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.graph
    }
}
impl<'g, T: Tokenize> IndexReader<'g, T> {
    pub fn new(graph: &'g mut Hypergraph<T>) -> Self {
        Self { graph }
    }
    pub fn read_sequence(&mut self, sequence: impl IntoIterator<Item = T>) -> Child {
        match self.find_sequence(sequence) {
            Ok(SearchFound {
                index,
                parent_match,
                ..
            }) => match parent_match.parent_range {
                FoundRange::Complete => index,
                FoundRange::Prefix(post) => {
                    let width = index.width - pattern_width(post);
                    let width =
                        NonZeroUsize::new(width).expect("returned full length postfix remainder");
                    let (c, _) = self.split_index(index, width);
                    c
                }
                FoundRange::Postfix(pre) => {
                    let width = pattern_width(pre);
                    let width =
                        NonZeroUsize::new(width).expect("returned zero length prefix remainder");
                    let (_, c) = self.split_index(index, width);
                    c
                }
                FoundRange::Infix(pre, post) => {
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
            Err(not_found) => panic!("Not found {:?}", not_found),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::hypergraph::*;
    #[test]
    fn read_text() {
        //let (mut tx, mut rx) = futures::channel::mpsc::unbounded::<char>();
        let text = "Hello world!";
        //futures::stream::iter(text.chars().map(|c| Ok(c))).forward(tx);

        let mut g = Hypergraph::default();
        g.read_sequence(text.chars());
    }
}
