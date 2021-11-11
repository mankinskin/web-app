use crate::*;
use std::fmt::Debug;
use tokio_stream::Stream;

/// trait for types which can used to read a pattern, with unknown size
pub trait PatternStream<I: Indexed, T: Tokenize = NoToken>:
    Stream<Item = Result<I, T>> + Unpin + Debug + Send
{
}
impl<I: Indexed, T: Tokenize, S: Stream<Item = Result<I, T>> + Unpin + Debug + Send>
    PatternStream<I, T> for S
{
}

/// trait for types which can used to read a pattern, with unknown size
pub trait TokenStream<T: Tokenize + Send>: Stream<Item = T> + Unpin + Debug + Send {}
impl<T: Tokenize + Send, S: Stream<Item = T> + Unpin + Debug + Send> TokenStream<T> for S {}

pub trait ReturnedPatternStream<T: Tokenize + Send>:
    PatternStream<Child, Token<T>, Item = Result<Child, Token<T>>>
{
}
impl<T: Tokenize + Send, A: PatternStream<Child, Token<T>, Item = Result<Child, Token<T>>>>
    ReturnedPatternStream<T> for A
{
}
