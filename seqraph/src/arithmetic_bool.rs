use std::{
    default::Default,
    fmt::{
        self,
        Debug,
        Display,
        Formatter,
    },
    ops::{
        Add,
        AddAssign,
        Deref,
        DerefMut,
        Mul,
        MulAssign,
    },
};

#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Default, Debug)]
pub struct ArithmeticBool(pub bool);

impl Display for ArithmeticBool {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self.0 {
                true => "1",
                false => "0",
            }
        )
    }
}
impl num_traits::Zero for ArithmeticBool {
    fn zero() -> Self {
        Self(false)
    }
    fn is_zero(&self) -> bool {
        !self.0
    }
}
impl num_traits::One for ArithmeticBool {
    fn one() -> Self {
        Self(true)
    }
    fn is_one(&self) -> bool {
        self.0
    }
}
impl From<bool> for ArithmeticBool {
    fn from(b: bool) -> Self {
        Self(b)
    }
}
impl Deref for ArithmeticBool {
    type Target = bool;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for ArithmeticBool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Mul for ArithmeticBool {
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        Self::from(self.0 && other.0)
    }
}
impl Add for ArithmeticBool {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Self::from(self.0 || other.0)
    }
}
impl AddAssign for ArithmeticBool {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}
impl MulAssign for ArithmeticBool {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}
