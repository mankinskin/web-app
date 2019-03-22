
use std::string::{ String, ToString };
use std::clone::Clone;

#[derive(Clone, Copy)]
struct User
{
    pub budget: Money,
}

impl User {
    pub fn new() -> Self {
        User { budget: Money::new_with(100.0) }
    }

    pub fn balance_string(&self) -> String {
        self.budget.to_string()
    }
}

type Euro = f32;

#[derive(Clone, Copy)]
struct Money
{
    amount: Euro,
}

impl Money {
    pub fn new_with(value: Euro) -> Self {
        Money { amount: value }
    }
    pub fn new() -> Self {
        Self::new_with(0.0)
    }
    pub fn transaction(&mut self, value: Euro, other: &mut Money) {
        println!("{} >\t {} \t> {}",
                 self.to_string(),
                 value.to_string(),
                 other.to_string());
        self.amount += value;
        other.amount -= value;
        println!("{}   \t \t  {}",
                 self.to_string(),
                 other.to_string());
    }
}

impl ToString for Money {
    fn to_string(&self) -> String {
        let s = self.amount.to_string();
        if self.amount > 0.0 {
            ["+", &s].join("")
        } else {
            s
        }
    }
}

const USER_COUNT: usize = 2;


fn main() {
    println!("Hello, world!");
    let mut users: Vec<User> = vec![User::new(); USER_COUNT];

    let mut user = users[0].clone();
    let other = &mut users[1];

    user.budget.transaction(-60.0, &mut other.budget);
    users[0] = user;
}
