use std::collections::{
    HashMap,
};
use daggy::{
    petgraph::algo::{
        astar,
    },
    NodeIndex,
    Dag,
};
pub type Purpose = String;

#[derive(Debug)]
pub enum GraphError {
    PurposeDoesNotExist(Purpose),
    WouldCycle,
}

pub struct PurposeGraph {
    graph: Dag<Purpose, usize>,
    purposes: HashMap<Purpose, NodeIndex>,
}
impl PurposeGraph {
    pub fn new() -> Self {
        Self{
            graph: Dag::new(),
            purposes: HashMap::new(),
        }
    }
    pub fn add_purpose<P: ToString>(&mut self, p: P) -> NodeIndex {
        let p = p.to_string();
        let id = self.graph.add_node(p.clone());
        self.purposes.insert(p, id);
        id
    }
    pub fn is_related_to<P: ToString>(&self, a: P, b: P) -> Result<bool, GraphError> {
        let a = a.to_string();
        let b = b.to_string();
        let a = self.purposes.get(&a).ok_or(GraphError::PurposeDoesNotExist(a))?;
        let b = self.purposes.get(&b).ok_or(GraphError::PurposeDoesNotExist(b))?;
        match astar(self.graph.graph(),
            *a,
            |n| n == *b,
            |_e| 0,
            |_| 0
            ) {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    pub fn link<P: ToString>(&mut self, a: P, b: P) -> Result<(), GraphError> {
        let a = a.to_string();
        let b = b.to_string();
        let a = self.purposes.get(&a).ok_or(GraphError::PurposeDoesNotExist(a))?;
        let b = self.purposes.get(&b).ok_or(GraphError::PurposeDoesNotExist(b))?;
        self.graph.add_edge(*a, *b, 0).map_err(|_e| GraphError::WouldCycle).map(|_| ())
    }
}



mod tests {
    #[test]
    fn relations() {
        use super::{
            PurposeGraph,
        };
        let mut pg = PurposeGraph::new();
        pg.add_purpose("Käse");
        pg.add_purpose("Brot");
        pg.add_purpose("Essen");
        pg.add_purpose("Gesundheit");
        assert!(!pg.is_related_to("Käse", "Brot").unwrap());
        assert!(!pg.is_related_to("Käse", "Essen").unwrap());
        assert!(!pg.is_related_to("Brot", "Käse").unwrap());
        assert!(!pg.is_related_to("Brot", "Essen").unwrap());
        pg.link("Käse", "Essen").unwrap();
        assert!(!pg.is_related_to("Käse", "Brot").unwrap());
        assert!(pg.is_related_to("Käse", "Essen").unwrap());
        assert!(!pg.is_related_to("Brot", "Käse").unwrap());
        assert!(!pg.is_related_to("Brot", "Essen").unwrap());
        pg.link("Brot", "Essen").unwrap();
        assert!(!pg.is_related_to("Käse", "Brot").unwrap());
        assert!(pg.is_related_to("Käse", "Essen").unwrap());
        assert!(!pg.is_related_to("Brot", "Käse").unwrap());
        assert!(pg.is_related_to("Brot", "Essen").unwrap());
        pg.link("Essen", "Gesundheit").unwrap();
        assert!(pg.is_related_to("Käse", "Gesundheit").unwrap());
        assert!(pg.is_related_to("Brot", "Gesundheit").unwrap());
        assert!(pg.is_related_to("Essen", "Gesundheit").unwrap());
        assert!(!pg.is_related_to("Gesundheit", "Essen").unwrap());
    }
}
