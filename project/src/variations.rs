//! Rolls: params set to uniform random values within their range or among their choices.

use std::f32::consts::TAU;
use std::ops::RangeInclusive;

use anyhow::{bail, Result};
use plottery_runtime::{ProjectParam, ProjectParamValue, ProjectParamsListWrapper};
use rand::seq::IndexedRandom;
use rand::{Rng, RngExt};

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

/// Every rollable param as a dotted path, in the order `plottery params` lists them.
pub fn rollable_paths(list: &[ProjectParam]) -> Vec<String> {
    let mut paths = Vec::new();
    push_rollable_list(list, "", &mut paths);
    paths
}

fn push_rollable_list(list: &[ProjectParam], prefix: &str, paths: &mut Vec<String>) {
    for param in list {
        push_rollable_value(&param.value, format!("{prefix}{}", param.name), paths);
    }
}

fn push_rollable_value(value: &ProjectParamValue, path: String, paths: &mut Vec<String>) {
    use ProjectParamValue as V;
    let prefix = format!("{path}.");
    if is_rollable(value) {
        paths.push(path);
    }
    match value {
        V::Struct(param_struct) => push_rollable_list(&param_struct.fields, &prefix, paths),
        V::DataEnum(data_enum) => push_rollable_list(data_enum.selected_fields(), &prefix, paths),
        V::Optional(optional) => match optional.value.as_ref() {
            V::Struct(param_struct) => push_rollable_list(&param_struct.fields, &prefix, paths),
            V::DataEnum(data_enum) => {
                push_rollable_list(data_enum.selected_fields(), &prefix, paths)
            }
            _ => {}
        },
        V::Vec(param_vec) => {
            for (i, item) in param_vec.items.iter().enumerate() {
                push_rollable_value(item, format!("{prefix}{i}"), paths);
            }
        }
        _ => {}
    }
}

/// Checks that every path names a rollable param. Returns them without duplicates, parents before their fields.
pub fn check_rollable(params: &ProjectParamsListWrapper, paths: &[String]) -> Result<Vec<String>> {
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
        if !is_rollable(value) {
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

/// Gives `value` a uniform random value within its range or among its choices.
pub fn roll_value(value: &mut ProjectParamValue, rng: &mut impl Rng) {
    use ProjectParamValue as V;
    match value {
        V::FloatRanged { val, min, max } | V::Percent { val, min, max } => {
            *val = uniform_f32(rng, *min, *max)
        }
        V::IntRanged { val, min, max } => *val = uniform_i32(rng, *min, *max),
        V::RangeF32 {
            min,
            max,
            bounds: Some((lo, hi)),
        } => {
            let (a, b) = (uniform_f32(rng, *lo, *hi), uniform_f32(rng, *lo, *hi));
            (*min, *max) = (a.min(b), a.max(b));
        }
        V::RangeI32 {
            min,
            max,
            bounds: Some((lo, hi)),
        } => {
            let (a, b) = (uniform_i32(rng, *lo, *hi), uniform_i32(rng, *lo, *hi));
            (*min, *max) = (a.min(b), a.max(b));
        }
        V::Seed(seed) => seed.set(rng.random_range(ROLL_SEED_RANGE)),
        V::Angle { rad, .. } => *rad = rng.random_range(0.0..TAU),
        V::AngleRanged { rad, min, max, .. } => *rad = uniform_f32(rng, *min, *max),
        V::Bool(val) => *val = rng.random_bool(0.5),
        V::Enum(param_enum) => {
            if let Some(variant) = param_enum.variants.choose(rng) {
                param_enum.selected = variant.clone();
            }
        }
        V::DataEnum(data_enum) => {
            if let Some(variant) = data_enum.variant_names().choose(rng) {
                data_enum.selected = variant.clone();
            }
        }
        V::Optional(optional) => {
            optional.enabled = rng.random_bool(0.5);
            if is_rollable(&optional.value) {
                roll_value(&mut optional.value, rng);
            }
        }
        _ => {}
    }
}

fn uniform_f32(rng: &mut impl Rng, min: f32, max: f32) -> f32 {
    match min < max {
        true => rng.random_range(min..=max),
        false => min,
    }
}

fn uniform_i32(rng: &mut impl Rng, min: i32, max: i32) -> i32 {
    match min < max {
        true => rng.random_range(min..=max),
        false => min,
    }
}

/// `start` with every path rolled. A path that no longer resolves after its parent rolled, like a
/// field of an enum variant that isn't selected anymore, is skipped.
pub fn roll_params(
    start: &ProjectParamsListWrapper,
    paths: &[String],
    rng: &mut impl Rng,
) -> ProjectParamsListWrapper {
    let mut params = start.clone();
    for path in paths {
        if let Ok(value) = params.value_at_path_mut(path) {
            if is_rollable(value) {
                roll_value(value, rng);
            }
        }
    }
    params
}

/// The start params followed by `count` rolls of `paths` (checked by [`check_rollable`]).
pub fn roll_param_sets(
    start: &ProjectParamsListWrapper,
    paths: &[String],
    count: usize,
    rng: &mut impl Rng,
) -> Vec<ProjectParamsListWrapper> {
    std::iter::once(start.clone())
        .chain((0..count).map(|_| roll_params(start, paths, rng)))
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
