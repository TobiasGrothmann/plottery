use plottery_lib::{geometry::v2i::V2i, *};

use crate::{accelleration_path::V2Speed, pins::DIST_PER_STEP_AXIS_CM};

#[derive(Debug)]
pub struct Hardware {
    enabled: bool,

    x: i32,
    y: i32,
    head_down: bool,
}

#[derive(Debug, Clone)]
enum Axis {
    X,
    Y,
    HEAD,
}

impl Hardware {
    pub fn new() -> Self {
        Hardware {
            enabled: false,
            x: 0,
            y: 0,
            head_down: false,
        }
    }

    fn get_pos(&self) -> V2 {
        V2::new(
            self.x as f32 * DIST_PER_STEP_AXIS_CM,
            self.y as f32 * DIST_PER_STEP_AXIS_CM,
        )
    }
    fn get_pos_steps(&self) -> V2i {
        V2i::new(self.x, self.y)
    }

    fn set_dir(&mut self, axis: Axis, forward: bool) {
        // set gpio direction pins (if raspi)
        println!("Setting direction for {:?} to {:?}", axis, forward);
    }

    fn step(&mut self, axis: Axis) {
        // set gpio step pins (if raspi)
        println!("Stepping {:?}", axis);
    }

    fn move_steps(&mut self, movement: &V2i) {
        println!("Moving steps {:?}", movement);
        self.set_dir(Axis::X, movement.x > 0);
        self.set_dir(Axis::Y, movement.y > 0);

        let directions_signs = V2i::new(
            if movement.x > 0 { 1 } else { -1 },
            if movement.y > 0 { 1 } else { -1 },
        );

        let mut steppedX = 0;
        let mut steppedY = 0;

        let movement_abs = movement.abs();
        let total_steps = movement_abs.x + movement_abs.y;

        // TODO: set speed

        while steppedX < movement_abs.x || steppedY < movement_abs.y {
            let moved_fraction = steppedX as f32 + steppedY as f32 / total_steps as f32;
            // TODO: set speed according to factor

            if Line::new(V2::new(0.0, 0.0), movement_abs.abs().to_float())
                .point_relation(&V2::new(steppedX as f32, steppedY as f32))
                == PointLineRelation::Left
            {
                steppedX += 1;
                self.x += directions_signs.x;
                // TODO: step with delay from speed
                self.step(Axis::X);
            } else {
                steppedY += 1;
                self.y += directions_signs.y;
                // TODO: step with delay from speed
                self.step(Axis::Y);
            }
        }
    }

    pub fn move_to(&mut self, pos: &V2Speed) {
        let delta = ((pos.point - self.get_pos()) / DIST_PER_STEP_AXIS_CM).round_to_int();
        // TODO: use speed from pos
        if delta == V2i::new(0, 0) {
            return;
        }
        println!("Moving from {:?} to {:?}", self.get_pos(), pos.point);
        self.move_steps(&delta);
    }
}
