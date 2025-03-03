use std::ops;

use super::angle::Angle;
use crate::LARGE_EPSILON;

// add
impl ops::Add<Angle> for Angle {
    type Output = Angle;
    fn add(self, _rhs: Angle) -> Angle {
        Angle::from_rad(self.to_rad() + _rhs.to_rad())
    }
}
impl ops::Add<&Angle> for Angle {
    type Output = Angle;
    fn add(self, _rhs: &Angle) -> Angle {
        Angle::from_rad(self.to_rad() + _rhs.to_rad())
    }
}
impl ops::Add<Angle> for &Angle {
    type Output = Angle;
    fn add(self, _rhs: Angle) -> Angle {
        Angle::from_rad(self.to_rad() + _rhs.to_rad())
    }
}
impl ops::Add<&Angle> for &Angle {
    type Output = Angle;
    fn add(self, _rhs: &Angle) -> Angle {
        Angle::from_rad(self.to_rad() + _rhs.to_rad())
    }
}
// add assign
impl ops::AddAssign<Angle> for Angle {
    fn add_assign(&mut self, rhs: Angle) {
        *self = Angle::from_rad(self.to_rad() + rhs.to_rad());
    }
}
impl ops::AddAssign<&Angle> for Angle {
    fn add_assign(&mut self, rhs: &Angle) {
        *self = Angle::from_rad(self.to_rad() + rhs.to_rad());
    }
}

// subtract
impl ops::Sub<Angle> for Angle {
    type Output = Angle;
    fn sub(self, _rhs: Angle) -> Angle {
        Angle::from_rad(self.to_rad() - _rhs.to_rad())
    }
}
impl ops::Sub<&Angle> for Angle {
    type Output = Angle;
    fn sub(self, _rhs: &Angle) -> Angle {
        Angle::from_rad(self.to_rad() - _rhs.to_rad())
    }
}
impl ops::Sub<Angle> for &Angle {
    type Output = Angle;
    fn sub(self, _rhs: Angle) -> Angle {
        Angle::from_rad(self.to_rad() - _rhs.to_rad())
    }
}
impl ops::Sub<&Angle> for &Angle {
    type Output = Angle;
    fn sub(self, _rhs: &Angle) -> Angle {
        Angle::from_rad(self.to_rad() - _rhs.to_rad())
    }
}
// subtract assign
impl ops::SubAssign<Angle> for Angle {
    fn sub_assign(&mut self, rhs: Angle) {
        *self = Angle::from_rad(self.to_rad() - rhs.to_rad());
    }
}
impl ops::SubAssign<&Angle> for Angle {
    fn sub_assign(&mut self, rhs: &Angle) {
        *self = Angle::from_rad(self.to_rad() - rhs.to_rad());
    }
}

// multiply
impl ops::Mul<f32> for Angle {
    type Output = Angle;
    fn mul(self, _rhs: f32) -> Angle {
        Angle::from_rad(self.to_rad() * _rhs)
    }
}
impl ops::Mul<f32> for &Angle {
    type Output = Angle;
    fn mul(self, _rhs: f32) -> Angle {
        Angle::from_rad(self.to_rad() * _rhs)
    }
}
// multiply assign
impl ops::MulAssign<f32> for Angle {
    fn mul_assign(&mut self, rhs: f32) {
        *self = Angle::from_rad(self.to_rad() * rhs);
    }
}

// divide
impl ops::Div<f32> for Angle {
    type Output = Angle;
    fn div(self, _rhs: f32) -> Angle {
        Angle::from_rad(self.to_rad() / _rhs)
    }
}
impl ops::Div<f32> for &Angle {
    type Output = Angle;
    fn div(self, _rhs: f32) -> Angle {
        Angle::from_rad(self.to_rad() / _rhs)
    }
}
// divide assign
impl ops::DivAssign<f32> for Angle {
    fn div_assign(&mut self, rhs: f32) {
        *self = Angle::from_rad(self.to_rad() / rhs);
    }
}

// equality
impl PartialEq<Angle> for Angle {
    fn eq(&self, rhs: &Angle) -> bool {
        (self.to_rad() - rhs.to_rad()).abs() < LARGE_EPSILON
    }
}

impl PartialEq<&Angle> for Angle {
    fn eq(&self, rhs: &&Angle) -> bool {
        (self.to_rad() - rhs.to_rad()).abs() < LARGE_EPSILON
    }
}

impl PartialEq<Angle> for &Angle {
    fn eq(&self, rhs: &Angle) -> bool {
        (self.to_rad() - rhs.to_rad()).abs() < LARGE_EPSILON
    }
}
