#![feature(test)]
#![feature(async_closure)]
#![feature(assert_matches)]

extern crate test;

pub mod arithmetic_bool;
pub mod graph;
//pub mod mapping;
//pub mod node;
pub mod token;
//pub mod grammar;
pub mod hypergraph;

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use test::Bencher;
//    lazy_static::lazy_static! {
//        pub static ref SEQS: Vec<&'static str> = Vec::from([
//            "",
//            "bc",
//            "aa",
//            "abc",
//            "bcade",
//            "bcaade",
//            "bcbcabc",
//            "abcaa",
//            "abcaabcbcabcbcade",
//        ]);
//    }
//    #[bench]
//    fn bench_read_sequence(b: &mut Bencher) {
//        b.iter(|| {
//            let mut g = SequenceGraph::<char>::new();
//            for &s in SEQS.iter() {
//                g.read_sequence(s.chars());
//            }
//        })
//    }
//    #[bench]
//    fn bench_read_from(b: &mut Bencher) {
//        b.iter(|| {
//            let mut g = SequenceGraph::<char>::new();
//            for &s in SEQS.iter() {
//                g.read_from(s.chars());
//            }
//        })
//    }
//}
