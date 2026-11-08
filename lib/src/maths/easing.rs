use serde::{Deserialize, Serialize};

/// Largest `|amount|` used by [`Easing::apply`], keeps the curves finite.
pub const EASING_MAX_AMOUNT: f32 = 0.98;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EasingKind {
    /// `x^p`, `f(0.5) = 0.5 + amount / 2`.
    #[default]
    Power,
    /// Schlick bias, same midpoint as `Power` but with finite slopes at the ends.
    Bias,
    /// Schlick gain, `amount > 0` eases in and out, `< 0` is an inverted S.
    SCurve,
}

impl EasingKind {
    pub const ALL: [EasingKind; 3] = [EasingKind::Power, EasingKind::Bias, EasingKind::SCurve];
}

/// One-knob curve mapping 0..1 to 0..1, `amount` in -1..1 with 0 being linear.
///
/// ```
/// # use plottery_lib::*;
/// let easing = Easing::from_exponent(2.0);
/// assert!((easing.apply(0.5) - 0.25).abs() < 1e-5);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Easing {
    pub kind: EasingKind,
    pub amount: f32,
}

impl Default for Easing {
    fn default() -> Self {
        Self::linear()
    }
}

impl Easing {
    pub fn new(kind: EasingKind, amount: f32) -> Self {
        Self { kind, amount }
    }

    pub fn linear() -> Self {
        Self::new(EasingKind::Power, 0.0)
    }

    pub fn power(amount: f32) -> Self {
        Self::new(EasingKind::Power, amount)
    }

    pub fn bias(amount: f32) -> Self {
        Self::new(EasingKind::Bias, amount)
    }

    pub fn s_curve(amount: f32) -> Self {
        Self::new(EasingKind::SCurve, amount)
    }

    /// Same curve as `x.powf(exponent)`, for exponents up to about 6.6.
    pub fn from_exponent(exponent: f32) -> Self {
        Self::power(2.0 * 0.5_f32.powf(exponent) - 1.0)
    }

    pub fn apply(&self, x: f32) -> f32 {
        let x = x.clamp(0.0, 1.0);
        let amount = self.amount.clamp(-EASING_MAX_AMOUNT, EASING_MAX_AMOUNT);
        let b = (amount + 1.0) / 2.0;
        match self.kind {
            EasingKind::Power => x.powf(b.ln() / 0.5_f32.ln()),
            EasingKind::Bias => schlick_bias(x, b),
            EasingKind::SCurve => {
                let g = (1.0 - amount) / 2.0;
                if x < 0.5 {
                    schlick_bias(2.0 * x, g) / 2.0
                } else {
                    1.0 - schlick_bias(2.0 - 2.0 * x, g) / 2.0
                }
            }
        }
    }
}

fn schlick_bias(x: f32, b: f32) -> f32 {
    x / ((1.0 / b - 2.0) * (1.0 - x) + 1.0)
}
