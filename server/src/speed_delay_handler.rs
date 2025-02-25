use plottery_server_lib::plot_setting::SpeedRange;

#[derive(Debug, Clone, Copy)]
pub struct SpeedDelayHandler {
    speed_min_cm_per_s: f32,
    speed_max_cm_per_s: f32,
    dist_per_step: f32,
}

impl SpeedDelayHandler {
    pub fn new(speed_min_cm_per_s: f32, speed_max_cm_per_s: f32, dist_per_step: f32) -> Self {
        SpeedDelayHandler {
            speed_min_cm_per_s: speed_min_cm_per_s.max(0.001),
            speed_max_cm_per_s: speed_max_cm_per_s.max(0.001),
            dist_per_step,
        }
    }

    pub fn new_from_speed_range(speed_range: &SpeedRange, dist_per_step: f32) -> Self {
        SpeedDelayHandler::new(speed_range.min, speed_range.max, dist_per_step)
    }

    // speed_fraction: fraction from 0 to 1 that is mapped to speed_min to speed_max
    pub fn get_delay_nanos(&self, speed_fraction: f32) -> u32 {
        let speed = self.speed_min_cm_per_s
            + (self.speed_max_cm_per_s - self.speed_min_cm_per_s) * speed_fraction.clamp(0.0, 1.0);
        let nanos = (self.dist_per_step / speed) as f64 * 1000.0 * 1000.0 * 1000.0;
        nanos.round() as u32
    }
}
