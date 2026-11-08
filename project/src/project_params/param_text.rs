//! Text form of param values for the CLI: `plottery params` prints it, `--set` reads it.

use std::str::FromStr;

use anyhow::{anyhow, bail, Result};
use plottery_lib::{ColorRgb, Margin, Paper, PaperSize, V2i, V2};

use super::angle_unit::AngleUnit;
use super::project_param::ProjectParam;
use super::project_param_value::ProjectParamValue;
use super::project_params_list_wrapper::ProjectParamsListWrapper;

impl ProjectParamValue {
    /// The value in the syntax [`ProjectParamValue::set_from_text`] reads. Values that can't be
    /// set from text fall back to [`ProjectParamValue::value_as_string`].
    pub fn to_text(&self) -> String {
        match self {
            Self::Float(val) | Self::FloatRanged { val, .. } => val.to_string(),
            Self::Int(val) | Self::IntRanged { val, .. } => val.to_string(),
            Self::Bool(val) => val.to_string(),
            Self::Enum(param_enum) => param_enum.selected.clone(),
            Self::String(val) => val.clone(),
            Self::Angle { rad, unit } => format!(
                "{}{}",
                short_float(unit.from_angle(plottery_lib::Angle::from_rad(*rad))),
                unit.short_name()
            ),
            Self::V2(v) => format!("{},{}", v.x, v.y),
            Self::V2i(v) => format!("{},{}", v.x, v.y),
            Self::RangeF32 { min, max, .. } => format!("{min}..{max}"),
            Self::RangeI32 { min, max, .. } => format!("{min}..{max}"),
            Self::Color(color) => color.hex(),
            Self::Paper(paper) => paper_to_text(paper),
            Self::Seed(seed) => seed.current.to_string(),
            Self::Image(file) | Self::Svg(file) => {
                file.path.clone().unwrap_or_else(|| "none".to_string())
            }
            Self::Optional(optional) => match (optional.enabled, optional.value.as_ref()) {
                (false, _) => "off".to_string(),
                (true, Self::Struct(_)) => "on".to_string(),
                (true, inner) => inner.to_text(),
            },
            Self::Struct(_)
            | Self::Vec(_)
            | Self::Noise(_)
            | Self::Easing(_)
            | Self::Curve2D(_)
            | Self::Curve2DNorm(_) => self.value_as_string(),
        }
    }

    /// Sets the value from its [`ProjectParamValue::to_text`] form. Out of range values are errors.
    pub fn set_from_text(&mut self, text: &str) -> Result<()> {
        match self {
            Self::Float(val) => *val = parse(text)?,
            Self::FloatRanged { val, min, max } => *val = parse_in(text, *min, *max)?,
            Self::Int(val) => *val = parse(text)?,
            Self::IntRanged { val, min, max } => *val = parse_in(text, *min, *max)?,
            Self::Bool(val) => *val = parse(text)?,
            Self::Enum(param_enum) => {
                if !param_enum.variants.iter().any(|v| v == text) {
                    bail!(
                        "'{text}' is not a variant, use one of {}",
                        param_enum.variants.join(" | ")
                    );
                }
                param_enum.selected = text.to_string();
            }
            Self::String(val) => *val = text.to_string(),
            Self::Angle { rad, unit } => *rad = parse_angle(text, *unit)?,
            Self::V2(v) => {
                let (x, y) = parse_pair(text, ",")?;
                *v = V2::new(x, y);
            }
            Self::V2i(v) => {
                let (x, y) = parse_pair(text, ",")?;
                *v = V2i::new(x, y);
            }
            Self::RangeF32 { min, max, bounds } => (*min, *max) = parse_range(text, *bounds)?,
            Self::RangeI32 { min, max, bounds } => (*min, *max) = parse_range(text, *bounds)?,
            Self::Color(color) => {
                *color = ColorRgb::from_hex(text)
                    .ok_or_else(|| anyhow!("'{text}' is not a color, use #rrggbb"))?
            }
            Self::Paper(paper) => *paper = parse_paper(text, paper.margin)?,
            Self::Seed(seed) => seed.set(parse(text)?),
            Self::Image(file) | Self::Svg(file) => {
                file.set((text != "none").then(|| text.to_string()))
            }
            Self::Optional(optional) => match text {
                "off" => optional.enabled = false,
                "on" => optional.enabled = true,
                _ if matches!(optional.value.as_ref(), Self::Struct(_)) => {
                    bail!("use on or off, set its fields as <name>.<field>")
                }
                _ => {
                    optional.value.set_from_text(text)?;
                    optional.enabled = true;
                }
            },
            Self::Struct(_) => bail!("is a struct, set its fields as <name>.<field>"),
            Self::Vec(_) => bail!("is a list, set its items as <name>.0, <name>.1, ..."),
            Self::Noise(_) | Self::Easing(_) | Self::Curve2D(_) | Self::Curve2DNorm(_) => {
                bail!("{} params are not settable from the CLI", self.type_name())
            }
        }
        Ok(())
    }
}

impl ProjectParamsListWrapper {
    /// Sets the param at a dotted path like `size`, `glow.radius` or `points.0` from text.
    pub fn set(&mut self, path: &str, text: &str) -> Result<()> {
        let value = value_at_path(&mut self.list, path)?;
        value
            .set_from_text(text)
            .map_err(|e| anyhow!("{path}: {e}"))
    }
}

