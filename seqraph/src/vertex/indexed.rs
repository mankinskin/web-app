use super::*;

pub trait Indexed {
    fn index(&self) -> &VertexIndex;
    fn vertex<'g, T: Tokenize>(&'g self, graph: &'g Hypergraph<T>) -> &'g VertexData {
        graph.expect_vertex_data(self.index())
    }
}
impl<I: Borrow<VertexIndex>> Indexed for I {
    fn index(&self) -> &VertexIndex {
        (*self).borrow()
    }
}

pub trait ToChild: Indexed + Wide {
    fn to_child(&self) -> Child {
        Child::new(self.index(), self.width())
    }
}
impl<T: Indexed + Wide> ToChild for T {}

pub trait MaybeIndexed<T: Tokenize> {
    type Inner: Indexed;
    fn into_inner(self) -> Result<Self::Inner, T>;
}
impl<I: Indexed, T: Tokenize> MaybeIndexed<T> for Result<I, T> {
    type Inner = I;
    fn into_inner(self) -> Result<Self::Inner, T> {
        self
    }
}
//impl<I: Indexed, T: Tokenize> MaybeIndexed<T> for I {
//    type Inner = I;
//    fn into_inner(self) -> Result<Self::Inner, T> {
//        Ok(self)
//    }
//}
