use serde::{Deserialize, Serialize};

use crate::LARGE_EPSILON;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ColorRgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl ColorRgb {
    pub fn hsv(&self) -> ColorHsv {
        (*self).into()
    }

    pub fn black() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        }
    }

    pub fn hex(&self) -> String {
        format!(
            "#{:02x}{:02x}{:02x}",
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8
        )
    }
}

impl From<ColorRgb> for ColorHsv {
    fn from(rgb: ColorRgb) -> Self {
        let r = rgb.r;
        let g = rgb.g;
        let b = rgb.b;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * ((b - r) / delta + 2.0)
        } else {
            60.0 * ((r - g) / delta + 4.0)
        };
        let s = if max == 0.0 { 0.0 } else { delta / max };
        let v = max;
        Self { h, s, v }
    }
}

impl From<&ColorRgb> for ColorHsv {
    fn from(rgb: &ColorRgb) -> Self {
        ColorHsv::from(*rgb)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ColorHsv {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

impl ColorHsv {
    pub fn rgb(&self) -> ColorRgb {
        (*self).into()
    }
}

impl From<ColorHsv> for ColorRgb {
    fn from(hsv: ColorHsv) -> Self {
        let h = hsv.h;
        let s = hsv.s;
        let v = hsv.v;
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;
        let (r, g, b) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };
        Self {
            r: r + m,
            g: g + m,
            b: b + m,
        }
    }
}

impl PartialEq for ColorRgb {
    fn eq(&self, other: &Self) -> bool {
        (self.r - other.r).abs() < LARGE_EPSILON
            && (self.g - other.g).abs() < LARGE_EPSILON
            && (self.b - other.b).abs() < LARGE_EPSILON
    }
}

impl PartialEq for ColorHsv {
    fn eq(&self, other: &Self) -> bool {
        (self.h - other.h).abs() < LARGE_EPSILON
            && (self.s - other.s).abs() < LARGE_EPSILON
            && (self.v - other.v).abs() < LARGE_EPSILON
    }
}

impl PartialEq<ColorRgb> for ColorHsv {
    fn eq(&self, other: &ColorRgb) -> bool {
        self.rgb() == *other
    }
}

impl PartialEq<ColorHsv> for ColorRgb {
    fn eq(&self, other: &ColorHsv) -> bool {
        *self == other.rgb()
    }
}
