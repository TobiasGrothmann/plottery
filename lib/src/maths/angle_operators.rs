use std::ops;

use super::angle::Angle;
use crate::ANGLE_TOLERANCE;

/// `$op` between angles, owned or borrowed, and its assign form.
macro_rules! angle_op {
    ($Op:ident, $op:ident, $OpAssign:ident, $op_assign:ident, $sym:tt) => {
        impl ops::$Op<Angle> for Angle {
            type Output = Angle;
            fn $op(self, rhs: Angle) -> Angle {
                Angle::from_rad(self.to_rad() $sym rhs.to_rad())
            }
        }
        impl ops::$Op<&Angle> for Angle {
            type Output = Angle;
            fn $op(self, rhs: &Angle) -> Angle {
                self $sym *rhs
            }
        }
        impl ops::$Op<Angle> for &Angle {
            type Output = Angle;
            fn $op(self, rhs: Angle) -> Angle {
                *self $sym rhs
            }
        }
        impl ops::$Op<&Angle> for &Angle {
            type Output = Angle;
            fn $op(self, rhs: &Angle) -> Angle {
                *self $sym *rhs
            }
        }
        impl ops::$OpAssign<Angle> for Angle {
            fn $op_assign(&mut self, rhs: Angle) {
                *self = *self $sym rhs;
            }
        }
        impl ops::$OpAssign<&Angle> for Angle {
            fn $op_assign(&mut self, rhs: &Angle) {
                *self = *self $sym *rhs;
            }
        }
    };
}

/// `$op` of an angle, owned or borrowed, by an `f32`, and its assign form.
macro_rules! angle_scalar_op {
    ($Op:ident, $op:ident, $OpAssign:ident, $op_assign:ident, $sym:tt) => {
        impl ops::$Op<f32> for Angle {
            type Output = Angle;
            fn $op(self, rhs: f32) -> Angle {
                Angle::from_rad(self.to_rad() $sym rhs)
            }
        }
        impl ops::$Op<f32> for &Angle {
            type Output = Angle;
            fn $op(self, rhs: f32) -> Angle {
                *self $sym rhs
            }
        }
        impl ops::$OpAssign<f32> for Angle {
            fn $op_assign(&mut self, rhs: f32) {
                *self = *self $sym rhs;
            }
        }
    };
}

angle_op!(Add, add, AddAssign, add_assign, +);
angle_op!(Sub, sub, SubAssign, sub_assign, -);
angle_scalar_op!(Mul, mul, MulAssign, mul_assign, *);
angle_scalar_op!(Div, div, DivAssign, div_assign, /);

// equality
impl PartialEq<Angle> for Angle {
    fn eq(&self, rhs: &Angle) -> bool {
        (self.to_rad() - rhs.to_rad()).abs() < ANGLE_TOLERANCE
    }
}

impl PartialEq<&Angle> for Angle {
    fn eq(&self, rhs: &&Angle) -> bool {
        (self.to_rad() - rhs.to_rad()).abs() < ANGLE_TOLERANCE
    }
}

impl PartialEq<Angle> for &Angle {
    fn eq(&self, rhs: &Angle) -> bool {
        (self.to_rad() - rhs.to_rad()).abs() < ANGLE_TOLERANCE
    }
}

// negation
impl ops::Neg for Angle {
    type Output = Angle;
    fn neg(self) -> Angle {
        Angle::from_rad(-self.to_rad())
    }
}
impl ops::Neg for &Angle {
    type Output = Angle;
    fn neg(self) -> Angle {
        Angle::from_rad(-self.to_rad())
    }
}

impl ops::Mul<Angle> for f32 {
    type Output = Angle;
    fn mul(self, rhs: Angle) -> Angle {
        rhs * self
    }
}
impl ops::Mul<&Angle> for f32 {
    type Output = Angle;
    fn mul(self, rhs: &Angle) -> Angle {
        *rhs * self
    }
}

impl std::iter::Sum for Angle {
    fn sum<I: Iterator<Item = Angle>>(iter: I) -> Self {
        iter.fold(Angle::zero(), |acc, a| acc + a)
    }
}
impl<'a> std::iter::Sum<&'a Angle> for Angle {
    fn sum<I: Iterator<Item = &'a Angle>>(iter: I) -> Self {
        iter.fold(Angle::zero(), |acc, a| acc + a)
    }
}
