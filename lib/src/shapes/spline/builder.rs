use std::f32::consts::{FRAC_PI_2, TAU};

use crate::{
    shapes::spline::hobby::{hobby_closed, hobby_open, EndCondition},
    Angle, Bezier, Line, Rotate90, Segment, Spline, LARGE_EPSILON, V2,
};

/// How a knot is reached from the previous one.
#[derive(Debug, Clone, PartialEq)]
enum Connection {
    Start,
    Line,
    /// Handles solved by Hobby's algorithm.
    Curve,
    /// Interior controls of an explicit Bézier.
    Bezier(Vec<V2>),
}

/// Tangent behaviour at a knot.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Join {
    Smooth,
    Corner,
    Direction(Angle),
}

#[derive(Debug, Clone)]
struct Knot {
    point: V2,
    incoming: Connection,
    join: Join,
    tension: f32,
    curl: f32,
}

impl Spline {
    /// Starts a pen-tool style builder at `point`; MetaPost's path language as a builder.
    pub fn start(point: V2) -> SplineBuilder {
        SplineBuilder {
            knots: vec![Knot {
                point,
                incoming: Connection::Start,
                join: Join::Smooth,
                tension: 1.0,
                curl: 1.0,
            }],
        }
    }
}

/// Builds a [`Spline`] knot by knot, see [`Spline::start`].
#[derive(Debug, Clone)]
pub struct SplineBuilder {
    knots: Vec<Knot>,
}

impl SplineBuilder {
    fn current(&self) -> &Knot {
        self.knots.last().expect("builder always has a start knot")
    }
    fn current_mut(&mut self) -> &mut Knot {
        self.knots
            .last_mut()
            .expect("builder always has a start knot")
    }
    fn push_knot(mut self, point: V2, incoming: Connection) -> Self {
        self.knots.push(Knot {
            point,
            incoming,
            join: Join::Smooth,
            tension: 1.0,
            curl: 1.0,
        });
        self
    }

    /// Straight segment, MetaPost `--`.
    pub fn line_to(self, to: V2) -> Self {
        self.push_knot(to, Connection::Line)
    }
    /// Smooth segment with handles solved by Hobby's algorithm, MetaPost `..`.
    pub fn curve_to(self, to: V2) -> Self {
        self.push_knot(to, Connection::Curve)
    }
    pub fn quad_to(self, ctrl: V2, to: V2) -> Self {
        self.push_knot(to, Connection::Bezier(vec![ctrl]))
    }
    pub fn cubic_to(self, ctrl1: V2, ctrl2: V2, to: V2) -> Self {
        self.push_knot(to, Connection::Bezier(vec![ctrl1, ctrl2]))
    }
    /// Cubic whose first handle mirrors the previous handle (SVG `S`), or starts at the current point.
    pub fn smooth_cubic_to(self, ctrl2: V2, to: V2) -> Self {
        let current = self.current();
        let ctrl1 = match &current.incoming {
            Connection::Bezier(ctrls) if !ctrls.is_empty() => {
                current.point * 2.0 - ctrls[ctrls.len() - 1]
            }
            _ => current.point,
        };
        self.cubic_to(ctrl1, ctrl2, to)
    }
    /// Bézier of any degree through the interior controls `ctrls`.
    pub fn bezier_to<I: IntoIterator<Item = V2>>(self, ctrls: I, to: V2) -> Self {
        let ctrls: Vec<V2> = ctrls.into_iter().collect();
        if ctrls.is_empty() {
            return self.line_to(to);
        }
        self.push_knot(to, Connection::Bezier(ctrls))
    }

    /// Exact circular arc around `center` from the current point, as cubics of at most 90°.
    pub fn arc_around(mut self, center: V2, sweep: Angle) -> Self {
        let from = self.current().point;
        let radius = from.dist(center);
        let sweep = sweep.to_rad();
        if radius <= LARGE_EPSILON || sweep.abs() <= LARGE_EPSILON {
            return self;
        }
        let pieces = (sweep.abs() / FRAC_PI_2 - 1e-4).ceil().max(1.0) as usize;
        let piece = sweep / pieces as f32;
        let handle = 4.0 / 3.0 * (piece / 4.0).tan() * radius;
        let start_angle = (from - center).angle();

        for i in 0..pieces {
            let a0 = start_angle + Angle::from_rad(piece * i as f32);
            let a1 = start_angle + Angle::from_rad(piece * (i + 1) as f32);
            let p0 = if i == 0 {
                from
            } else {
                center + V2::polar(a0, radius)
            };
            let p1 = center + V2::polar(a1, radius);
            let ctrl1 = p0 + V2::polar(a0 + Angle::quarter_rotation(), handle);
            let ctrl2 = p1 - V2::polar(a1 + Angle::quarter_rotation(), handle);
            self = self.push_knot(p1, Connection::Bezier(vec![ctrl1, ctrl2]));
            if i + 1 < pieces {
                let tangent = V2::polar(a1 + Angle::quarter_rotation(), piece.signum());
                self.current_mut().join = Join::Direction(tangent.angle());
            }
        }
        self
    }

