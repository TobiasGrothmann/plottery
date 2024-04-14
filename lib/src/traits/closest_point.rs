use crate::{SampleSettings, V2};

pub trait ClosestPoint {
    fn closest_point(&self, sample_settings: &SampleSettings, point: &V2) -> Option<V2>;
}
