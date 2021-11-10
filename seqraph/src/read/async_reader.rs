use std::collections::VecDeque;

use crate::{
    r#match::*,
    search::*,
    token::*,
    *,
};
use futures::Stream;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};
use std::pin::Pin;
use async_std::sync::{
    RwLock,
    Arc,
};

#[derive(Debug)]
struct BufferedPatternStream<T: Tokenize> {
    buffer: VecDeque<Child>,
    stream: UnboundedReceiverStream<T>,
}
#[derive(Clone, Debug)]
struct BufferedPatternReceiver<T: Tokenize> {
    offset: usize,
    stream: Arc<RwLock<BufferedPatternStream<T>>>,
}

//impl<T: Tokenize> Stream for BufferedPatternStream<T> {
//    type Item = Result<Child, Token<T>>;
//    fn poll_next(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
//        let tokio_stream::iter(self.buffer.iter()).chain(self.stream);
//        Stream::poll_next(Pin::new(&mut ), cx)
//    }
//}

#[derive(Clone, Debug)]
pub struct AsyncReader<T: Tokenize + Send + Sync, D: AsyncMatchDirection<T> + Clone> {
    graph: HypergraphHandle<T>,
    _ty: std::marker::PhantomData<D>,
}
impl<T: Tokenize + Send + Sync + 'static, D: AsyncMatchDirection<T> + Clone> AsyncReader<T, D> {
    pub(crate) fn right_searcher(&self) -> AsyncSearcher<T, MatchRight> {
        AsyncSearcher::new(self.graph)
    }
    pub async fn read_sequence_stream(&mut self, stream: impl TokenStream<T>) -> SearchResult {
        let stream = Hypergraph::<T>::async_to_token_children_stream(
            self.graph.clone(),
            stream,
        ).await;
        self.async_read_pattern(stream).await
    }
    /// read until first known token and create any unknown token indices
    pub(crate) async fn async_read_unknown_tokens(
        &mut self,
        mut pattern: impl PatternStream<Child, Token<T>>,
    ) -> impl PatternStream<Child, Token<T>> {
        while let Some(r) = pattern.next().await {
            match r.into_inner() {
                Err(token) => {
                    // insert and skip unknown tokens
                    async_std::task::block_on(async {
                        self.graph.write().await.insert_token(token);
                    });
                },
                Ok(c) => break, // stop skipping
            }
        }
        pattern
    }
    pub(crate) async fn async_read_pattern(
        &mut self,
        pattern: impl PatternStream<Child, Token<T>>,
    ) -> SearchResult {
        let mut pattern = self.async_read_unknown_tokens(pattern).await;
        // take first known token
        let head = pattern.next().await
            .map(|r| r.into_inner().unwrap())
            .ok_or(NotFound::EmptyPatterns)?;
        self.right_searcher().find_largest_matching_parent(head, pattern).await
    }
}
