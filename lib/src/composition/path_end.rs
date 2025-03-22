use crate::{Angle, Path, V2};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathEnd {
    pub point: V2,
    pub angle: Angle,
}

impl PathEnd {
    pub fn is_compatible(&self, other: &PathEnd, max_delta: &Option<Angle>) -> bool {
        match max_delta {
            None => self.point == other.point,
            Some(delta) => {
                if self.point != other.point {
                    return false;
                };
                let angle_difference = self.angle.dist_mod_one_rotation(other.angle);
                angle_difference.abs() <= *delta
            }
        }
    }

    pub fn from_path_start(path: &Path) -> Self {
        PathEnd {
            point: *path.get_start().unwrap(),
            angle: path
                .get_points_ref()
                .first()
                .unwrap()
                .angle_to(*path.get_points_ref().get(1).unwrap()),
        }
    }

    pub fn from_path_end(path: &Path) -> Self {
        let path_len = path.get_points_ref().len();
        PathEnd {
            point: *path.get_end().unwrap(),
            angle: path
                .get_points_ref()
                .get(path_len - 2)
                .unwrap()
                .angle_to(*path.get_points_ref().get(path_len - 1).unwrap()),
        }
    }

    pub fn flipped(&self) -> Self {
        PathEnd {
            point: self.point,
            angle: self.angle + Angle::half_rotation(),
        }
    }
}
