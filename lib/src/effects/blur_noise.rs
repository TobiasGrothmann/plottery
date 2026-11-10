use std::f64::consts::SQRT_2;

use rand::{rngs::StdRng, Rng};
use rand_distr::StandardNormal;

use crate::V2;

use super::Scatter;

/// Unit gaussian 2D noise per sample, correlated over `smoothness` cm, wrapping around if `closed`.
pub(super) fn gaussian_noise(
    rng: &mut StdRng,
    count: usize,
    spacing: f32,
    smoothness: f32,
    closed: bool,
) -> Vec<V2> {
    let mut normal = || V2::new(rng.sample(StandardNormal), rng.sample(StandardNormal));
    if smoothness <= 0.0 {
        return (0..count).map(|_| normal()).collect();
    }

    // AR(1): stationary with unit variance, correlation exp(-distance / smoothness)
    let rho = (-spacing / smoothness).exp();
    let fresh = (1.0 - rho * rho).sqrt();
    let blend = if closed {
        ((smoothness / spacing).ceil() as usize).min(count)
    } else {
        0
    };
    let mut noise: Vec<V2> =
        std::iter::successors(Some(normal()), |g| Some(*g * rho + normal() * fresh))
            .take(count + blend)
            .collect();

    // fade the continuation past the end into the start, so the last sample flows into the first
    let tail_correlation = rho.powf(count as f32);
    let (head, continuation) = noise.split_at_mut(count);
    for (i, (g, next)) in head.iter_mut().zip(continuation.iter()).enumerate() {
        let t = i as f32 / blend as f32;
        let variance = (1.0 - t).powi(2) + t * t + 2.0 * t * (1.0 - t) * tail_correlation;
        *g = (*next * (1.0 - t) + *g * t) / variance.sqrt();
    }
    noise.truncate(count);
    noise
}

impl Scatter {
    /// Offset for σ = 1, mapped radially from unit gaussian noise `g`.
    pub(super) fn unit_offset(self, g: V2) -> V2 {
        match self {
            Scatter::Gaussian => g,
            Scatter::Otter => {
                let r = f64::from(g.x).hypot(f64::from(g.y));
                if r == 0.0 {
                    return V2::zero();
                }
                g * (rayleigh_to_half_normal(r) / r) as f32
            }
        }
    }
}

/// Maps a 2D gaussian radius to the half-normal radius at the same quantile.
fn rayleigh_to_half_normal(r: f64) -> f64 {
    let quantile = -(-0.5 * r * r).exp_m1();
    SQRT_2 * erf_inv(quantile)
}

/// Inverse error function, Giles (2010) double precision, `x` clamped below ±1.
pub(super) fn erf_inv(x: f64) -> f64 {
    let x = x.clamp(-MAX_ERF, MAX_ERF);
    let w = -((1.0 - x) * (1.0 + x)).ln();
    let p = if w < 6.25 {
        horner(&ERF_INV_CENTRAL, w - 3.125)
    } else if w < 16.0 {
        horner(&ERF_INV_TAIL, w.sqrt() - 3.25)
    } else {
        horner(&ERF_INV_FAR_TAIL, w.sqrt() - 5.0)
    };
    p * x
}

const MAX_ERF: f64 = 1.0 - f64::EPSILON / 2.0;

fn horner(coefficients: &[f64], w: f64) -> f64 {
    coefficients.iter().fold(0.0, |p, c| c + p * w)
}

const ERF_INV_CENTRAL: [f64; 23] = [
    -3.64441206401782e-21,
    -1.6850591381820166e-19,
    1.28584807152564e-18,
    1.1157877678025181e-17,
    -1.333171662854621e-16,
    2.0972767875968562e-17,
    6.637638134358324e-15,
    -4.054566272975207e-14,
    -8.151934197605472e-14,
    2.6335093153082323e-12,
    -1.2975133253453532e-11,
    -5.415412054294628e-11,
    1.0512122733215323e-09,
    -4.112633980346984e-09,
    -2.9070369957882005e-08,
    4.2347877827932404e-07,
    -1.3654692000834679e-06,
    -1.3882523362786469e-05,
    0.00018673420803405714,
    -0.000740702534166267,
    -0.006033670871430149,
    0.24015818242558962,
    1.6536545626831027,
];

const ERF_INV_TAIL: [f64; 19] = [
    2.2137376921775787e-09,
    9.075656193888539e-08,
    -2.7517406297064545e-07,
    1.8239629214389228e-08,
    1.5027403968909828e-06,
    -4.013867526981546e-06,
    2.9234449089955446e-06,
    1.2475304481671779e-05,
    -4.7318229009055734e-05,
    6.828485145957318e-05,
    2.4031110387097894e-05,
    -0.0003550375203628475,
    0.0009532893797373805,
    -0.0016882755560235047,
    0.002491442096107851,
    -0.003751208507569241,
    0.005370914553590064,
    1.0052589676941592,
    3.0838856104922208,
];

const ERF_INV_FAR_TAIL: [f64; 17] = [
    -2.7109920616438573e-11,
    -2.555641816996525e-10,
    1.5076572693500548e-09,
    -3.789465440126737e-09,
    7.61570120807834e-09,
    -1.496002662714924e-08,
    2.914795345090108e-08,
    -6.771199775845234e-08,
    2.2900482228026655e-07,
    -9.9298272942317e-07,
    4.526062597223154e-06,
    -1.968177810553167e-05,
    7.599527703001776e-05,
    -0.00021503011930044477,
    -0.00013871931833623122,
    1.0103004648645344,
    4.849906401408584,
];
