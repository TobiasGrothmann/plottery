use serde::{Deserialize, Serialize};

use crate::{Angle, LARGE_EPSILON};

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
    /// Converts this RGB color to the perceptual Oklab format.
    pub fn oklab(&self) -> ColorOklab {
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
    /// Parses `#rrggbb` or `rrggbb`.
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim().trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        let channel = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
        Some(Self::new(
            channel(0)? as f32 / 255.0,
            channel(2)? as f32 / 255.0,
            channel(4)? as f32 / 255.0,
        ))
    }

    pub fn hex(&self) -> String {
        format!(
            "#{:02x}{:02x}{:02x}",
            (self.r.clamp(0.0, 1.0) * 255.0) as u8,
            (self.g.clamp(0.0, 1.0) * 255.0) as u8,
            (self.b.clamp(0.0, 1.0) * 255.0) as u8
        )
    }

    /// Plain channel average of the encoded values. For perceived lightness use [`ColorRgb::lightness`].
    pub fn brightness(&self) -> f32 {
        (self.r.clamp(0.0, 1.0) + self.g.clamp(0.0, 1.0) + self.b.clamp(0.0, 1.0)) / 3.0
    }

    /// Perceived lightness 0..=1, see [`ColorOklab`].
    pub fn lightness(&self) -> f32 {
        ColorOklab::from(*self).l
    }

    /// Physical luminance Y 0..=1 (Rec. 709 weights on linearized sRGB).
    pub fn luminance(&self) -> f32 {
        let (r, g, b) = self.linear();
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// Clamped channels converted from sRGB to linear light.
    fn linear(&self) -> (f32, f32, f32) {
        (
            srgb_to_linear(self.r.clamp(0.0, 1.0)),
            srgb_to_linear(self.g.clamp(0.0, 1.0)),
            srgb_to_linear(self.b.clamp(0.0, 1.0)),
        )
    }

    /// Linear interpolation towards `other`, `t` from 0 to 1.
    pub fn lerp(&self, other: ColorRgb, t: f32) -> ColorRgb {
        ColorRgb::new(
            self.r + (other.r - self.r) * t,
            self.g + (other.g - self.g) * t,
            self.b + (other.b - self.b) * t,
        )
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

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// Perceptual color: `l` lightness 0..=1, `a` green-red, `b` blue-yellow (roughly -0.4..=0.4).
///
/// Distances and interpolation in this space match how different the colors look.
///
/// ### Example
/// ```
/// # use plottery_lib::*;
/// let lab = ColorRgb::red().oklab();
/// let half_way = lab.lerp(ColorRgb::blue().oklab(), 0.5).rgb();
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct ColorOklab {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}

impl ColorOklab {
    pub fn new(l: f32, a: f32, b: f32) -> Self {
        Self { l, a, b }
    }

    /// Converts this Oklab color to RGB format.
    pub fn rgb(&self) -> ColorRgb {
        (*self).into()
    }

    /// Colorfulness, 0 for grays.
    pub fn chroma(&self) -> f32 {
        (self.a * self.a + self.b * self.b).sqrt()
    }

    pub fn hue(&self) -> Angle {
        Angle::from_rad(self.b.atan2(self.a))
    }

    /// Perceptual distance (ΔE in Oklab), unlike [`ColorRgb::dist_euclidean`].
    pub fn dist(&self, other: ColorOklab) -> f32 {
        let dl = self.l - other.l;
        let da = self.a - other.a;
        let db = self.b - other.b;
        (dl * dl + da * da + db * db).sqrt()
    }

    /// Perceptually even interpolation, `t` from 0 to 1.
    #[must_use]
    pub fn lerp(&self, other: ColorOklab, t: f32) -> ColorOklab {
        ColorOklab::new(
            self.l + (other.l - self.l) * t,
            self.a + (other.a - self.a) * t,
            self.b + (other.b - self.b) * t,
        )
    }
}

impl From<ColorRgb> for ColorOklab {
    fn from(rgb: ColorRgb) -> Self {
        let (r, g, b) = rgb.linear();
        let l = (0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b).cbrt();
        let m = (0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b).cbrt();
        let s = (0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b).cbrt();
        Self {
            l: 0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
            a: 1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
            b: 0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
        }
    }
}
impl From<&ColorRgb> for ColorOklab {
    fn from(rgb: &ColorRgb) -> Self {
        ColorOklab::from(*rgb)
    }
}

impl From<ColorOklab> for ColorRgb {
    fn from(lab: ColorOklab) -> Self {
        let l = lab.l + 0.396_337_78 * lab.a + 0.215_803_76 * lab.b;
        let m = lab.l - 0.105_561_346 * lab.a - 0.063_854_17 * lab.b;
        let s = lab.l - 0.089_484_18 * lab.a - 1.291_485_5 * lab.b;
        let (l, m, s) = (l * l * l, m * m * m, s * s * s);
        let to_channel = |linear: f32| linear_to_srgb(linear.clamp(0.0, 1.0)).clamp(0.0, 1.0);
        Self {
            r: to_channel(4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_94 * s),
            g: to_channel(-1.268_438 * l + 2.609_757_4 * m - 0.341_319_38 * s),
            b: to_channel(-0.0041960863 * l - 0.703_418_6 * m + 1.707_614_7 * s),
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

impl PartialEq for ColorOklab {
    fn eq(&self, other: &Self) -> bool {
        (self.l - other.l).abs() < LARGE_EPSILON
            && (self.a - other.a).abs() < LARGE_EPSILON
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
