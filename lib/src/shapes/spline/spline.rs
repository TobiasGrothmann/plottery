use serde::{Deserialize, Serialize};

use crate::{
    geometry::TransformMatrix, Angle, BoundingBox, ClosestPoint, Fit, Line, Mirror, Path, Rect,
    Rotate, Rotate90, SampleSettings, Scale, Scale2D, Segment, Transform, Translate, V2,
};

/// Chain of connected segments, optionally closed. Build with [`Spline::start`] or the `through*` constructors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Spline {
    segments: Vec<Segment>,
    closed: bool,
}

impl Spline {
    pub(crate) fn from_segments(segments: Vec<Segment>, closed: bool) -> Self {
        Self { segments, closed }
    }

    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }
    pub fn is_closed(&self) -> bool {
        self.closed
    }
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }
    /// Number of segments.
    pub fn len(&self) -> usize {
        self.segments.len()
    }
    pub fn start_point(&self) -> Option<V2> {
        self.segments.first().map(Segment::from)
    }
    pub fn end_point(&self) -> Option<V2> {
        self.segments.last().map(Segment::to)
    }

    /// The on-curve vertices: every segment start, plus the end if open.
    pub fn knots(&self) -> impl Iterator<Item = V2> + '_ {
        let end = if self.closed { None } else { self.end_point() };
        self.segments.iter().map(Segment::from).chain(end)
    }

    /// All control points, knots included (for drawing handles).
    pub fn control_points(&self) -> impl Iterator<Item = V2> + '_ {
        let count = self
            .segments
            .iter()
            .map(|segment| segment.control_points().count() - 1)
            .sum::<usize>()
            + usize::from(!self.closed);
        self.start_point()
            .into_iter()
            .chain(
                self.segments
                    .iter()
                    .flat_map(|segment| segment.control_points().skip(1)),
            )
            .take(count.max(1))
    }

    /// Handle lines `(knot, control)` for drawing, one per off-curve control adjacent to a knot.
    pub fn handles(&self) -> impl Iterator<Item = Line> + '_ {
        self.segments.iter().flat_map(|segment| match segment {
            Segment::Bezier(bezier) if bezier.degree() >= 2 => {
                let controls = bezier.controls();
                vec![
                    Line::new(bezier.from(), controls[1]),
                    Line::new(bezier.to(), controls[controls.len() - 2]),
                ]
            }
            _ => vec![],
        })
    }

    pub fn length(&self) -> f32 {
        self.segments.iter().map(Segment::length).sum()
    }

    fn locate(&self, t: f32) -> Option<(&Segment, f32)> {
        if self.segments.is_empty() {
            return None;
        }
        let scaled = t.clamp(0.0, 1.0) * self.segments.len() as f32;
        let index = (scaled.floor() as usize).min(self.segments.len() - 1);
        Some((&self.segments[index], scaled - index as f32))
    }

    /// `t` in `0..=1` over the whole spline, uniform per segment.
    pub fn point_at(&self, t: f32) -> V2 {
        self.locate(t)
            .map(|(segment, local)| segment.point_at(local))
            .unwrap_or_default()
    }
    pub fn tangent_at(&self, t: f32) -> V2 {
        self.locate(t)
            .map(|(segment, local)| segment.tangent_at(local))
            .unwrap_or_default()
    }

    /// Sampled points of all segments, join points appear once.
    pub fn points(&self, sample_settings: SampleSettings) -> Vec<V2> {
        self.start_point()
            .into_iter()
            .chain(
                self.segments
                    .iter()
                    .flat_map(|segment| segment.points(sample_settings).into_iter().skip(1)),
            )
            .collect()
    }
    pub fn to_path(&self, sample_settings: SampleSettings) -> Path {
        Path::new_from(self.points(sample_settings))
    }

    pub fn reversed(&self) -> Spline {
        Spline {
            segments: self.segments.iter().rev().map(Segment::reversed).collect(),
            closed: self.closed,
        }
    }

    /// Appends a segment, which must start where the spline ends.
    pub fn push(&mut self, segment: Segment) {
        debug_assert!(
            self.end_point().is_none_or(|end| end == segment.from()),
            "segment does not continue the spline"
        );
        self.segments.push(segment);
    }

    fn map_points(&self, f: impl Fn(V2) -> V2) -> Spline {
        Spline {
            segments: self
                .segments
                .iter()
                .map(|segment| segment.map_points(&f))
                .collect(),
            closed: self.closed,
        }
    }
}

impl Translate for Spline {
    fn translate(&self, dist: V2) -> Self {
        self.map_points(|p| p + dist)
    }
}

impl Rotate for Spline {
    fn rotate(&self, angle: Angle) -> Self {
        self.map_points(|p| p.rotate(angle))
    }
    fn rotate_around(&self, pivot: V2, angle: Angle) -> Self {
        self.map_points(|p| p.rotate_around(pivot, angle))
    }
}

impl Rotate90 for Spline {
    fn rotate_90(&self) -> Self {
        self.map_points(|p| p.rotate_90())
    }
    fn rotate_180(&self) -> Self {
        self.map_points(|p| p.rotate_180())
    }
    fn rotate_270(&self) -> Self {
        self.map_points(|p| p.rotate_270())
    }
}

impl Scale for Spline {
    fn scale(&self, factor: f32) -> Self {
        self.map_points(|p| p * factor)
    }
}

impl Scale2D for Spline {
    fn scale_2d(&self, factor: V2) -> Self {
        self.map_points(|p| p * factor)
    }
}

impl Mirror for Spline {
    fn mirror_x(&self) -> Self {
        self.map_points(|p| p.mirror_x())
    }
    fn mirror_y(&self) -> Self {
        self.map_points(|p| p.mirror_y())
    }
}

impl Transform for Spline {
    fn transform(&self, matrix: &TransformMatrix) -> Self {
        self.map_points(|p| matrix.mul_vector(p))
    }
}

/// Hull of all control points, which is conservative: the curve stays inside but may not touch it.
impl BoundingBox for Spline {
    fn bounding_box(&self) -> Option<Rect> {
        self.control_points()
            .fold(None, |acc, point| match acc {
                None => Some((point, point)),
                Some((min, max)) => Some((min.min(point), max.max(point))),
            })
            .map(|(min, max)| Rect::new(min, max))
    }
}

impl Fit for Spline {}

/// Approximate, measured on the sampled path.
impl ClosestPoint for Spline {
    fn closest_point(&self, sample_settings: SampleSettings, point: V2) -> Option<V2> {
        self.to_path(sample_settings)
            .closest_point(sample_settings, point)
    }
}
