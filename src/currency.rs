type Value = f32;
type Units = u32;

trait Quantity {
    fn amount(&self) -> Units;
}

trait Currency : Quantity + Sized {
    fn unit_value() -> Value;
    fn value(&self) -> Value {
        Self::unit_value() * self.amount() as Value
    }
}
use std::ops::{Mul, Add};

pub struct Euro(Units);

pub fn euro() -> Euro {
    Euro::from(1)
}
impl From<Units> for Euro {
    fn from(units: Units) -> Self {
        Euro(units)
    }
}
impl Mul<Units> for Euro {
    type Output = Self;
    fn mul(self, rhs: Units) -> Self::Output {
        Euro(self.0 * rhs)
    }
}

impl Add for Euro {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Euro(self.0 + rhs.0)
    }
}

impl Quantity for Euro {
    fn amount(&self) -> Units {
        self.0
    }
}

impl Currency for Euro {
    fn unit_value() -> Value {
        1.0
    }
}

impl<A: Currency> PartialEq<A> for Euro {
    fn eq(&self, other: &A) -> bool {
        self.value() == other.value()
    }
}
