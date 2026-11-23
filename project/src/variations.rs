//! Rolls: params set to random values within their range or among their choices, uniform or near
//! the start.

use std::collections::BTreeMap;
use std::f32::consts::TAU;
use std::ops::RangeInclusive;

use anyhow::{bail, Result};
use plottery_runtime::{ProjectParam, ProjectParamValue, ProjectParamsListWrapper};
use rand::seq::IndexedRandom;
use rand::{Rng, RngExt};
use serde::{Deserialize, Serialize};

/// Seeds a roll picks from, the same as the editor's dice button.
pub const ROLL_SEED_RANGE: RangeInclusive<i32> = 100..=999;

/// Values with a range or a fixed set of choices to roll from.
pub fn is_rollable(value: &ProjectParamValue) -> bool {
    use ProjectParamValue as V;
    match value {
        V::FloatRanged { .. }
        | V::IntRanged { .. }
        | V::Percent { .. }
        | V::Seed(_)
        | V::Angle { .. }
        | V::AngleRanged { .. }
        | V::Bool(_)
        | V::Enum(_)
        | V::DataEnum(_)
        | V::Optional(_) => true,
        V::RangeF32 { bounds, .. } => bounds.is_some(),
        V::RangeI32 { bounds, .. } => bounds.is_some(),
        _ => false,
    }
}

/// From and to of a number param in a roll, instead of its `#[range]`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RollRange {
    pub from: f32,
    pub to: f32,
}

/// Roll ranges by the dotted path of their param.
pub type RollRanges = BTreeMap<String, RollRange>;

/// How far a roll goes from the start params.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RollMode {
    /// Anywhere in the range or among the choices.
    #[default]
    Uniform,
    /// Numbers at most `percent` of their range away from the start, everything else kept.
    Similar { percent: f32 },
}

impl RollMode {
    pub fn is_uniform(&self) -> bool {
        *self == RollMode::Uniform
    }

    /// The part of `lo..=hi` a number at `center` rolls in.
    fn window(self, center: f32, lo: f32, hi: f32) -> (f32, f32) {
        match self {
            RollMode::Uniform => (lo, hi),
            RollMode::Similar { percent } => {
                let reach = (hi - lo) * percent / 100.0;
                let center = center.max(lo).min(hi);
                ((center - reach).max(lo), (center + reach).min(hi))
            }
        }
    }

    /// [`Self::window`] for an int, its ends rounded.
    fn window_i32(self, center: i32, lo: i32, hi: i32) -> (i32, i32) {
        match self {
            RollMode::Uniform => (lo, hi),
            similar => {
                let (from, to) = similar.window(center as f32, lo as f32, hi as f32);
                (from.round() as i32, to.round() as i32)
            }
        }
    }
}

/// Numbers a roll can give a range of its own. A float or int without a `#[range]` only rolls with one.
pub fn takes_roll_range(value: &ProjectParamValue) -> bool {
    use ProjectParamValue as V;
    match value {
        V::Float(_) | V::Int(_) | V::FloatRanged { .. } | V::IntRanged { .. } => true,
        V::Optional(optional) => takes_roll_range(&optional.value),
        _ => false,
    }
}

/// The `#[range]` of a number, what it rolls in without a roll range.
pub fn own_roll_range(value: &ProjectParamValue) -> Option<RollRange> {
    use ProjectParamValue as V;
    let (from, to) = match value {
        V::FloatRanged { min, max, .. } => (*min, *max),
        V::IntRanged { min, max, .. } => (*min as f32, *max as f32),
        V::Optional(optional) => return own_roll_range(&optional.value),
        _ => return None,
    };
    Some(RollRange { from, to })
}

/// Rollable as it is or with a roll range.
pub fn can_roll(value: &ProjectParamValue) -> bool {
    is_rollable(value) || takes_roll_range(value)
}

/// Numbers, what a similar roll moves. It keeps everything else as it is.
pub fn is_rollable_number(value: &ProjectParamValue) -> bool {
    use ProjectParamValue as V;
    match value {
        V::Float(_)
        | V::Int(_)
        | V::FloatRanged { .. }
        | V::IntRanged { .. }
        | V::Percent { .. }
        | V::Angle { .. }
        | V::AngleRanged { .. } => true,
        V::RangeF32 { bounds, .. } => bounds.is_some(),
        V::RangeI32 { bounds, .. } => bounds.is_some(),
        V::Optional(optional) => is_rollable_number(&optional.value),
        _ => false,
    }
}

/// Every rollable param as a dotted path, in the order `plottery params` lists them.
pub fn rollable_paths(list: &[ProjectParam]) -> Vec<String> {
    let mut paths = Vec::new();
    push_paths_of_list(list, "", is_rollable, &mut paths);
    paths
}

/// Like [`rollable_paths`], with the numbers that need a roll range.
pub fn can_roll_paths(list: &[ProjectParam]) -> Vec<String> {
    let mut paths = Vec::new();
    push_paths_of_list(list, "", can_roll, &mut paths);
    paths
}

