#![allow(unused)]
use crate::currency::{
    Euro,
    Currency,
};
use crate::transaction::{
    Transaction,
};
use crate::person::{
    Person,
};
use crate::purpose::{
    Purpose,
    PurposeGraph,
};
use ::chrono::{
    DateTime,
    Utc,
};
use crate::query::*;

#[allow(unused)]
#[derive(Clone)]
pub struct Budget<C: Currency> {
    pub name: String,
    pub balance: C,
    pub transactions: Vec<Transaction<C>>,
    //purposes: PurposeGraph,
}

impl<C: Currency> Budget<C> {
    pub fn create<N: Into<String>, Amt: Into<C>>(name: N, balance: Amt) -> Budget<C> {
        Budget::<C> {
            name: name.into(),
            balance: balance.into(),
            transactions: Vec::new(),
            //purposes: PurposeGraph::new(),
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn execute_transaction(&mut self, t: Transaction<C>) -> &mut Transaction<C> {
        self.balance += t.amount.clone();
        self.transactions.push(t);
        self.transactions.iter_mut().last().expect("Failed to push transaction!")
    }
    pub fn get<Amt: Into<C>>(&mut self, amount: Amt) -> &mut Transaction<C> {
        self.execute_transaction(Transaction::get(amount.into()))
    }
    pub fn give<Amt: Into<C>>(&mut self, amount: Amt) -> &mut Transaction<C> {
        self.execute_transaction(Transaction::give(amount.into()))
    }
    pub fn find<'a>(&'a self) -> Query<'a, C> {
        Query(self.transactions
            .iter().map(|t| t)
            .collect()
            )
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
