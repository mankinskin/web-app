use crate::{
    pattern::*,
    r#match::*,
};
use itertools::{
    EitherOrBoth,
    Itertools,
};
use std::collections::{
    HashMap,
    HashSet,
};
use futures::{
    Stream,
    StreamExt,
    stream::Fuse,
    task::*,
};
use std::{
    cmp,
    pin::Pin,
    sync::Arc,
};
use pin_project_lite::pin_project;

pin_project! {
    struct ZipLongest<St1: Stream, St2: Stream> {
        #[pin]
        stream1: Fuse<St1>,
        #[pin]
        stream2: Fuse<St2>,
        queued1: Option<St1::Item>,
        queued2: Option<St2::Item>,
    }
}
impl<St1: Stream, St2: Stream> ZipLongest<St1, St2> {
    fn new(stream1: St1, stream2: St2) -> Self {
        Self { stream1: stream1.fuse(), stream2: stream2.fuse(), queued1: None, queued2: None }
    }
}
impl<St1, St2> Stream for ZipLongest<St1, St2>
where
    St1: Stream,
    St2: Stream,
{
    type Item = EitherOrBoth<St1::Item, St2::Item>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        let mut pending = false;
        if this.queued1.is_none() {
            match this.stream1.as_mut().poll_next(cx) {
                Poll::Ready(Some(item1)) => *this.queued1 = Some(item1),
                Poll::Pending => pending |= true,
                Poll::Ready(None) => {}
            }
        }
        if this.queued2.is_none() {
            match this.stream2.as_mut().poll_next(cx) {
                Poll::Ready(Some(item2)) => *this.queued2 = Some(item2),
                Poll::Pending => pending |= true,
                Poll::Ready(None)  => {}
            }
        }
        if pending  {
            Poll::Pending
        } else {
            Poll::Ready(
                match (this.queued1.take(), this.queued2.take()) {
                    (None, None) => None,
                    (Some(a), None) => Some(EitherOrBoth::Left(a)),
                    (None, Some(b)) => Some(EitherOrBoth::Right(b)),
                    (Some(a), Some(b)) => Some(EitherOrBoth::Both(a, b)),
                }
            )
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let queued1_len = if self.queued1.is_some() { 1 } else { 0 };
        let queued2_len = if self.queued2.is_some() { 1 } else { 0 };
        let (stream1_lower, stream1_upper) = self.stream1.size_hint();
        let (stream2_lower, stream2_upper) = self.stream2.size_hint();

        let stream1_lower = stream1_lower.saturating_add(queued1_len);
        let stream2_lower = stream2_lower.saturating_add(queued2_len);

        let lower = cmp::max(stream1_lower, stream2_lower);

        let upper = match (stream1_upper, stream2_upper) {
            (Some(x), Some(y)) => {
                let x = x.saturating_add(queued1_len);
                let y = y.saturating_add(queued2_len);
                Some(cmp::max(x, y))
            }
            (Some(x), None) => x.checked_add(queued1_len),
            (None, Some(y)) => y.checked_add(queued2_len),
            (None, None) => None,
        };

        (lower, upper)
    }
}
fn zip_longest<St1: Stream, St2: Stream>(stream1: St1, stream2: St2) -> ZipLongest<St1, St2> {
    ZipLongest::new(stream1, stream2)
}
async fn skip_matching_stream<'a, T: Tokenize + Send, A: ReturnedPatternStream<T> + 'a, B: ReturnedPatternStream<T> + 'a>(
    mut a: A,
    mut b: B,
) -> Option<(usize, EitherOrBoth<(Child, A), (Child, B)>)> {
    let mut s = zip_longest(&mut a, &mut b)
        .enumerate()
        .skip_while(|(_, eob)| futures::future::ready(match eob {
            EitherOrBoth::Both(Ok(l), Ok(r)) => l == r,
            _ => false,
        }));
    s.next()
        .await
        .map(|(i, eob)|
            (
                i,
                eob.map_left(|c| (c.unwrap(), a))
                   .map_right(|c| (c.unwrap(), b))
            )
        )
}
#[async_trait::async_trait]
pub trait AsyncMatchDirection<T: Tokenize + Send> : MatchDirection {
    async fn skip_equal_indices<'a, A: ReturnedPatternStream<T> + 'a, B: ReturnedPatternStream<T> + 'a>(
        a: A,
        b: B,
    ) -> Option<(
            TokenPosition,
            EitherOrBoth<
                (Child, A),
                (Child, B),
            >
        )>;

    /// get first next token and return remaining stream
    async fn take_head<A: ReturnedPatternStream<T>>(pattern: A) -> Option<Result<(Child, A), Token<T>>>;
    /// get remaining pattern in matching direction including index
    async fn split_end<A: ReturnedPatternStream<T>>(pattern: A, index: PatternId) -> (Pattern, A);
    //async fn split_end_normalized(pattern: impl PatternStream<Child, Token<T>>, index: PatternId) -> (Pattern, impl PatternStream<Child, Token<T>>) {
    //    Self::split_end(pattern, Self::normalize_index(pattern, index))
    //}
    /// get remaining pattern in matching direction excluding index
    async fn front_context<A: ReturnedPatternStream<T>>(mut pattern: A, index: PatternId) -> (Pattern, A);
    //async fn front_context_normalized(pattern: impl PatternStream<Child, Token<T>>, index: PatternId) -> (Pattern, impl PatternStream<Child, Token<T>>) {
    //    Self::front_context(pattern, Self::normalize_index(pattern, index))
    //}
    //async fn normalize_index(pattern: &'_ impl PatternStream<Child, Token<T>>, index: usize) -> usize;

    // prepend remainder to context in direction
    async fn merge_remainder_with_context<A: ReturnedPatternStream<T>>(
        rem: Pattern,
        context: A,
    ) -> A;

    async fn to_found_range(p: Option<impl PatternStream<Child, Token<T>>>, context: impl PatternStream<Child, Token<T>>) -> FoundRange;
}

