use std::f32::consts::SQRT_2;

use crate::{Angle, Bezier, Line, Rotate, Spline, LARGE_EPSILON, V2};

/// Boundary condition at one end of an open Hobby run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum EndCondition {
    /// MetaPost `curl`: 1 is the natural end, 0 straightens the end, larger curls it more.
    Curl(f32),
    /// Fixed tangent direction.
    Direction(Angle),
}

/// Removes zero-length chords.
fn dedup_points(points: &[V2]) -> Vec<V2> {
    points.iter().fold(Vec::new(), |mut acc: Vec<V2>, p| {
        if acc.last().is_none_or(|last| last.dist(*p) > LARGE_EPSILON) {
            acc.push(*p);
        }
        acc
    })
}

fn tension_at(tensions: &[f32], i: usize) -> f32 {
    tensions.get(i).copied().unwrap_or(1.0).max(0.75)
}

/// Signed turning angle from `a` to `b` in `(-π, π]`.
fn turning_angle(a: V2, b: V2) -> f32 {
    (a.x * b.y - a.y * b.x).atan2(a.dot(b))
}

/// MetaPost's `curl_ratio`: coefficient relating the end angle to its neighbour under curl `gamma`.
fn curl_ratio(gamma: f32, alpha: f32, beta: f32) -> f32 {
    let ratio = ((3.0 - alpha) * alpha * alpha * gamma + beta * beta * beta)
        / (alpha * alpha * alpha * gamma + (3.0 - beta) * beta * beta);
    ratio.min(4.0)
}

/// Hobby's velocity function `ρ(θ, φ)`; `σ(θ, φ) = ρ(φ, θ)`.
fn rho(theta: f32, phi: f32) -> f32 {
    let (st, ct) = theta.sin_cos();
    let (sf, cf) = phi.sin_cos();
    let c = (3.0 - 5f32.sqrt()) / 2.0;
    (2.0 + SQRT_2 * (st - sf / 16.0) * (sf - st / 16.0) * (ct - cf))
        / (1.0 + (1.0 - c) * ct + c * cf)
}

/// Cubic from `from` to `to` with Hobby's handle lengths for angles `theta` and `phi`.
fn segment(from: V2, to: V2, theta: f32, phi: f32, alpha: f32, beta: f32) -> Bezier {
    let chord = to - from;
    let length = chord.length();
    let direction = chord / length;
    let start_scale = (alpha * rho(theta, phi) / 3.0).min(4.0);
    let end_scale = (beta * rho(phi, theta) / 3.0).min(4.0);
    let ctrl1 = from + direction.rotate(Angle::from_rad(theta)) * (length * start_scale);
    let ctrl2 = to - direction.rotate(Angle::from_rad(-phi)) * (length * end_scale);
    Bezier::cubic(from, ctrl1, ctrl2, to)
}

/// Solves `sub[i] x[i-1] + diag[i] x[i] + sup[i] x[i+1] = rhs[i]` (Thomas algorithm).
fn solve_tridiagonal(sub: &[f32], diag: &[f32], sup: &[f32], rhs: &[f32]) -> Vec<f32> {
    let n = diag.len();
    let mut sup_mod = vec![0.0; n];
    let mut rhs_mod = vec![0.0; n];
    sup_mod[0] = sup[0] / diag[0];
    rhs_mod[0] = rhs[0] / diag[0];
    for i in 1..n {
        let denom = diag[i] - sub[i] * sup_mod[i - 1];
        sup_mod[i] = sup[i] / denom;
        rhs_mod[i] = (rhs[i] - sub[i] * rhs_mod[i - 1]) / denom;
    }
    let mut x = rhs_mod;
    for i in (0..n - 1).rev() {
        x[i] -= sup_mod[i] * x[i + 1];
    }
    x
}

/// Tridiagonal system with corner entries `top_right` (row 0) and `bottom_left` (last row), by Sherman–Morrison.
fn solve_cyclic_tridiagonal(
    sub: &[f32],
    diag: &[f32],
    sup: &[f32],
    rhs: &[f32],
    top_right: f32,
    bottom_left: f32,
) -> Vec<f32> {
    let n = diag.len();
    let gamma = -diag[0];
    let mut diag_mod = diag.to_vec();
    diag_mod[0] -= gamma;
    diag_mod[n - 1] -= top_right * bottom_left / gamma;

    let x = solve_tridiagonal(sub, &diag_mod, sup, rhs);
    let mut u = vec![0.0; n];
    u[0] = gamma;
    u[n - 1] = bottom_left;
    let z = solve_tridiagonal(sub, &diag_mod, sup, &u);

    let factor =
        (x[0] + top_right * x[n - 1] / gamma) / (1.0 + z[0] + top_right * z[n - 1] / gamma);
    x.iter().zip(z).map(|(x, z)| x - factor * z).collect()
}

