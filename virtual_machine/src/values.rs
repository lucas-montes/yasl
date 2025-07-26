#[derive(Default, Debug, PartialEq, PartialOrd)]
pub struct Value(f64);

impl std::ops::Add for Value {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Value(self.0 + other.0)
    }
}

impl std::ops::Neg for Value {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Value(-self.0)
    }

}

impl std::ops::Sub for Value {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Value(self.0 - other.0)
    }
}

impl std::ops::Mul for Value {
    type Output = Self;

    fn mul(self, other: Self) -> Self::Output {
        Value(self.0 * other.0)
    }
}
impl std::ops::Div for Value {
    type Output = Self;

    fn div(self, other: Self) -> Self::Output {
        if other.0 == 0.0 {
            panic!("Division by zero");
        }
        Value(self.0 / other.0)
    }
}
