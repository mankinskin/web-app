pub mod merge_direction;
pub mod split_minimizer;
pub use {
    merge_direction::*,
    split_minimizer::*,
};

#[cfg(test)]
mod tests {
    use crate::{
        r#match::*,
        split::*,
        vertex::*,
        *,
    };
    use maplit::hashset;
    use pretty_assertions::assert_eq;
    use std::{
        collections::HashSet,
        num::NonZeroUsize,
    };
    #[test]
    fn merge_single_split_1() {
        let mut graph = Hypergraph::default();
        if let [a, b, c, d] = graph.insert_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('c'),
            Token::Element('d'),
        ])[..]
        {
            // wxabyzabbyxabyz
            let ab = graph.insert_pattern([a, b]);
            let bc = graph.insert_pattern([b, c]);
            let cd = graph.insert_pattern([c, d]);
            let abc = graph.insert_patterns([vec![ab, c], vec![a, bc]]);
            let bcd = graph.insert_patterns([vec![bc, d], vec![b, cd]]);
            let _abcd = graph.insert_patterns([vec![abc, d], vec![a, bcd]]);
            let left = vec![
                (vec![a], SplitSegment::Child(b)),
                (vec![], SplitSegment::Child(ab)),
            ];
            let right = vec![
                (vec![d], SplitSegment::Child(c)),
                (vec![], SplitSegment::Child(cd)),
            ];
            let mut minimizer = SplitMinimizer::new(&mut graph);
            let left = minimizer.merge_left_splits(left);
            let right = minimizer.merge_right_splits(right);
            assert_eq!(left, SplitSegment::Child(ab), "left");
            assert_eq!(right, SplitSegment::Child(cd), "right");
        } else {
            panic!();
        }
    }
    #[test]
    fn merge_split_2() {
        let mut graph = Hypergraph::default();
        if let [a, b, _w, x, y, z] = graph.insert_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('w'),
            Token::Element('x'),
            Token::Element('y'),
            Token::Element('z'),
        ])[..]
        {
            // wxabyzabbyxabyz
            let ab = graph.insert_pattern([a, b]);
            let by = graph.insert_pattern([b, y]);
            let yz = graph.insert_pattern([y, z]);
            let xab = graph.insert_pattern([x, ab]);
            let xaby = graph.insert_patterns([vec![xab, y], vec![x, a, by]]);
            let xabyz = graph.insert_patterns([vec![xaby, z], vec![xab, yz]]);

            let mut splitter = IndexSplitter::new(&mut graph);
            let (ps, child_splits) =
                splitter.separate_perfect_split(xabyz, NonZeroUsize::new(2).unwrap());
            assert_eq!(ps, None);
            let (left, right) = splitter.build_child_splits(child_splits);

            let xa_found = graph.find_pattern(vec![x, a]);
            let xa = if let SearchFound {
                index: xa,
                parent_match:
                    ParentMatch {
                        parent_range: FoundRange::Complete,
                        ..
                    },
                ..
            } = xa_found.expect("xa not found")
            {
                Some(xa)
            } else {
                None
            }
            .expect("xa");

            let expleft = hashset![(vec![], SplitSegment::Child(xa)),];
            let expright = hashset![
                (vec![yz], SplitSegment::Child(b)),
                (vec![z], SplitSegment::Child(by)),
            ];

            let (sleft, sright): (HashSet<_>, HashSet<_>) = (
                left.clone().into_iter().collect(),
                right.clone().into_iter().collect(),
            );
            assert_eq!(sleft, expleft, "left");
            assert_eq!(sright, expright, "right");

            let mut minimizer = SplitMinimizer::new(&mut graph);
            let left = minimizer.merge_left_splits(left);
            let right = minimizer.merge_right_splits(right);
            println!("{:#?}", graph);
            println!("left = {:#?}", left);
            println!("right = {:#?}", right);
            let byz_found = graph.find_pattern(vec![b, y, z]);
            let byz = if let SearchFound {
                index: byz,
                parent_match:
                    ParentMatch {
                        parent_range: FoundRange::Complete,
                        ..
                    },
                ..
            } = byz_found.expect("byz not found")
            {
                Some(byz)
            } else {
                None
            }
            .expect("byz");
            assert_eq!(left, SplitSegment::Child(xa), "left");
            assert_eq!(right, SplitSegment::Child(byz), "left");
        } else {
            panic!();
        }
    }
}