    /// Small arc of `radius` to `to`; a chord longer than `2 * radius` gives a half circle.
    pub fn arc_to(self, to: V2, radius: f32, clockwise: bool) -> Self {
        let from = self.current().point;
        let chord = to - from;
        let chord_length = chord.length();
        if chord_length <= LARGE_EPSILON {
            return self;
        }
        let radius = radius.abs().max(chord_length / 2.0);
        let height = (radius * radius - chord_length * chord_length / 4.0)
            .max(0.0)
            .sqrt();
        let normal_right = chord.rotate_270() / chord_length;
        let center = if clockwise {
            from + chord * 0.5 + normal_right * height
        } else {
            from + chord * 0.5 - normal_right * height
        };
        let sweep = 2.0 * (chord_length / (2.0 * radius)).min(1.0).asin();
        let sweep = if clockwise { -sweep } else { sweep };
        self.arc_around(center, Angle::from_rad(sweep))
    }

    /// Arc to `to` leaving along the current tangent; a line if `to` lies on the tangent.
    pub fn tangent_arc_to(mut self, to: V2) -> Self {
        let from = self.current().point;
        let Some(tangent) = self.current_tangent() else {
            return self.line_to(to);
        };
        let chord = to - from;
        let chord_length_squared = chord.length_squared();
        if chord_length_squared <= LARGE_EPSILON * LARGE_EPSILON {
            return self;
        }
        let normal_left = tangent.normalize().rotate_90();
        let denominator = 2.0 * chord.dot(normal_left);
        if denominator.abs() <= 1e-6 * chord_length_squared {
            return self.line_to(to);
        }
        let signed_radius = chord_length_squared / denominator;
        let center = from + normal_left * signed_radius;
        let a0 = (from - center).angle().to_rad();
        let a1 = (to - center).angle().to_rad();
        let sweep = if signed_radius > 0.0 {
            (a1 - a0).rem_euclid(TAU)
        } else {
            -(a0 - a1).rem_euclid(TAU)
        };
        self.current_mut().join = Join::Direction(tangent.angle());
        self.arc_around(center, Angle::from_rad(sweep))
    }

    /// Tangent at the current knot, the chord from the previous knot for unsolved curves.
    fn current_tangent(&self) -> Option<V2> {
        let current = self.current();
        if let Join::Direction(angle) = current.join {
            return Some(V2::polar(angle, 1.0));
        }
        let previous = self
            .knots
            .len()
            .checked_sub(2)
            .map(|i| self.knots[i].point)?;
        let tangent = match &current.incoming {
            Connection::Start => return None,
            Connection::Bezier(ctrls) if !ctrls.is_empty() => {
                current.point - ctrls[ctrls.len() - 1]
            }
            _ => current.point - previous,
        };
        (tangent.length_squared() > LARGE_EPSILON * LARGE_EPSILON).then_some(tangent)
    }

    /// The tangent may break at the knot just added.
    pub fn corner(mut self) -> Self {
        self.current_mut().join = Join::Corner;
        self
    }
    /// Fixes the tangent direction at the knot just added.
    pub fn direction(mut self, angle: Angle) -> Self {
        self.current_mut().join = Join::Direction(angle);
        self
    }
    /// MetaPost tension at the knot just added, default `1.0`, at least `0.75`, higher is straighter.
    pub fn tension(mut self, tension: f32) -> Self {
        self.current_mut().tension = tension.max(0.75);
        self
    }
    /// MetaPost curl where a curve run ends at the knot just added, default `1.0`, `0.0` is straight.
    pub fn curl(mut self, curl: f32) -> Self {
        self.current_mut().curl = curl.max(0.0);
        self
    }

    /// Open spline.
    pub fn build(self) -> Spline {
        self.finish(None)
    }
    /// Closes with a straight segment back to the start, MetaPost `-- cycle`.
    pub fn close(self) -> Spline {
        self.finish(Some(Connection::Line))
    }
    /// Closes with a Hobby segment back to the start, MetaPost `.. cycle`.
    pub fn close_curve(self) -> Spline {
        self.finish(Some(Connection::Curve))
    }