/// Coefficients of the mock curvature equation at knot `k` between chords `prev` and `next`.
/// Returns `(A, B, C, D)` such that `A θ_{k-1} + (B + C) θ_k + D θ_{k+1} = -B ψ_k - D ψ_{k+1}`.
fn interior_coefficients(
    prev_length: f32,
    next_length: f32,
    alpha_prev: f32,
    beta_k: f32,
    alpha_k: f32,
    beta_next: f32,
) -> (f32, f32, f32, f32) {
    let a = alpha_prev / (beta_k * beta_k * prev_length);
    let b = (3.0 - alpha_prev) / (beta_k * beta_k * prev_length);
    let c = (3.0 - beta_next) / (alpha_k * alpha_k * next_length);
    let d = beta_next / (alpha_k * alpha_k * next_length);
    (a, b, c, d)
}

/// Hobby's algorithm for an open run of knots, one tension per knot. Returns one cubic per chord.
pub(crate) fn hobby_open(
    points: &[V2],
    tensions: &[f32],
    start: EndCondition,
    end: EndCondition,
) -> Vec<Bezier> {
    let points = dedup_points(points);
    if points.len() < 2 {
        return vec![];
    }
    let n = points.len() - 1;
    let chords: Vec<V2> = points.windows(2).map(|pair| pair[1] - pair[0]).collect();
    let lengths: Vec<f32> = chords.iter().map(V2::length).collect();
    let alpha = |k: usize| 1.0 / tension_at(tensions, k);
    let beta = |k: usize| 1.0 / tension_at(tensions, k);
    // psi[k]: turning angle at knot k, zero at both ends
    let psi: Vec<f32> = std::iter::once(0.0)
        .chain(
            chords
                .windows(2)
                .map(|pair| turning_angle(pair[0], pair[1])),
        )
        .chain(std::iter::once(0.0))
        .collect();

    let mut sub = vec![0.0; n + 1];
    let mut diag = vec![0.0; n + 1];
    let mut sup = vec![0.0; n + 1];
    let mut rhs = vec![0.0; n + 1];

    for k in 1..n {
        let (a, b, c, d) = interior_coefficients(
            lengths[k - 1],
            lengths[k],
            alpha(k - 1),
            beta(k),
            alpha(k),
            beta(k + 1),
        );
        sub[k] = a;
        diag[k] = b + c;
        sup[k] = d;
        rhs[k] = -b * psi[k] - d * psi[k + 1];
    }

    match start {
        EndCondition::Direction(angle) => {
            diag[0] = 1.0;
            rhs[0] = turning_angle(chords[0], V2::polar(angle, 1.0));
        }
        EndCondition::Curl(gamma) => {
            let ratio = curl_ratio(gamma, alpha(0), beta(1));
            diag[0] = 1.0;
            sup[0] = ratio;
            rhs[0] = -ratio * psi[1];
        }
    }
    match end {
        EndCondition::Direction(angle) => {
            diag[n] = 1.0;
            rhs[n] = turning_angle(chords[n - 1], V2::polar(angle, 1.0));
        }
        EndCondition::Curl(gamma) => {
            let ratio = curl_ratio(gamma, beta(n), alpha(n - 1));
            sub[n] = ratio;
            diag[n] = 1.0;
            rhs[n] = 0.0;
        }
    }

    let theta = if n == 1 && matches!((start, end), (EndCondition::Curl(_), EndCondition::Curl(_)))
    {
        vec![0.0, 0.0]
    } else {
        solve_tridiagonal(&sub, &diag, &sup, &rhs)
    };

    (0..n)
        .map(|k| {
            let phi = -(psi[k + 1] + theta[k + 1]);
            segment(
                points[k],
                points[k + 1],
                theta[k],
                phi,
                alpha(k),
                beta(k + 1),
            )
        })
        .collect()
}

