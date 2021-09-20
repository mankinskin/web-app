use crate::{
    hypergraph::{
        VertexIndex,
        Hypergraph,
        split::*,
        Indexed,
        Child,
    },
    token::Tokenize,
};
use std::{
    num::NonZeroUsize,
    cmp::PartialEq,
    borrow::Borrow,
    fmt::Display,
};



pub(crate) struct IndexReader;
impl IndexReader {
    pub fn read_sequence<T: Tokenize>(graph: &mut Hypergraph<T>, sequence: impl IntoIterator<Item=T>) -> Child {
        if let Some((child, (pattern_id, sub_index), found_range)) = graph.find_sequence(sequence) {
            match found_range {
                FoundRange::Complete => child,
                FoundRange::Prefix(post) => {},
                FoundRange::Postfix(pre) => {},
                FoundRange::Infix(pre, post) => {},
            }
        } else {

        }
        Child::new(0, 0)
    }
}

mod tests {
    use hypergraph::Hypergraph;
    use futures::{
        SinkExt,
        StreamExt,
    };
    async fn read_text() {
        let (mut tx, mut rx) = futures::channel::mpsc::unbounded::<char>();
        let text = "Hello world!";
        futures::stream::iter(text.chars().map(|c| Ok(c))).forward(tx);

        let mut g = Hypergraph::new();
        g.insert_sequence(rx);
    }
}