use crate::currency::{
    Euro,
    Currency,
};
use crate::transaction::{
    Transaction,
};
use crate::actor::{
    Actor,
};
use crate::purpose::{
    Purpose,
    PurposeGraph,
};
pub struct Budget<C: Currency> {
    name: String,
    balance: C,
    transactions: Vec<Transaction<C>>,
    purposes: PurposeGraph,
}

impl<C: Currency> Budget<C> {
    pub fn create(name: &str, balance: C) -> Budget<C> {
        Budget::<C> {
            name: name.into(),
            balance: balance.clone(),
            transactions: Vec::new(),
            purposes: PurposeGraph::new(),
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn get(&mut self, amount: C) -> &mut Transaction<C> {
        let t = Transaction::get(amount.clone());
        self.balance += amount;
        self.transactions.push(t);
        self.transactions.iter_mut().last().unwrap()
    }
    pub fn give(&mut self, amount: C) -> &mut Transaction<C> {
        let t = Transaction::give(amount.clone());
        self.balance -= amount;
        self.transactions.push(t);
        self.transactions.iter_mut().last().unwrap()
    }
    pub fn find_with_partner(&self, a: Actor) -> Vec<&Transaction<C>> {
        self.transactions
            .iter()
            .filter_map(move |t| if t.partner == Some(a.clone()) {
                Some(t)
            } else {
                None
            }).collect()
    }
    pub fn find_with_purpose(&self, purp: Purpose) -> Vec<&Transaction<C>> {
        self.transactions
            .iter()
            .filter_map(move |t|
            if t.purposes.clone().map(|ps| ps.contains(&purp)).unwrap_or(false) {
                Some(t)
            } else {
                None
            }).collect()
    }
    pub fn find_with_max(&self, max: C) -> Vec<&Transaction<C>> {
        self.transactions
            .iter()
            .filter_map(move |t|
                if t.amount <= max {
                    Some(t)
                } else {
                    None
                }).collect()
    }
    pub fn find_with_min(&self, min: C) -> Vec<&Transaction<C>> {
        self.transactions
            .iter()
            .filter_map(move |t|
                if t.amount >= min {
                    Some(t)
                } else {
                    None
                }).collect()
    }
    pub fn find_earnings(&self) -> Vec<&Transaction<C>> {
        self.find_with_min(C::from(0))
    }
    pub fn find_expenses(&self) -> Vec<&Transaction<C>> {
        self.find_with_max(C::from(0))
    }
}

impl From<Budget<Euro>> for Euro {
    fn from(budget: Budget<Euro>) -> Euro {
        budget.balance
    }
}

use tabular::{table, row};
use std::fmt;
impl<C: Currency> fmt::Display for Budget<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut table =
            table!("{:<}\t|\t{:>}|{:<}\t|{:<}",
                    row!("Date", "Amount", "Partner", "Purposes"));
        for t in &self.transactions {
            table.add_row((*t).clone().into());
        }
        write!(f, "{}\n{}",
            table!("{:<}\t\t{:<}: {:>}",
                   row!(self.name.clone(), "Balance", self.balance.clone())),
               table)
    }
}