/// Hobby's algorithm for a closed run of knots, one tension per knot. Returns one cubic per chord, the last one returning to the first knot.
pub(crate) fn hobby_closed(points: &[V2], tensions: &[f32]) -> Vec<Bezier> {
    let mut points = dedup_points(points);
    if points.len() > 1 && points[0].dist(points[points.len() - 1]) <= LARGE_EPSILON {
        points.pop();
    }
    let n = points.len();
    if n < 2 {
        return vec![];
    }
    let chords: Vec<V2> = (0..n).map(|k| points[(k + 1) % n] - points[k]).collect();
    let lengths: Vec<f32> = chords.iter().map(V2::length).collect();
    let alpha = |k: usize| 1.0 / tension_at(tensions, k % n);
    let beta = |k: usize| 1.0 / tension_at(tensions, k % n);
    let psi: Vec<f32> = (0..n)
        .map(|k| turning_angle(chords[(k + n - 1) % n], chords[k]))
        .collect();

    let coefficients: Vec<(f32, f32, f32, f32)> = (0..n)
        .map(|k| {
            interior_coefficients(
                lengths[(k + n - 1) % n],
                lengths[k],
                alpha((k + n - 1) % n),
                beta(k),
                alpha(k),
                beta((k + 1) % n),
            )
        })
        .collect();
    let rhs: Vec<f32> = (0..n)
        .map(|k| {
            let (_, b, _, d) = coefficients[k];
            -b * psi[k] - d * psi[(k + 1) % n]
        })
        .collect();

    let theta = if n == 2 {
        // both neighbours are the same knot, so the corner entries merge into one coefficient
        let (a0, b0, c0, d0) = coefficients[0];
        let (a1, b1, c1, d1) = coefficients[1];
        let (m00, m01, m10, m11) = (b0 + c0, a0 + d0, a1 + d1, b1 + c1);
        let det = m00 * m11 - m01 * m10;
        vec![
            (rhs[0] * m11 - m01 * rhs[1]) / det,
            (m00 * rhs[1] - m10 * rhs[0]) / det,
        ]
    } else {
        let sub: Vec<f32> = coefficients.iter().map(|c| c.0).collect();
        let diag: Vec<f32> = coefficients.iter().map(|c| c.1 + c.2).collect();
        let sup: Vec<f32> = coefficients.iter().map(|c| c.3).collect();
        solve_cyclic_tridiagonal(
            &sub,
            &diag,
            &sup,
            &rhs,
            coefficients[0].0,
            coefficients[n - 1].3,
        )
    };

    (0..n)
        .map(|k| {
            let next = (k + 1) % n;
            let phi = -(psi[next] + theta[next]);
            segment(points[k], points[next], theta[k], phi, alpha(k), beta(next))
        })
        .collect()
}

impl Spline {
    /// Smooth open curve through all points (Hobby's algorithm).
    pub fn through<I: IntoIterator<Item = V2>>(points: I) -> Spline {
        Self::through_with_tension(points, 1.0)
    }
    /// Smooth closed curve through all points (Hobby's algorithm).
    pub fn through_closed<I: IntoIterator<Item = V2>>(points: I) -> Spline {
        Self::through_closed_with_tension(points, 1.0)
    }
    /// Like [`Spline::through`] with one tension at every knot, higher is straighter.
    pub fn through_with_tension<I: IntoIterator<Item = V2>>(points: I, tension: f32) -> Spline {
        let points: Vec<V2> = points.into_iter().collect();
        let tensions = vec![tension; points.len()];
        let beziers = hobby_open(
            &points,
            &tensions,
            EndCondition::Curl(1.0),
            EndCondition::Curl(1.0),
        );
        Self::from_beziers(beziers, false)
    }
    /// Like [`Spline::through_closed`] with one tension at every knot, higher is straighter.
    pub fn through_closed_with_tension<I: IntoIterator<Item = V2>>(
        points: I,
        tension: f32,
    ) -> Spline {
        let points: Vec<V2> = points.into_iter().collect();
        let tensions = vec![tension; points.len()];
        Self::from_beziers(hobby_closed(&points, &tensions), true)
    }

    fn from_beziers(beziers: Vec<Bezier>, closed: bool) -> Spline {
        match beziers.as_slice() {
            [] => Spline::default(),
            [single] if !closed => {
                Spline::from_segments(vec![Line::new(single.from(), single.to()).into()], false)
            }
            _ => Spline::from_segments(beziers.into_iter().map(Into::into).collect(), closed),
        }
    }
}
