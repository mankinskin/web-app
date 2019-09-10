use crate::transaction::*;
use crate::currency::*;
use crate::actor::{
    Actor,
};
use crate::purpose::{
    Purpose,
};
use ::chrono::{
    DateTime,
    Utc,
};
pub struct Query<'a, C: Currency>(pub Vec<&'a Transaction<C>>);

impl<'a, C: Currency> Query<'a, C> {

    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn filter<P: Fn(&'a Transaction<C>) -> bool>(
        self,
        predicate: P
        ) -> Self {
        Self(
            self.0
            .iter().filter_map(
                move |&t| if predicate(t) {
                    Some(t)
                } else {
                    None
                })
            .collect()
            )
    }
    pub fn with_partner(self, a: Actor) -> Self {
        self.filter(|t|
                    if let Some(p) = &t.partner {
                        p.clone() == a
                    } else { false })
    }
    pub fn with_any_partners(self, parts: Vec<Actor>) -> Self {
        self.filter(|t|
                    if let Some(p) = &t.partner {
                        parts.contains(p)
                    } else { false })
    }
    pub fn with_purpose(self, purp: Purpose) -> Self {
        self.filter(|t|
                    t.purposes
                    .clone()
                    .map(|ps| ps.contains(&purp)).unwrap_or(false))
    }
    pub fn with_any_purposes(self, purps: Vec<Purpose>) -> Self {
        self.filter(|t|
                    t.purposes
                    .clone()
                    .map(|ps| ps.iter()
                         .map(|p| purps.contains(&p))
                         .fold(false,
                               |acc, x| acc || x))
                    .unwrap_or(false))
    }
    pub fn with_all_purposes(self, purps: Vec<Purpose>) -> Self {
        self.filter(|t|
                    t.purposes
                    .clone()
                    .map(|ps| ps.iter()
                         .map(|p| purps.contains(&p))
                         .fold(true,
                               |acc, x| acc && x))
                    .unwrap_or(false))
    }
    pub fn with_max(self, max: C) -> Self {
        self.filter(|t| t.amount <= max)
    }
    pub fn with_min(self, min: C) -> Self {
        self.filter(|t| t.amount >= min)
    }
    pub fn earnings(self) -> Self {
        self.with_min(C::from(0))
    }
    pub fn expenses(self) -> Self {
        self.with_max(C::from(0))
    }
    pub fn before(self,
                  time: DateTime<Utc>) -> Self {
        self.filter(|t| t.date < time)
    }
    pub fn after(self,
                 time: DateTime<Utc>) -> Self {
        self.filter(|t| t.date >= time)
    }
    pub fn within_timespan(self,
                           start: DateTime<Utc>,
                           end: DateTime<Utc>) -> Self {
        self.filter(|t| t.date >= start && t.date <= end)
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
        assert!(budget.find().earnings().len() == 0);
        assert!(budget.find().expenses().len() == 0);
        assert!(budget.find().with_partner("Papa".into()).len() == 0);
        assert!(budget.find().with_purpose("Fahrstunde".into()).len() == 0);
        assert!(budget.find().with_purpose("Arbeit".into()).len() == 0);

        budget.get(Euro(19)).set_partner("Papa".into());
        assert!(budget.balance == Euro(140+19));
        assert!(budget.find().earnings().len() == 1);
        assert!(budget.find().expenses().len() == 0);
        assert!(budget.find().with_partner("Papa".into()).len() == 1);
        assert!(budget.find().with_purpose("Fahrstunde".into()).len() == 0);
        assert!(budget.find().with_purpose("Arbeit".into()).len() == 0);
        assert!(budget.find().with_any_partners(vec!["Papa".into(), "Schölermann".into()]).len() == 1);
        assert!(budget.find().with_any_partners(vec!["Jonas".into(), "Leon".into(), "Schölermann".into()]).len() == 0);

        budget.give(Euro(49)).set_purpose("Fahrstunde".into()).set_partner("Schölermann".into());
        assert!(budget.balance == Euro((140+19)-49));
        assert!(budget.find().earnings().len() == 1);
        assert!(budget.find().expenses().len() == 1);
        assert!(budget.find().with_partner("Papa".into()).len() == 1);
        assert!(budget.find().with_purpose("Fahrstunde".into()).len() == 1);
        assert!(budget.find().with_purpose("Arbeit".into()).len() == 0);
        assert!(budget.find().with_any_partners(vec!["Papa".into(), "Schölermann".into()]).len() == 2);
        assert!(budget.find().with_any_partners(vec!["Jonas".into(), "Leon".into(), "Schölermann".into()]).len() == 1);

        budget.get(Euro(72)).set_purposes(vec!["Arbeit".into(), "Programmieren".into()]);
        assert!(budget.balance == Euro(((140+19)-49)+72));
        assert!(budget.find().earnings().len() == 2);
        assert!(budget.find().expenses().len() == 1);
        assert!(budget.find().with_partner("Papa".into()).len() == 1);
        assert!(budget.find().with_purpose("Fahrstunde".into()).len() == 1);
        assert!(budget.find().with_purpose("Arbeit".into()).len() == 1);
        assert!(budget.find().with_any_purposes(vec!["Programmieren".into(), "Essen".into()]).len() == 1);
        assert!(budget.find().with_all_purposes(vec!["Programmieren".into(), "Essen".into()]).len() == 0);
        assert!(budget.find().with_any_partners(vec!["Papa".into(), "Schölermann".into()]).len() == 2);
        assert!(budget.find().with_any_partners(vec!["Jonas".into(), "Leon".into(), "Schölermann".into()]).len() == 1);

        budget.give(Euro(19))
              .set_purposes(vec!["Programmieren".into(), "Essen".into()])
              .set_partner("Jonas".into());
        assert!(budget.balance == Euro((((140+19)-49)+72)-19));
        assert!(budget.find().earnings().len() == 2);
        assert!(budget.find().expenses().len() == 2);
        assert!(budget.find().with_partner("Papa".into()).len() == 1);
        assert!(budget.find().with_purpose("Arbeit".into()).len() == 1);
        assert!(budget.find().with_any_purposes(vec!["Programmieren".into(), "Essen".into()]).len() == 2);
        assert!(budget.find().with_all_purposes(vec!["Programmieren".into(), "Essen".into()]).len() == 1);
        assert!(budget.find().with_any_partners(vec!["Papa".into(), "Schölermann".into()]).len() == 2);
        assert!(budget.find().with_any_partners(vec!["Jonas".into(), "Leon".into(), "Schölermann".into()]).len() == 2);

        budget
    }
    #[test]
    fn find_partner() {
        create_test_budget();
    }
}
