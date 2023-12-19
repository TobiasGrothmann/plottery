use std::ops;

use super::angle::Angle;

// adding
impl ops::Add<Angle> for Angle {
    type Output = Angle;
    fn add(self, _rhs: Angle) -> Angle {
        Angle::from_rad(self.to_rad() + _rhs.to_rad())
    }
}

impl ops::Sub<Angle> for Angle {
    type Output = Angle;
    fn sub(self, _rhs: Angle) -> Angle {
        Angle::from_rad(self.to_rad() - _rhs.to_rad())
    }
}

impl ops::Mul<f32> for Angle {
    type Output = Angle;
    fn mul(self, _rhs: f32) -> Angle {
        Angle::from_rad(self.to_rad() * _rhs)
    }
}

impl ops::Div<f32> for Angle {
    type Output = Angle;
    fn div(self, _rhs: f32) -> Angle {
        Angle::from_rad(self.to_rad() / _rhs)
    }
}