fn value_at_path<'a>(
    list: &'a mut [ProjectParam],
    path: &str,
) -> Result<&'a mut ProjectParamValue> {
    let mut segments = path.split('.');
    let first = segments.next().unwrap_or_default();
    let mut value = field_mut(list, first, None)?;
    let mut walked = first.to_string();
    for segment in segments {
        value = match value {
            ProjectParamValue::Struct(param_struct) => {
                field_mut(&mut param_struct.fields, segment, Some(&walked))?
            }
            ProjectParamValue::Optional(optional) => match optional.value.as_mut() {
                ProjectParamValue::Struct(param_struct) => {
                    field_mut(&mut param_struct.fields, segment, Some(&walked))?
                }
                inner => bail!(
                    "'{walked}' is an option<{}> without fields",
                    inner.type_name()
                ),
            },
            ProjectParamValue::Vec(param_vec) => {
                let len = param_vec.items.len();
                segment
                    .parse::<usize>()
                    .ok()
                    .and_then(|i| param_vec.items.get_mut(i))
                    .ok_or_else(|| match len {
                        0 => anyhow!("'{walked}' has no items"),
                        _ => anyhow!(
                            "'{walked}' has {len} items, use an index from 0 to {}",
                            len - 1
                        ),
                    })?
            }
            other => bail!("'{walked}' is a {} without fields", other.type_name()),
        };
        walked = format!("{walked}.{segment}");
    }
    Ok(value)
}

fn field_mut<'a>(
    list: &'a mut [ProjectParam],
    name: &str,
    parent: Option<&str>,
) -> Result<&'a mut ProjectParamValue> {
    match list.iter().position(|param| param.name == name) {
        Some(i) => Ok(&mut list[i].value),
        None => {
            let names: Vec<&str> = list.iter().map(|param| param.name.as_str()).collect();
            match parent {
                Some(parent) => bail!(
                    "'{parent}' has no field '{name}', fields are: {}",
                    names.join(", ")
                ),
                None => bail!("unknown param '{name}', params are: {}", names.join(", ")),
            }
        }
    }
}

fn parse<T: FromStr>(text: &str) -> Result<T> {
    text.trim().parse().map_err(|_| {
        anyhow!(
            "'{text}' is not a valid {}",
            std::any::type_name::<T>()
                .rsplit("::")
                .next()
                .unwrap_or("value")
        )
    })
}

fn parse_in<T: FromStr + PartialOrd + std::fmt::Display>(text: &str, min: T, max: T) -> Result<T> {
    let value = parse(text)?;
    if value < min || value > max {
        bail!("{value} is outside {min}..{max}");
    }
    Ok(value)
}

fn parse_pair<T: FromStr>(text: &str, separator: &str) -> Result<(T, T)> {
    let (a, b) = text
        .split_once(separator)
        .ok_or_else(|| anyhow!("'{text}' needs two values like 1{separator}2"))?;
    Ok((parse(a)?, parse(b)?))
}

fn parse_range<T: FromStr + PartialOrd + Copy + std::fmt::Display>(
    text: &str,
    bounds: Option<(T, T)>,
) -> Result<(T, T)> {
    let (min, max) = match bounds {
        Some((low, high)) => {
            let (min, max) = text
                .split_once("..")
                .ok_or_else(|| anyhow!("'{text}' needs two values like 1..2"))?;
            (parse_in(min, low, high)?, parse_in(max, low, high)?)
        }
        None => parse_pair(text, "..")?,
    };
    if min > max {
        bail!("{min} is larger than {max}");
    }
    Ok((min, max))
}

/// Radians from `45deg`, `0.25rot` or `1.2rad`; a plain number is in `unit`.
fn parse_angle(text: &str, unit: AngleUnit) -> Result<f32> {
    let text = text.trim();
    let (number, unit) = AngleUnit::ALL
        .iter()
        .find_map(|u| text.strip_suffix(u.short_name()).map(|n| (n, *u)))
        .unwrap_or((text, unit));
    let value: f32 = parse(number)
        .map_err(|_| anyhow!("'{text}' is not an angle, use e.g. 45deg, 0.25rot or 1.2rad"))?;
    Ok(unit.to_angle(value).to_rad())
}

fn paper_to_text(paper: &Paper) -> String {
    let size = match paper.size {
        PaperSize::DinA(number) => format!("A{number}"),
        PaperSize::DinB(number) => format!("B{number}"),
        PaperSize::Custom(size) => format!("{}x{}", size.x, size.y),
    };
    if paper.landscape {
        format!("{size} landscape")
    } else {
        size
    }
}

/// `A4`, `B3 landscape` or `30x40` in cm, keeping `margin`.
fn parse_paper(text: &str, margin: Margin) -> Result<Paper> {
    let usage = || anyhow!("'{text}' is not a paper, use e.g. A4, A3 landscape or 30x40 (cm)");
    let mut words = text.split_whitespace().map(str::to_lowercase);
    let size_word = words.next().ok_or_else(usage)?;
    let din_number = |n: &str| n.parse::<u8>().ok().filter(|n| *n <= 10);
    let size = if let Some(n) = size_word.strip_prefix('a').and_then(din_number) {
        PaperSize::DinA(n)
    } else if let Some(n) = size_word.strip_prefix('b').and_then(din_number) {
        PaperSize::DinB(n)
    } else {
        let (x, y) = parse_pair::<f32>(&size_word, "x").map_err(|_| usage())?;
        if x <= 0.0 || y <= 0.0 {
            return Err(usage());
        }
        PaperSize::Custom(V2::new(x, y))
    };
    let landscape = match words.next().as_deref() {
        None | Some("portrait") => false,
        Some("landscape") => true,
        Some(_) => return Err(usage()),
    };
    if words.next().is_some() {
        return Err(usage());
    }
    Ok(Paper::new(size, landscape, margin))
}

/// At most four decimals, so unit conversions don't print as `45.000004`.
fn short_float(value: f32) -> String {
    let text = format!("{value:.4}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    match text {
        "-0" => "0".to_string(),
        _ => text.to_string(),
    }
}