#[async_trait::async_trait]
impl<T: Tokenize + Send> AsyncMatchDirection<T> for MatchRight {
    /// get the parent where vertex is at the relevant position
    async fn skip_equal_indices<'a, A: ReturnedPatternStream<T> + 'a, B: ReturnedPatternStream<T> + 'a>(
        a: A,
        b: B,
    ) -> Option<(
            TokenPosition,
            EitherOrBoth<
                (Child, A),
                (Child, B),
            >
        )> {
        skip_matching_stream(a, b).await
    }
    async fn take_head<A: ReturnedPatternStream<T>>(mut pattern: A) -> Option<Result<(Child, A), Token<T>>> {
        pattern.next().await.map(|r| r.map(|c| (c, pattern)))
    }
    /// get remaining pattern in matching direction including index
    async fn split_end<A: ReturnedPatternStream<T>>(mut pattern: A, index: PatternId) -> (Pattern, A) {
        // TODO: is this ok?
        let index = Self::index_next(index).unwrap_or(index);
        let back = pattern.take(index).collect::<Vec<_>>().await;
        (back, pattern)
    }
    //async fn split_end_normalized(pattern: impl PatternStream<Child, Token<T>>, index: PatternId) -> (Pattern, impl PatternStream<Child, Token<T>>) {
    //    Self::split_end(pattern, Self::normalize_index(pattern, index))
    //}
    /// get remaining pattern in matching direction excluding index
    async fn front_context<A: ReturnedPatternStream<T>>(mut pattern: A, index: PatternId) -> (Pattern, A) {
        let back = pattern.take(index).collect::<Vec<_>>().await;
        (back, pattern)
    }
    //async fn front_context_normalized(pattern: impl PatternStream<Child, Token<T>>, index: PatternId) -> (Pattern, impl PatternStream<Child, Token<T>>) {
    //    Self::front_context(pattern, Self::normalize_index(pattern, index))
    //}
    /// get remaining pattern agains matching direction excluding index
    //async fn normalize_index(pattern: &'_ impl PatternStream<Child, Token<T>>, index: usize) -> usize;
    async fn merge_remainder_with_context<A: ReturnedPatternStream<T>>(
        rem: Pattern,
        context: A,
    ) -> A {
        context.prepend_pattern(rem)
    }
    async fn to_found_range(p: Option<impl PatternStream<Child, Token<T>>>, context: impl PatternStream<Child, Token<T>>) -> FoundRange {

    }
}
