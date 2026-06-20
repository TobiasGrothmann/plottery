use plottery_lib::{Angle, V2};

pub fn get_corner_sharpness(before: V2, point: V2, after: V2) -> f32 {
    let v_in = (point - before).normalize();
    let v_out = (after - point).normalize();
    let angle_rad = v_in.dot(v_out).clamp(-1.0, 1.0).acos();

    let angle = Angle::from_rad(angle_rad);
    (angle.to_rotations() / 0.5).abs()
}
