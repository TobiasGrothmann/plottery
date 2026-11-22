use std::ops;

use crate::LENGTH_TOLERANCE;

use super::v2::V2;

/// Componentwise `$op` for every mix of owned and borrowed `V2`, with `f32` broadcast, and its assign form.
macro_rules! componentwise_op {
    ($Op:ident, $op:ident, $OpAssign:ident, $op_assign:ident, $sym:tt) => {
        impl ops::$Op<V2> for V2 {
            type Output = V2;
            fn $op(self, rhs: V2) -> V2 {
                V2::new(self.x $sym rhs.x, self.y $sym rhs.y)
            }
        }
        impl ops::$Op<&V2> for V2 {
            type Output = V2;
            fn $op(self, rhs: &V2) -> V2 {
                self $sym *rhs
            }
        }
        impl ops::$Op<V2> for &V2 {
            type Output = V2;
            fn $op(self, rhs: V2) -> V2 {
                *self $sym rhs
            }
        }
        impl ops::$Op<&V2> for &V2 {
            type Output = V2;
            fn $op(self, rhs: &V2) -> V2 {
                *self $sym *rhs
            }
        }
        impl ops::$Op<f32> for V2 {
            type Output = V2;
            fn $op(self, rhs: f32) -> V2 {
                V2::new(self.x $sym rhs, self.y $sym rhs)
            }
        }
        impl ops::$Op<f32> for &V2 {
            type Output = V2;
            fn $op(self, rhs: f32) -> V2 {
                *self $sym rhs
            }
        }
        impl ops::$OpAssign<V2> for V2 {
            fn $op_assign(&mut self, rhs: V2) {
                *self = *self $sym rhs;
            }
        }
        impl ops::$OpAssign<&V2> for V2 {
            fn $op_assign(&mut self, rhs: &V2) {
                *self = *self $sym *rhs;
            }
        }
        impl ops::$OpAssign<f32> for V2 {
            fn $op_assign(&mut self, rhs: f32) {
                *self = *self $sym rhs;
            }
        }
    };
}

componentwise_op!(Add, add, AddAssign, add_assign, +);
componentwise_op!(Sub, sub, SubAssign, sub_assign, -);
componentwise_op!(Mul, mul, MulAssign, mul_assign, *);
componentwise_op!(Div, div, DivAssign, div_assign, /);

// #################### EQUALITY ####################

impl PartialEq<V2> for V2 {
    fn eq(&self, _rhs: &V2) -> bool {
        self.dist_manhattan(*_rhs) < LENGTH_TOLERANCE
    }
}

impl PartialEq<&V2> for V2 {
    fn eq(&self, _rhs: &&V2) -> bool {
        self.dist_manhattan(**_rhs) < LENGTH_TOLERANCE
    }
}

impl PartialEq<V2> for &V2 {
    fn eq(&self, _rhs: &V2) -> bool {
        self.dist_manhattan(*_rhs) < LENGTH_TOLERANCE
    }
}

// #################### NEGATION ####################

impl ops::Neg for V2 {
    type Output = V2;
    fn neg(self) -> V2 {
        V2::new(-self.x, -self.y)
    }
}
impl ops::Neg for &V2 {
    type Output = V2;
    fn neg(self) -> V2 {
        V2::new(-self.x, -self.y)
    }
}

impl ops::Mul<V2> for f32 {
    type Output = V2;
    fn mul(self, rhs: V2) -> V2 {
        rhs * self
    }
}
impl ops::Mul<&V2> for f32 {
    type Output = V2;
    fn mul(self, rhs: &V2) -> V2 {
        *rhs * self
    }
}
