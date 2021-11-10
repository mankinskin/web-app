#![feature(test)]
use std::{
    fmt::{
        Debug,
    },
    ops::{
        Deref,
        DerefMut,
    },
    collections::{
        HashMap,
        HashSet,
    },
    hash::{
        Hash,
    },
    default::{
        Default,
    },
};

trait Alphabet : Hash + Sized + Default + PartialEq + Eq + Clone {
    fn to_symbol(self) -> Symbol<Self>;
}
trait Text<A: Alphabet> : Iterator<Item=A> + Clone { }
impl<A: Alphabet, I: Iterator<Item=A> + Clone> Text<A> for I {}

#[derive(Hash)]
struct Word<A: Alphabet> {
    letters: Vec<A>,
}
impl<A: Alphabet> Word<A> {
    pub fn intersects(&self, text: impl Text<A>) -> bool {
        text.any(|c| self.letters.iter().find(|l| **l == c).is_some())
    }
    /// consume common prefix
    pub fn common_prefix_length(&self, text: impl Text<A>) -> usize {
        let letters = self.letters.iter();
        text.zip(letters).position(|(c, l)| *l != c)
    }
    /// decide whether text starts with self
    pub fn prefix_of(&self, text: impl Text<A>) -> bool {
    }
}
#[derive(Hash)]
struct Rule<A: Alphabet> {
    symbols: Vec<Symbol<A>>
}
impl<A: Alphabet> Rule<A> {
    pub fn intersects(&self, text: impl Text<A>) -> bool {
        for symbol in &self.symbols {
            symbol.intersects(text.clone());
        }
        false
    }
    pub fn common_prefix_length(&self, text: impl Text<A>) -> usize {
        let mut text: Vec<A> = text.collect();
        for symbol in &self.symbols {
            if symbol.prefix_of(text.iter().cloned()) {

            }
        }
        false
    }
    pub fn prefix_of(&self, text: impl Text<A>) -> bool {
        let mut text: Vec<A> = text.collect();
        for symbol in &self.symbols {
            if symbol.prefix_of(text.iter().cloned()) {

            }
        }
        false
    }
}
impl<A: Alphabet> From<Vec<Symbol<A>>> for Rule<A> {
    fn from(symbols: Vec<Symbol<A>>) -> Self {
        Self {
            symbols,
        }
    }
}
#[derive(Hash)]
enum Symbol<A: Alphabet> {
    Word(Word<A>),
    Rule(Rule<A>),
}
impl<A: Alphabet> Symbol<A> {
    pub fn intersects(&self, text: impl Text<A>) -> bool {
        match self {
            Self::Word(word) => {
                word.intersects(text.clone())
            },
            Self::Rule(rule) => {
                rule.intersects(text.clone())
            },
        }
    }
    pub fn prefix_of(&self, text: impl Text<A>) -> bool {
        match self {
            Self::Word(word) => {
                word.prefix_of(text.clone())
            },
            Self::Rule(rule) => {
                rule.prefix_of(text.clone())
            },
        }
    }
}
type RuleID = usize;
pub trait AdaptiveGrammar<A: Alphabet> {
    fn add_rule(&mut self, rule: Rule<A>) -> RuleID;
    fn delete_rule(&mut self, id: &RuleID) -> Option<Rule<A>>;
    fn get_rule_mut(&mut self, id: &RuleID) -> Option<&mut Rule<A>>;
    fn get_rule(&self, id: &RuleID) -> Option<&Rule<A>>;
}
#[derive(Default)]
struct Grammar<A: Alphabet> {
    // A: AlphaMap
    // Rule<A>: Non-Terminal Symbols
    rules: HashMap<RuleID, Rule<A>>,
    id_counter: usize,
}
impl<A: Alphabet> AdaptiveGrammar<A> for Grammar<A> {
    fn add_rule(&mut self, rule: Rule<A>) -> RuleID {
        let id = self.id_counter;
        self.id_counter += 1;
        self.rules.insert(id, rule);
        id
    }
    fn delete_rule(&mut self, id: &RuleID) -> Option<Rule<A>> {
        self.rules.remove(id)
    }
    fn get_rule_mut(&mut self, id: &RuleID) -> Option<&mut Rule<A>> {
        self.rules.get_mut(id)
    }
    fn get_rule(&self, id: &RuleID) -> Option<&Rule<A>> {
        self.rules.get(id)
    }
}
impl<'g, A: Alphabet> Grammar<A> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn learn(&mut self, text: impl Text<A>) {
        let symbols: Vec<Symbol<A>> = text.clone().map(Alphabet::to_symbol).collect();
        for (id, rule) in &self.rules {
            if rule.matches(text.clone(), &self) {
                return;
            }
            if rule.intersects(text.clone(), &self) {
                // create new rules for intersections
                // a rule may not contain two consequtive words

            }
        }
        let new_rule = Rule::from(symbols);
        self.add_rule(new_rule);
    }
    pub fn knows(&self, text: impl Text<A>) -> bool {
        false
    }
}