/// Whether `value` or anything in it can be rolled, as it is or with a roll range.
pub fn holds_rollable(value: &ProjectParamValue) -> bool {
    let mut paths = Vec::new();
    push_paths_of_value(value, String::new(), can_roll, &mut paths);
    !paths.is_empty()
}

type Accept = fn(&ProjectParamValue) -> bool;

fn push_paths_of_list(
    list: &[ProjectParam],
    prefix: &str,
    accept: Accept,
    paths: &mut Vec<String>,
) {
    for param in list {
        let path = format!("{prefix}{}", param.name);
        push_paths_of_value(&param.value, path, accept, paths);
    }
}

fn push_paths_of_value(
    value: &ProjectParamValue,
    path: String,
    accept: Accept,
    paths: &mut Vec<String>,
) {
    use ProjectParamValue as V;
    let prefix = format!("{path}.");
    if accept(value) {
        paths.push(path);
    }
    match value {
        V::Struct(param_struct) => push_paths_of_list(&param_struct.fields, &prefix, accept, paths),
        V::DataEnum(data_enum) => {
            push_paths_of_list(data_enum.selected_fields(), &prefix, accept, paths)
        }
        V::Optional(optional) => match optional.value.as_ref() {
            V::Struct(param_struct) => {
                push_paths_of_list(&param_struct.fields, &prefix, accept, paths)
            }
            V::DataEnum(data_enum) => {
                push_paths_of_list(data_enum.selected_fields(), &prefix, accept, paths)
            }
            _ => {}
        },
        V::Vec(param_vec) => {
            for (i, item) in param_vec.items.iter().enumerate() {
                push_paths_of_value(item, format!("{prefix}{i}"), accept, paths);
            }
        }
        _ => {}
    }
}

/// Checks that every path names a param that is rollable, as it is or with its range in `ranges`.
/// Returns them without duplicates, parents before their fields.
pub fn check_rollable(
    params: &ProjectParamsListWrapper,
    paths: &[String],
    ranges: &RollRanges,
) -> Result<Vec<String>> {
    use ProjectParamValue as V;
    let rollable = rollable_paths(&params.list);
    let listing = match rollable.is_empty() {
        true => "nothing in this project".to_string(),
        false => rollable.join(", "),
    };
    if paths.is_empty() {
        bail!("Name the params to roll. Rollable: {listing}");
    }
    let mut probe = params.clone();
    let mut checked: Vec<String> = Vec::new();
    for path in paths {
        let value = probe.value_at_path_mut(path)?;
        let range = ranges.get(path).filter(|_| takes_roll_range(value));
        if let Some(range) = range.filter(|r| !(r.from.is_finite() && r.to.is_finite())) {
            bail!(
                "The range to roll '{path}' in needs two numbers, not {} and {}",
                range.from,
                range.to
            );
        }
        if !is_rollable(value) && range.is_none() {
            let hint = match value {
                V::Float(_) | V::Int(_) | V::RangeF32 { .. } | V::RangeI32 { .. } => {
                    " without a #[range]"
                }
                _ => "",
            };
            bail!(
                "'{path}' is a {}{hint} and can't be rolled. Rollable: {listing}",
                value.type_name()
            );
        }
        if !checked.contains(path) {
            checked.push(path.clone());
        }
    }
    checked.sort_by_key(|path| path.split('.').count());
    Ok(checked)
}

/// Checks that a similar roll has a percentage it can use and a number among `paths` to move.
pub fn check_roll_mode(
    params: &ProjectParamsListWrapper,
    paths: &[String],
    mode: RollMode,
) -> Result<()> {
    let RollMode::Similar { percent } = mode else {
        return Ok(());
    };
    if !(percent > 0.0 && percent <= 100.0) {
        bail!("A similar roll moves numbers by more than 0 and at most 100 % of their range, not {percent}");
    }
    let mut probe = params.clone();
    let moves = paths.iter().any(|path| {
        probe
            .value_at_path_mut(path)
            .is_ok_and(|value| is_rollable_number(value))
    });
    if !moves {
        let mut numbers = Vec::new();
        push_paths_of_list(&params.list, "", is_rollable_number, &mut numbers);
        bail!(
            "A similar roll only moves numbers and keeps the rest as it is. Numbers: {}",
            match numbers.is_empty() {
                true => "none in this project".to_string(),
                false => numbers.join(", "),
            }
        );
    }
    Ok(())
}

