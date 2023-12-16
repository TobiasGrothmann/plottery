use std::ops;

use super::vec2::V2;

// #################### ADDITION ####################

// adding vectors
impl ops::Add<V2> for V2 {
    type Output = V2;
    fn add(self, _rhs: V2) -> V2 {
        V2::new(self.x + _rhs.x, self.y + _rhs.y)
    }
}
impl ops::Add<&V2> for V2 {
    type Output = V2;
    fn add(self, _rhs: &V2) -> V2 {
        V2::new(self.x + _rhs.x, self.y + _rhs.y)
    }
}
impl ops::Add<V2> for &V2 {
    type Output = V2;
    fn add(self, _rhs: V2) -> V2 {
        V2::new(self.x + _rhs.x, self.y + _rhs.y)
    }
}
impl ops::Add<&V2> for &V2 {
    type Output = V2;
    fn add(self, _rhs: &V2) -> V2 {
        V2::new(self.x + _rhs.x, self.y + _rhs.y)
    }
}

// adding number to vector
impl ops::Add<f32> for V2 {
    type Output = V2;
    fn add(self, _rhs: f32) -> V2 {
        V2::new(self.x + _rhs, self.y + _rhs)
    }
}
impl ops::Add<f32> for &V2 {
    type Output = V2;
    fn add(self, _rhs: f32) -> V2 {
        V2::new(self.x + _rhs, self.y + _rhs)
    }
}

// #################### SUBTRACTION ####################

// subtracting vectors
impl ops::Sub<V2> for V2 {
    type Output = V2;
    fn sub(self, _rhs: V2) -> V2 {
        V2::new(self.x - _rhs.x, self.y - _rhs.y)
    }
}
impl ops::Sub<&V2> for V2 {
    type Output = V2;
    fn sub(self, _rhs: &V2) -> V2 {
        V2::new(self.x - _rhs.x, self.y - _rhs.y)
    }
}
impl ops::Sub<V2> for &V2 {
    type Output = V2;
    fn sub(self, _rhs: V2) -> V2 {
        V2::new(self.x - _rhs.x, self.y - _rhs.y)
    }
}
impl ops::Sub<&V2> for &V2 {
    type Output = V2;
    fn sub(self, _rhs: &V2) -> V2 {
        V2::new(self.x - _rhs.x, self.y - _rhs.y)
    }
}

// subtracting number from vector
impl ops::Sub<f32> for V2 {
    type Output = V2;
    fn sub(self, _rhs: f32) -> V2 {
        V2::new(self.x - _rhs, self.y - _rhs)
    }
}
impl ops::Sub<f32> for &V2 {
    type Output = V2;
    fn sub(self, _rhs: f32) -> V2 {
        V2::new(self.x - _rhs, self.y - _rhs)
    }
}

// #################### MULTIPLICATION ####################

// multiplying vectors
impl ops::Mul<V2> for V2 {
    type Output = V2;
    fn mul(self, _rhs: V2) -> V2 {
        V2::new(self.x * _rhs.x, self.y * _rhs.y)
    }
}
impl ops::Mul<&V2> for V2 {
    type Output = V2;
    fn mul(self, _rhs: &V2) -> V2 {
        V2::new(self.x * _rhs.x, self.y * _rhs.y)
    }
}
impl ops::Mul<V2> for &V2 {
    type Output = V2;
    fn mul(self, _rhs: V2) -> V2 {
        V2::new(self.x * _rhs.x, self.y * _rhs.y)
    }
}
impl ops::Mul<&V2> for &V2 {
    type Output = V2;
    fn mul(self, _rhs: &V2) -> V2 {
        V2::new(self.x * _rhs.x, self.y * _rhs.y)
    }
}

// multiplying number and vector
impl ops::Mul<f32> for V2 {
    type Output = V2;
    fn mul(self, _rhs: f32) -> V2 {
        V2::new(self.x * _rhs, self.y * _rhs)
    }
}
impl ops::Mul<f32> for &V2 {
    type Output = V2;
    fn mul(self, _rhs: f32) -> V2 {
        V2::new(self.x * _rhs, self.y * _rhs)
    }
}

// #################### DIVISON ####################

// dividing vectors
impl ops::Div<V2> for V2 {
    type Output = V2;
    fn div(self, _rhs: V2) -> V2 {
        V2::new(self.x / _rhs.x, self.y / _rhs.y)
    }
}
impl ops::Div<&V2> for V2 {
    type Output = V2;
    fn div(self, _rhs: &V2) -> V2 {
        V2::new(self.x / _rhs.x, self.y / _rhs.y)
    }
}
impl ops::Div<V2> for &V2 {
    type Output = V2;
    fn div(self, _rhs: V2) -> V2 {
        V2::new(self.x / _rhs.x, self.y / _rhs.y)
    }
}
impl ops::Div<&V2> for &V2 {
    type Output = V2;
    fn div(self, _rhs: &V2) -> V2 {
        V2::new(self.x / _rhs.x, self.y / _rhs.y)
    }
}

// divinding number and vector
impl ops::Div<f32> for V2 {
    type Output = V2;
    fn div(self, _rhs: f32) -> V2 {
        V2::new(self.x / _rhs, self.y / _rhs)
    }
}
impl ops::Div<f32> for &V2 {
    type Output = V2;
    fn div(self, _rhs: f32) -> V2 {
        V2::new(self.x / _rhs, self.y / _rhs)
    }
}

// #################### EQUALITY ####################

impl PartialEq<V2> for V2 {
    fn eq(&self, _rhs: &V2) -> bool {
        self.dist_manhattan(_rhs) < 0.00001
    }
}
