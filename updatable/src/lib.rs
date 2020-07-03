extern crate serde;
extern crate derive_updatable;
pub use derive_updatable::*;

pub trait Updatable: Clone {
    type Update: Update<Self>;
}
pub trait Update<T> : Clone
    where T: Clone + Updatable<Update=Self>,
{
    fn update(&self, data: &mut T);
}

macro_rules! impl_updatable {
    ($($t:ty),+ $(,)?) => {
        $(impl Updatable for $t
        {
            type Update = Self;
        }
        impl Update<$t> for $t
        {
            fn update(&self, data: &mut $t) {
                *data = *self;
            }
        })+
    }
}

impl_updatable!{
    bool,
    u8, u16, u32, u64,
    i8, i16, i32, i64,
    f32, f64,
    char,
    usize, isize,
}
impl Updatable for String {
     type Update = Self;
}
impl Update<String> for String {
    fn update(&self, data: &mut String) {
        *data = self.clone();
    }
}
impl<T> Updatable for Vec<T>
    where T: Clone
{
     type Update = Self;
}
impl<T> Update<Vec<T>> for Vec<T>
    where T: Clone
{
    fn update(&self, data: &mut Vec<T>) {
        data.extend(self.iter().cloned());
    }
}
impl<T> Updatable for Option<T>
    where T: Clone
{
     type Update = Self;
}
impl<T> Update<Option<T>> for Option<T>
    where T: Clone
{
    fn update(&self, data: &mut Option<T>) {
        *data = self.clone();
    }
}

mod tests {
    use super::*;
    #[test]
    fn primitive() {
        let initial: u32 = 10;
        let mut number: u32 = initial;
        let new: u32 = 11;
        new.update(&mut number);
        assert_eq!(number, new);
    }
    #[test]
    fn vec() {
        let initial: Vec<u32> = vec![1, 2, 3];
        let mut vec = initial.clone();

        let new: Vec<u32> = vec![3, 2, 4];
        new.update(&mut vec);
        assert_eq!(vec, new);
    }
    #[derive(Clone, Updatable, Debug, PartialEq)]
    struct Inner1 {
        text: String,
    }
    #[derive(Clone, Updatable, Debug, PartialEq)]
    struct Inner2 {
        number: u32,
    }
    #[derive(Clone, Updatable, Debug, PartialEq)]
    struct Nested {
        inner1: Inner1,
        inner2: Inner2,
    }
    #[test]
    fn builder() {
        let empty =
            NestedUpdate {
                inner1: None,
                inner2: None,
            };
        let built = Nested::update();
        assert_eq!(empty, built);

        let one_set =
            NestedUpdate {
                inner1: Some(Inner1Update {
                    text: Some(String::from("Text")),
                }),
                inner2: None,
            };
        let built = Nested::update()
            .inner1(Inner1::update()
                .text(String::from("Text")));
        assert_eq!(one_set, built);
        let all_set =
            NestedUpdate {
                inner1: Some(Inner1Update {
                    text: Some(String::from("Text")),
                }),
                inner2: Some(Inner2Update {
                    number: Some(2),
                }),
            };
        let built = Nested::update()
            .inner1(Inner1::update()
                .text(String::from("Text")))
            .inner2(Inner2::update()
                .number(2))
            ;
        assert_eq!(all_set, built);
    }
    #[test]
    fn nested() {
        let initial =
            Nested {
                inner1: Inner1 {
                    text: "Hello".to_string(),
                },
                inner2: Inner2 {
                    number: 1,
                },
            };
        let mut nested = initial.clone();
        let update =
            NestedUpdate {
                inner1: Some(Inner1Update {
                    text: None,
                }),
                inner2: Some(Inner2Update {
                    number: None,
                }),
            };
        update.update(&mut nested);
        assert_eq!(nested, initial);

        let new1 =
            Nested {
                inner1: Inner1 {
                    text: "World".to_string(),
                },
                inner2: Inner2 {
                    number: 1,
                },
            };
        let update =
            NestedUpdate {
                inner1: Some(Inner1Update {
                    text: Some("World".to_string()),
                }),
                inner2: Some(Inner2Update {
                    number: None,
                }),
            };
        update.update(&mut nested);
        assert_eq!(nested, new1);

        let new2 =
            Nested {
                inner1: Inner1 {
                    text: "World".to_string(),
                },
                inner2: Inner2 {
                    number: 2,
                },
            };
        let update =
            NestedUpdate {
                inner1: Some(Inner1Update {
                    text: None,
                }),
                inner2: Some(Inner2Update {
                    number: Some(2),
                }),
            };
        update.update(&mut nested);
        assert_eq!(nested, new2);
        let new3 =
            Nested {
                inner1: Inner1 {
                    text: ".".to_string(),
                },
                inner2: Inner2 {
                    number: 3,
                },
            };
        let update =
            NestedUpdate {
                inner1: Some(Inner1Update {
                    text: Some(".".to_string()),
                }),
                inner2: Some(Inner2Update {
                    number: Some(3),
                }),
            };
        update.update(&mut nested);
        assert_eq!(nested, new3);
    }

    #[derive(Clone, Debug, PartialEq, Updatable)]
    pub struct Tuple(String, u32);
    #[test]
    fn tuple() {
        let initial = Tuple(String::from("hello"), 40);
        let mut tup = initial.clone();
        let update = TupleUpdate::builder()
            .field_0(String::from("world"));
        let new1 = Tuple(String::from("world"), 40);
        update.update(&mut tup);
        assert_eq!(tup, new1);
        let update = TupleUpdate::builder()
            .field_1(42);
        let new2 = Tuple(String::from("world"), 42);
        update.update(&mut tup);
        assert_eq!(tup, new2);
    }
}