/// Gives `value` a random value within its range or among its choices: uniform, or near where it
/// is in a similar roll, which keeps everything but numbers.
pub fn roll_value(value: &mut ProjectParamValue, mode: RollMode, rng: &mut impl Rng) {
    use ProjectParamValue as V;
    let uniform = mode.is_uniform();
    match value {
        V::FloatRanged { val, min, max } | V::Percent { val, min, max } => {
            *val = uniform_f32(rng, mode.window(*val, *min, *max))
        }
        V::IntRanged { val, min, max } => {
            *val = uniform_i32(rng, mode.window_i32(*val, *min, *max))
        }
        V::RangeF32 {
            min,
            max,
            bounds: Some((lo, hi)),
        } => {
            let a = uniform_f32(rng, mode.window(*min, *lo, *hi));
            let b = uniform_f32(rng, mode.window(*max, *lo, *hi));
            (*min, *max) = (a.min(b), a.max(b));
        }
        V::RangeI32 {
            min,
            max,
            bounds: Some((lo, hi)),
        } => {
            let a = uniform_i32(rng, mode.window_i32(*min, *lo, *hi));
            let b = uniform_i32(rng, mode.window_i32(*max, *lo, *hi));
            (*min, *max) = (a.min(b), a.max(b));
        }
        V::Angle { rad, .. } => match mode {
            RollMode::Uniform => *rad = rng.random_range(0.0..TAU),
            RollMode::Similar { percent } => {
                let reach = TAU * percent / 100.0;
                *rad = (*rad + uniform_f32(rng, (-reach, reach))).rem_euclid(TAU)
            }
        },
        V::AngleRanged { rad, min, max, .. } => {
            *rad = uniform_f32(rng, mode.window(*rad, *min, *max))
        }
        V::Seed(seed) if uniform => seed.set(rng.random_range(ROLL_SEED_RANGE)),
        V::Bool(val) if uniform => *val = rng.random_bool(0.5),
        V::Enum(param_enum) if uniform => {
            if let Some(variant) = param_enum.variants.choose(rng) {
                param_enum.selected = variant.clone();
            }
        }
        V::DataEnum(data_enum) if uniform => {
            if let Some(variant) = data_enum.variant_names().choose(rng) {
                data_enum.selected = variant.clone();
            }
        }
        V::Optional(optional) => {
            if uniform {
                optional.enabled = rng.random_bool(0.5);
            }
            if is_rollable(&optional.value) {
                roll_value(&mut optional.value, mode, rng);
            }
        }
        _ => {}
    }
}

/// Gives a number a random value between `range`'s ends, within its `#[range]` if it has one:
/// uniform, or near where it is in a similar roll. A uniform roll of an `Option` also rolls
/// whether it is on.
pub fn roll_value_in(
    value: &mut ProjectParamValue,
    range: RollRange,
    mode: RollMode,
    rng: &mut impl Rng,
) {
    use ProjectParamValue as V;
    let (lo, hi) = (range.from.min(range.to), range.from.max(range.to));
    let int = |x: f32| x.round() as i32;
    match value {
        V::Float(val) => *val = uniform_f32(rng, mode.window(*val, lo, hi)),
        V::FloatRanged { val, min, max } => {
            let (lo, hi) = (lo.max(*min).min(*max), hi.max(*min).min(*max));
            *val = uniform_f32(rng, mode.window(*val, lo, hi))
        }
        V::Int(val) => *val = uniform_i32(rng, mode.window_i32(*val, int(lo), int(hi))),
        V::IntRanged { val, min, max } => {
            let lo = int(lo).max(*min).min(*max);
            let hi = int(hi).max(*min).min(*max);
            *val = uniform_i32(rng, mode.window_i32(*val, lo, hi))
        }
        V::Optional(optional) => {
            if mode.is_uniform() {
                optional.enabled = rng.random_bool(0.5);
            }
            roll_value_in(&mut optional.value, range, mode, rng);
        }
        _ => {}
    }
}

fn uniform_f32(rng: &mut impl Rng, (min, max): (f32, f32)) -> f32 {
    match min < max {
        true => rng.random_range(min..=max),
        false => min,
    }
}

fn uniform_i32(rng: &mut impl Rng, (min, max): (i32, i32)) -> i32 {
    match min < max {
        true => rng.random_range(min..=max),
        false => min,
    }
}

/// `start` with every path rolled, numbers with a range in `ranges` within it. A path that no
/// longer resolves after its parent rolled, like a field of an enum variant that isn't selected
/// anymore, is skipped.
pub fn roll_params(
    start: &ProjectParamsListWrapper,
    paths: &[String],
    ranges: &RollRanges,
    mode: RollMode,
    rng: &mut impl Rng,
) -> ProjectParamsListWrapper {
    let mut params = start.clone();
    for path in paths {
        if let Ok(value) = params.value_at_path_mut(path) {
            match ranges.get(path) {
                Some(range) if takes_roll_range(value) => roll_value_in(value, *range, mode, rng),
                _ if is_rollable(value) => roll_value(value, mode, rng),
                _ => {}
            }
        }
    }
    params
}

/// The start params followed by `count` rolls of `paths` (checked by [`check_rollable`]).
pub fn roll_param_sets(
    start: &ProjectParamsListWrapper,
    paths: &[String],
    ranges: &RollRanges,
    mode: RollMode,
    count: usize,
    rng: &mut impl Rng,
) -> Vec<ProjectParamsListWrapper> {
    std::iter::once(start.clone())
        .chain((0..count).map(|_| roll_params(start, paths, ranges, mode, rng)))
        .collect()
}

/// The values at `paths` as text, for captions. Paths that don't resolve are left out.
pub fn rolled_values(params: &ProjectParamsListWrapper, paths: &[String]) -> Vec<(String, String)> {
    let mut params = params.clone();
    paths
        .iter()
        .filter_map(|path| {
            let value = params.value_at_path_mut(path).ok()?;
            Some((path.clone(), value.to_text()))
        })
        .collect()
}
