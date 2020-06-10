use std::collections::{
    *,
    HashSet,
    HashMap,
};


mod parse;

/// The globally unique identifier of a set
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SetId(usize);

/// Operations on sets that evaluate to new sets
#[derive(Debug, Clone, PartialEq, Eq)]
enum SetExpression {
    Set(Set),
    Join(Box<(SetExpression, SetExpression)>),
    Intersect(Box<(SetExpression, SetExpression)>),
    Complement(Box<SetExpression>),
    Subtract(Box<(SetExpression, SetExpression)>),
    Difference(Box<(SetExpression, SetExpression)>),
}
/// Operations on sets that evaluate to a boolean
#[derive(Debug, Clone, PartialEq, Eq)]
enum SetPredicate {
    Subset(Box<(SetExpression, SetExpression)>),
    SubsetEq(Box<(SetExpression, SetExpression)>),
    Equal(Box<(SetExpression, SetExpression)>),
}
/// A single set
#[derive(Debug, Clone, PartialEq, Eq)]
struct Set {
    id: SetId,
    attributes: Vec<Attribute>,
}
impl Set {
    pub fn new(id: SetId) -> Self {
        Set {
            id,
            attributes: Vec::new(),
        }
    }
}
/// An attribute of a set
#[derive(Debug, Clone, PartialEq, Eq)]
struct Attribute {
    name: String,
    set: SetId,
}
/// provides the context for all sets and predicates
#[derive(Debug)]
struct Universe {
    sets: HashMap<SetId, Set>,
    predicates: Vec<SetPredicate>
}
impl Universe {
    /// creates an empty universe
    pub fn empty() -> Self {
        Self {
            sets: HashMap::new(),
            predicates: Vec::new(),
        }
    }
    /// checks whether self defines a set
    pub fn defines_set(&self, set: &SetId) -> bool {
        self.sets.contains_key(&set)
    }
    /// ensures no set is defined multiple times
    pub fn add_set(&mut self, set: SetId) {
        if !self.defines_set(&set) {
            self.sets.insert(set.clone(), Set::new(set));
        }
    }
    /// checks whether self contains a Predicate
    pub fn predicates(&self, predicate: &SetPredicate) -> bool {
        self.predicates.contains(&predicate)
    }
    /// ensures no predicates are duplicated
    pub fn add_predicate(&mut self, predicate: SetPredicate) {
        if !self.predicates(&predicate) {
            self.predicates.push(predicate);
        }
    }
}
