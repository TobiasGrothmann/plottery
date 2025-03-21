use serde::{Deserialize, Serialize};

use crate::LARGE_EPSILON;

use super::color_names::{ColorName, COLOR_NAMES};

/// A color represented in RGB (Red, Green, Blue) format.
///
/// Each component (r, g, b) is a float in range \[0-1\].
///
/// ### Example
/// ```
/// # use plottery_lib::*;
/// let red = ColorRgb::red();
/// let custom_color = ColorRgb::new(0.5, 0.7, 0.3);
/// let hex_string = custom_color.hex();
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct ColorRgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl ColorRgb {
    /// Creates a new RGB color with the specified red, green, and blue components.
    ///
    /// Each component should be between 0.0 and 1.0.
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }
    /// Converts this RGB color to HSV format.
    pub fn hsv(&self) -> ColorHsv {
        (*self).into()
    }
    /// Returns the hexadecimal representation of this color.
    /// ```
    /// # use plottery_lib::*;
    /// assert_eq!(ColorRgb::red().hex(), "#ff0000");
    /// assert_eq!(ColorRgb::black().hex(), "#000000");
    /// assert_eq!(ColorRgb::white().hex(), "#ffffff");
    /// assert_eq!(ColorRgb::yellow().hex(), "#ffff00");
    /// assert_eq!(ColorRgb::blue().hex(), "#0000ff");
    /// ```
    pub fn hex(&self) -> String {
        format!(
            "#{:02x}{:02x}{:02x}",
            (self.r.clamp(0.0, 1.0) * 255.0) as u8,
            (self.g.clamp(0.0, 1.0) * 255.0) as u8,
            (self.b.clamp(0.0, 1.0) * 255.0) as u8
        )
    }

    /// Returns the average value of the red, green, and blue components.
    pub fn brightness(&self) -> f32 {
        (self.r.clamp(0.0, 1.0) + self.g.clamp(0.0, 1.0) + self.b.clamp(0.0, 1.0)) / 3.0
    }

    /// Returns the Euclidean distance between this color and another color.
    /// This measure is not a perfect representation of human perception of color difference.
    pub fn dist_euclidean(&self, other: ColorRgb) -> f32 {
        let dr = self.r - other.r;
        let dg = self.g - other.g;
        let db = self.b - other.b;
        (dr * dr + dg * dg + db * db).sqrt()
    }

    /// Returns the closest [ColorName] from the list of predefined named colors [COLOR_NAMES].
    pub fn get_name(&self) -> ColorName {
        *COLOR_NAMES
            .iter()
            .min_by_key(|color| self.dist_euclidean(color.color).to_bits())
            .unwrap()
    }
}

impl From<ColorRgb> for ColorHsv {
    fn from(rgb: ColorRgb) -> Self {
        let r = rgb.r.clamp(0.0, 1.0);
        let g = rgb.g.clamp(0.0, 1.0);
        let b = rgb.b.clamp(0.0, 1.0);
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
        } / 360.0; // Convert degrees to 0.0-1.0 range
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

/// A color represented in HSV (Hue, Saturation, Value) format.
///
/// - Hue (h): Color \[0-1\] (`0.0` is red, `0.333` is green, `0.667` is blue)
/// - Saturation (s): Amount of color \[0-1\]
/// - Value (v): Brightness \[0-1\]
///
/// ### Example
/// ```
/// # use plottery_lib::*;
/// let red_hsv = ColorRgb::red().hsv();
/// let red_rgb = red_hsv.rgb();
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ColorHsv {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

impl ColorHsv {
    /// Creates a new HSV color with the specified hue, saturation and value values.
    ///
    /// `saturation` and `value` should be between 0.0 and 1.0.
    pub fn new(h: f32, s: f32, v: f32) -> Self {
        Self { h, s, v }
    }

    /// Converts this HSV color to RGB format.
    pub fn rgb(&self) -> ColorRgb {
        (*self).into()
    }
}

impl From<ColorHsv> for ColorRgb {
    fn from(hsv: ColorHsv) -> Self {
        let h = (hsv.h % 1.0) * 360.0;
        let s = hsv.s.clamp(0.0, 1.0);
        let v = hsv.v.clamp(0.0, 1.0);
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

impl ColorRgb {
    pub fn black() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        }
    }
    pub fn white() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        }
    }
    pub fn gray_scale(brightness: f32) -> Self {
        Self {
            r: brightness,
            g: brightness,
            b: brightness,
        }
    }
    pub fn red() -> Self {
        Self {
            r: 1.0,
            g: 0.0,
            b: 0.0,
        }
    }
    pub fn green() -> Self {
        Self {
            r: 0.0,
            g: 1.0,
            b: 0.0,
        }
    }
    pub fn blue() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 1.0,
        }
    }
    /// (1, 1, 0)
    pub fn yellow() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 0.0,
        }
    }
    /// (0, 1, 1)
    pub fn cyan() -> Self {
        Self {
            r: 0.0,
            g: 1.0,
            b: 1.0,
        }
    }
    /// (1, 0, 1)
    pub fn magenta() -> Self {
        Self {
            r: 1.0,
            g: 0.0,
            b: 1.0,
        }
    }
    pub fn gray() -> Self {
        Self {
            r: 0.5,
            g: 0.5,
            b: 0.5,
        }
    }
    pub fn light_gray() -> Self {
        Self {
            r: 0.75,
            g: 0.75,
            b: 0.75,
        }
    }
    pub fn dark_gray() -> Self {
        Self {
            r: 0.25,
            g: 0.25,
            b: 0.25,
        }
    }
}