    fn finish(self, closing: Option<Connection>) -> Spline {
        let mut knots = self.knots;
        if knots.len() < 2 {
            return Spline::default();
        }
        let Some(closing) = closing else {
            return Spline::from_segments(build_linear(&knots, false), false);
        };

        if knots[knots.len() - 1].point.dist(knots[0].point) > LARGE_EPSILON {
            let first = knots[0].clone();
            knots.push(Knot {
                incoming: closing,
                ..first
            });
        }
        // the distinct knots; the last entry closes back to the first one
        let count = knots.len() - 1;
        let incoming_of = |i: usize| -> &Connection {
            if i == 0 {
                &knots[count].incoming
            } else {
                &knots[i].incoming
            }
        };

        let all_smooth_curves = (0..count)
            .all(|i| knots[i].join == Join::Smooth && *incoming_of(i) == Connection::Curve);
        if all_smooth_curves {
            let points: Vec<V2> = knots[..count].iter().map(|k| k.point).collect();
            let tensions: Vec<f32> = knots[..count].iter().map(|k| k.tension).collect();
            let segments = hobby_closed(&points, &tensions)
                .into_iter()
                .map(Segment::Bezier)
                .collect();
            return Spline::from_segments(segments, true);
        }

        // rotate so that the sequence starts at a run boundary, then no Hobby run wraps around
        let start = (0..count)
            .find(|&i| knots[i].join != Join::Smooth || *incoming_of(i) != Connection::Curve)
            .unwrap_or(0);
        let rotated: Vec<Knot> = (0..=count)
            .map(|i| {
                let source = (start + i) % count;
                Knot {
                    incoming: if i == 0 {
                        Connection::Start
                    } else {
                        incoming_of(source).clone()
                    },
                    ..knots[source].clone()
                }
            })
            .collect();
        let mut segments = build_linear(&rotated, true);
        segments.rotate_left((count - start) % count);
        Spline::from_segments(segments, true)
    }
}

/// Builds the segments between consecutive knots, solving each run of `Curve` connections with Hobby's algorithm.
fn build_linear(knots: &[Knot], closed: bool) -> Vec<Segment> {
    let n = knots.len();
    let mut segments: Vec<Option<Segment>> = (1..n)
        .map(|i| explicit_segment(&knots[i - 1], &knots[i]))
        .collect();

    let mut a = 0;
    while a + 1 < n {
        if segments[a].is_some() {
            a += 1;
            continue;
        }
        let mut b = a + 1;
        while b + 1 < n && segments[b].is_none() && knots[b].join == Join::Smooth {
            b += 1;
        }

        let previous = if a > 0 {
            segments[a - 1].as_ref()
        } else if closed {
            segments[n - 2].as_ref()
        } else {
            None
        };
        let next = if b + 1 < n {
            segments[b].as_ref()
        } else if closed {
            segments[0].as_ref()
        } else {
            None
        };
        let start_condition = end_condition(&knots[a], previous.map(Segment::tangent_at_end));
        let end_condition = end_condition(&knots[b], next.map(Segment::tangent_at_start));

        let points: Vec<V2> = knots[a..=b].iter().map(|k| k.point).collect();
        let tensions: Vec<f32> = knots[a..=b].iter().map(|k| k.tension).collect();
        let beziers = hobby_open(&points, &tensions, start_condition, end_condition);
        if beziers.len() == b - a {
            for (slot, bezier) in segments[a..b].iter_mut().zip(beziers) {
                *slot = Some(Segment::Bezier(bezier));
            }
        } else {
            // duplicate knots in the run, fall back to straight segments
            for (slot, pair) in segments[a..b].iter_mut().zip(points.windows(2)) {
                *slot = Some(Segment::Line(Line::new(pair[0], pair[1])));
            }
        }
        a = b;
    }

    segments.into_iter().flatten().collect()
}

fn explicit_segment(from: &Knot, to: &Knot) -> Option<Segment> {
    match &to.incoming {
        Connection::Line => Some(Segment::Line(Line::new(from.point, to.point))),
        Connection::Bezier(ctrls) => Some(Segment::Bezier(Bezier::new(
            std::iter::once(from.point)
                .chain(ctrls.iter().copied())
                .chain(std::iter::once(to.point)),
        ))),
        Connection::Curve | Connection::Start => None,
    }
}

/// Hobby end condition at `knot`, continuing an adjacent explicit segment's tangent when smooth.
fn end_condition(knot: &Knot, neighbour_tangent: Option<V2>) -> EndCondition {
    match knot.join {
        Join::Direction(angle) => EndCondition::Direction(angle),
        Join::Corner => EndCondition::Curl(knot.curl),
        Join::Smooth => neighbour_tangent
            .filter(|tangent| tangent.length_squared() > LARGE_EPSILON * LARGE_EPSILON)
            .map(|tangent| EndCondition::Direction(tangent.angle()))
            .unwrap_or(EndCondition::Curl(knot.curl)),
    }
}
