use crate::{
    hypergraph::{
        Hypergraph,
        vertex::*,
        r#match::*,
    },
    token::*,
};
use std::borrow::Borrow;
use super::VertexIndex;
mod reader;
pub use reader::*;
//mod async_reader;
//pub use async_reader::*;
#[derive(Clone, Debug, Copy, Hash, PartialEq, Eq)]
pub(crate) enum NewTokenIndex {
    New(VertexIndex),
    Known(VertexIndex),
}
impl NewTokenIndex {
    pub fn is_known(&self) -> bool {
        matches!(self, Self::Known(_))
    }
    pub fn is_new(&self) -> bool {
        matches!(self, Self::New(_))
    }
    #[allow(unused)]
    pub fn into_inner(self) -> VertexIndex {
        match self {
            Self::New(i) => i,
            Self::Known(i) => i,
        }
    }
}
impl Wide for NewTokenIndex {
    fn width(&self) -> usize {
        1
    }
}
impl Borrow<VertexIndex> for NewTokenIndex {
    fn borrow(&self) -> &VertexIndex {
        match self {
            Self::New(i) => i,
            Self::Known(i) => i,
        }
    }
}
impl Borrow<VertexIndex> for &'_ NewTokenIndex {
    fn borrow(&self) -> &VertexIndex {
        (*self).index()
    }
}
impl Borrow<VertexIndex> for &'_ mut NewTokenIndex {
    fn borrow(&self) -> &VertexIndex {
        (*self).index()
    }
}
pub(crate) type NewTokenIndices = Vec<NewTokenIndex>;
impl<T: Tokenize + Send + std::fmt::Display> Hypergraph<T> {
    pub fn right_reader(&mut self) -> Reader<'_, T, MatchRight> {
        Reader::new(self)
    }
    pub fn left_reader(&mut self) -> Reader<'_, T, MatchLeft> {
        Reader::new(self)
    }
    pub fn read_sequence(&mut self, sequence: impl IntoIterator<Item=T>) -> Child {
        self.right_reader().read_sequence(sequence)
    }
}

#[cfg(test)]
mod tests {
    use crate::hypergraph::*;
    //use tokio::sync::mpsc;
    //use tokio_stream::wrappers::*;

    #[tokio::test]
    async fn sync_read_text() {
        let text = "Heldldo world!";
        let mut g = Hypergraph::default();
        let result = g.read_sequence(text.chars().collect());
        assert_eq!(result.width, text.len());
    }
    //#[tokio::test]
    //async fn async_read_text() {
    //    let (mut tx, mut rx) = mpsc::unbounded_channel::<char>();
    //    let text = "Hello world!";
    //    text.chars().for_each(|c| tx.send(c).unwrap());
    //    let mut g = Hypergraph::default();
    //    let rx = UnboundedReceiverStream::new(rx);
    //    let result = g.read_sequence(text.chars().collect());
    //    assert_eq!(result.width, text.len());
    //}
}
