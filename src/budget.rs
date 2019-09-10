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
use ::chrono::{
    DateTime,
    Utc,
};
use crate::query::*;

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
    pub fn execute_transaction(&mut self, t: Transaction<C>) -> &mut Transaction<C> {
        self.balance += t.amount.clone();
        self.transactions.push(t);
        self.transactions.iter_mut().last().expect("Failed to push transaction!")
    }
    pub fn get(&mut self, amount: C) -> &mut Transaction<C> {
        self.execute_transaction(Transaction::get(amount.clone()))
    }
    pub fn give(&mut self, amount: C) -> &mut Transaction<C> {
        self.execute_transaction(Transaction::give(amount.clone()))
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

#[cfg(test)]
mod tests {
    use crate::currency::{
        Euro,
    };
    use crate::budget::{
        Budget,
    };
    fn create_test_budget() -> Budget<Euro> {
        let mut budget = Budget::create("TestBudget", Euro(140));
        assert!(budget.balance == Euro(140));
        assert!(budget.find_earnings().len() == 0);
        assert!(budget.find_expenses().len() == 0);
        assert!(budget.find_with_partner("Papa".into()).len() == 0);
        assert!(budget.find_with_purpose("Fahrstunde".into()).len() == 0);
        assert!(budget.find_with_purpose("Arbeit".into()).len() == 0);

        budget.get(Euro(19)).set_partner("Papa".into());
        assert!(budget.balance == Euro(140+19));
        assert!(budget.find_earnings().len() == 1);
        assert!(budget.find_expenses().len() == 0);
        assert!(budget.find_with_partner("Papa".into()).len() == 1);
        assert!(budget.find_with_purpose("Fahrstunde".into()).len() == 0);
        assert!(budget.find_with_purpose("Arbeit".into()).len() == 0);
        assert!(budget.find_with_any_partners(vec!["Papa".into(), "Schölermann".into()]).len() == 1);
        assert!(budget.find_with_any_partners(vec!["Jonas".into(), "Leon".into(), "Schölermann".into()]).len() == 0);

        budget.give(Euro(49)).set_purpose("Fahrstunde".into()).set_partner("Schölermann".into());
        assert!(budget.balance == Euro((140+19)-49));
        assert!(budget.find_earnings().len() == 1);
        assert!(budget.find_expenses().len() == 1);
        assert!(budget.find_with_partner("Papa".into()).len() == 1);
        assert!(budget.find_with_purpose("Fahrstunde".into()).len() == 1);
        assert!(budget.find_with_purpose("Arbeit".into()).len() == 0);
        assert!(budget.find_with_any_partners(vec!["Papa".into(), "Schölermann".into()]).len() == 2);
        assert!(budget.find_with_any_partners(vec!["Jonas".into(), "Leon".into(), "Schölermann".into()]).len() == 1);

        budget.get(Euro(72)).set_purposes(vec!["Arbeit".into(), "Programmieren".into()]);
        assert!(budget.balance == Euro(((140+19)-49)+72));
        assert!(budget.find_earnings().len() == 2);
        assert!(budget.find_expenses().len() == 1);
        assert!(budget.find_with_partner("Papa".into()).len() == 1);
        assert!(budget.find_with_purpose("Fahrstunde".into()).len() == 1);
        assert!(budget.find_with_purpose("Arbeit".into()).len() == 1);
        assert!(budget.find_with_any_purposes(vec!["Programmieren".into(), "Essen".into()]).len() == 1);
        assert!(budget.find_with_all_purposes(vec!["Programmieren".into(), "Essen".into()]).len() == 0);
        assert!(budget.find_with_any_partners(vec!["Papa".into(), "Schölermann".into()]).len() == 2);
        assert!(budget.find_with_any_partners(vec!["Jonas".into(), "Leon".into(), "Schölermann".into()]).len() == 1);

        budget.give(Euro(19))
              .set_purposes(vec!["Programmieren".into(), "Essen".into()])
              .set_partner("Jonas".into());
        assert!(budget.balance == Euro((((140+19)-49)+72)-19));
        assert!(budget.find_earnings().len() == 2);
        assert!(budget.find_expenses().len() == 2);
        assert!(budget.find_with_partner("Papa".into()).len() == 1);
        assert!(budget.find_with_purpose("Arbeit".into()).len() == 1);
        assert!(budget.find_with_any_purposes(vec!["Programmieren".into(), "Essen".into()]).len() == 2);
        assert!(budget.find_with_all_purposes(vec!["Programmieren".into(), "Essen".into()]).len() == 1);
        assert!(budget.find_with_any_partners(vec!["Papa".into(), "Schölermann".into()]).len() == 2);
        assert!(budget.find_with_any_partners(vec!["Jonas".into(), "Leon".into(), "Schölermann".into()]).len() == 2);

        budget
    }
    #[test]
    fn find_partner() {
        create_test_budget();
    }
}
