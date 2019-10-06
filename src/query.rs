use crate::transaction::*;
use crate::currency::*;
use crate::person::*;
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
    pub fn with_partner<P: Into<Person> + Clone>(self, partner: P) -> Self {
        self.filter(move |t|
                    if let Some(p) = &t.partner {
                        p.clone() == partner.clone().into()
                    } else { false })
    }
    pub fn with_any_partners<P: Into<Person> + Clone>(self, parts: Vec<P>) -> Self {
        let parts: Vec<Person> = parts.iter().map(|p| p.clone().into()).collect();
        self.filter(move |t|
                    if let Some(p) = &t.partner {
                        parts.contains(p)
                    } else { false })
    }
    pub fn with_purpose<P: Into<Purpose> + Clone>(self, purp: P) -> Self {
        self.filter(move |t|
                    t.purposes
                    .clone()
                    .map(|ps| ps.contains(&purp.clone().into())).unwrap_or(false))
    }
    pub fn with_any_purposes<P: Into<Purpose> + Clone>(self, purps: Vec<P>) -> Self {
        let purps: Vec<Purpose> = purps.iter().map(|p| p.clone().into()).collect();
        self.filter(|t|
                    t.purposes
                    .clone()
                    .map(|ps| ps.iter()
                         .map(|p| purps.contains(&p))
                         .fold(false,
                               |acc, x| acc || x))
                    .unwrap_or(false))
    }
    pub fn with_all_purposes<P: Into<Purpose> + Clone>(self, purps: Vec<P>) -> Self {
        let purps: Vec<Purpose> = purps.iter().map(|p| p.clone().into()).collect();
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
        self.filter(|t| t.date
                         .map(|d| d < time)
                         .unwrap_or(false))
    }
    pub fn after(self,
                 time: DateTime<Utc>) -> Self {
        self.filter(|t| t.date
                         .map(|d| d >= time)
                         .unwrap_or(false))
    }
    pub fn within_timespan(self,
                           start: DateTime<Utc>,
                           end: DateTime<Utc>) -> Self {
        self.filter(|t| t.date
                         .map(|d| d >= start && d <= end)
                         .unwrap_or(false))
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
        assert!(budget.find().with_partner("Papa").len() == 0);
        assert!(budget.find().with_purpose("Fahrstunde").len() == 0);
        assert!(budget.find().with_purpose("Arbeit").len() == 0);

        budget.get(Euro(19)).set_partner("Papa");
        assert!(budget.balance == Euro(140+19));
        assert!(budget.find().earnings().len() == 1);
        assert!(budget.find().expenses().len() == 0);
        assert!(budget.find().with_partner("Papa").len() == 1);
        assert!(budget.find().with_purpose("Fahrstunde").len() == 0);
        assert!(budget.find().with_purpose("Arbeit").len() == 0);
        assert!(budget.find().with_any_partners(vec!["Papa", "Schölermann"]).len() == 1);
        assert!(budget.find().with_any_partners(vec!["Jonas", "Leon", "Schölermann"]).len() == 0);

        budget.give(Euro(49)).set_purpose("Fahrstunde").set_partner("Schölermann");
        assert!(budget.balance == Euro((140+19)-49));
        assert!(budget.find().earnings().len() == 1);
        assert!(budget.find().expenses().len() == 1);
        assert!(budget.find().with_partner("Papa").len() == 1);
        assert!(budget.find().with_purpose("Fahrstunde").len() == 1);
        assert!(budget.find().with_purpose("Arbeit").len() == 0);
        assert!(budget.find().with_any_partners(vec!["Papa", "Schölermann"]).len() == 2);
        assert!(budget.find().with_any_partners(vec!["Jonas", "Leon", "Schölermann"]).len() == 1);

        budget.get(Euro(72)).set_purposes(vec!["Arbeit", "Programmieren"]);
        assert!(budget.balance == Euro(((140+19)-49)+72));
        assert!(budget.find().earnings().len() == 2);
        assert!(budget.find().expenses().len() == 1);
        assert!(budget.find().with_partner("Papa").len() == 1);
        assert!(budget.find().with_purpose("Fahrstunde").len() == 1);
        assert!(budget.find().with_purpose("Arbeit").len() == 1);
        assert!(budget.find().with_any_purposes(vec!["Programmieren", "Essen"]).len() == 1);
        assert!(budget.find().with_all_purposes(vec!["Programmieren", "Essen"]).len() == 0);
        assert!(budget.find().with_any_partners(vec!["Papa", "Schölermann"]).len() == 2);
        assert!(budget.find().with_any_partners(vec!["Jonas", "Leon", "Schölermann"]).len() == 1);

        budget.give(Euro(19))
              .set_purposes(vec!["Programmieren", "Essen"])
              .set_partner("Jonas");
        assert!(budget.balance == Euro((((140+19)-49)+72)-19));
        assert!(budget.find().earnings().len() == 2);
        assert!(budget.find().expenses().len() == 2);
        assert!(budget.find().with_partner("Papa").len() == 1);
        assert!(budget.find().with_purpose("Arbeit").len() == 1);
        assert!(budget.find().with_any_purposes(vec!["Programmieren", "Essen"]).len() == 2);
        assert!(budget.find().with_all_purposes(vec!["Programmieren", "Essen"]).len() == 1);
        assert!(budget.find().with_any_partners(vec!["Papa", "Schölermann"]).len() == 2);
        assert!(budget.find().with_any_partners(vec!["Jonas", "Leon", "Schölermann"]).len() == 2);

        budget
    }
    #[test]
    fn find_partner() {
        create_test_budget();
    }
}
