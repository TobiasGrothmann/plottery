//! Permutation rolls: a grid over the rolled params, `samples` values per number and every choice
//! of the rest, thinned out by a keep chance that is the same every time.

use std::f32::consts::TAU;

use plottery_lib::fractions_exclusive;
use plottery_runtime::{ProjectParamValue, ProjectParamsListWrapper};

use crate::{
    can_roll_paths, float_bounds, int_bounds, is_rollable, takes_roll_range, RollRange, RollRanges,
};

/// Grid points [`permutation_info`] goes through to count the kept ones, it estimates beyond.
pub const PERMUTATION_SCAN_LIMIT: u128 = 4_000_000;

/// The size of a permutation roll.
#[derive(Debug, Clone, PartialEq)]
pub struct PermutationInfo {
    /// Values per param in grid order, the first changes slowest.
    pub steps: Vec<(String, usize)>,
    /// Different permutations, the start params left out. Saturates.
    pub grid: u128,
    /// Kept permutations, the ones that are rolled.
    pub rolls: u128,
    /// `rolls` is the keep chance of `grid`, the grid is too large to count them.
    pub estimated: bool,
}

impl PermutationInfo {
    /// Like `24 permutations`.
    pub fn summary(&self) -> String {
        let about = if self.estimated { "≈" } else { "" };
        match self.rolls {
            1 => format!("{about}1 permutation"),
            rolls => format!("{about}{rolls} permutations"),
        }
    }
}

/// The size of the permutation roll of `paths`, without building its param sets.
pub fn permutation_info(
    start: &ProjectParamsListWrapper,
    paths: &[String],
    ranges: &RollRanges,
    samples: usize,
    keep: f32,
) -> PermutationInfo {
    let axes = axes(start, paths, ranges, samples);
    let grid = grid_size(&axes);
    let points = axes.iter().fold(1u128, |product, axis| {
        product.saturating_mul(axis.steps.len() as u128)
    });
    let (rolls, estimated) = if keep >= 100.0 {
        (grid, false)
    } else if points <= PERMUTATION_SCAN_LIMIT {
        let mut rolls = 0;
        walk(&axes, keep, |_, _| rolls += 1);
        (rolls, false)
    } else {
        ((grid as f64 * keep as f64 / 100.0).round() as u128, true)
    };
    PermutationInfo {
        steps: steps(&axes),
        grid,
        rolls,
        estimated,
    }
}

/// The kept permutations of `paths` in grid order, the start params left out, and their size.
/// Numbers with a range in `ranges` take their samples within it.
pub fn permutations(
    start: &ProjectParamsListWrapper,
    paths: &[String],
    ranges: &RollRanges,
    samples: usize,
    keep: f32,
) -> (PermutationInfo, Vec<ProjectParamsListWrapper>) {
    let axes = axes(start, paths, ranges, samples);
    let mut sets = Vec::new();
    walk(&axes, keep, |digits, active| {
        let mut params = start.clone();
        for (k, axis) in axes.iter().enumerate().filter(|(k, _)| active[*k]) {
            if let Ok(value) = params.value_at_path_mut(&axis.path) {
                *value = axis.steps[digits[k]].clone();
            }
        }
        sets.push(params);
    });
    let info = PermutationInfo {
        steps: steps(&axes),
        grid: grid_size(&axes),
        rolls: sets.len() as u128,
        estimated: false,
    };
    (info, sets)
}

fn steps(axes: &[Axis]) -> Vec<(String, usize)> {
    axes.iter()
        .map(|axis| (axis.path.clone(), axis.steps.len()))
        .collect()
}

/// One rolled param of the grid.
struct Axis {
    path: String,
    /// Whole values, the start value changed once.
    steps: Vec<ProjectParamValue>,
    /// Per step: whether the fields under the param are in use, like those of the variant the
    /// start params have or of an option that is on. Fields of other variants can't be rolled.
    shows: Vec<bool>,
    /// The closest axis above this one.
    parent: Option<usize>,
    /// The step that is the start value.
    start: Option<usize>,
}

/// The axes of `paths` in the order the params are listed, parents before their fields, so the
/// order `paths` come in doesn't change the grid.
fn axes(
    start: &ProjectParamsListWrapper,
    paths: &[String],
    ranges: &RollRanges,
    samples: usize,
) -> Vec<Axis> {
    let mut probe = start.clone();
    let mut axes: Vec<Axis> = Vec::new();
    for path in can_roll_paths(&start.list) {
        if !paths.contains(&path) {
            continue;
        }
        let Ok(value) = probe.value_at_path_mut(&path).cloned() else {
            continue;
        };
        let parent = axes
            .iter()
            .rposition(|axis| path.starts_with(&format!("{}.", axis.path)));
        let range = ranges
            .get(&path)
            .copied()
            .filter(|_| takes_roll_range(&value));
        let steps = match under_an_option_that_stays_off(&mut probe, &path, &axes) {
            true => vec![value.clone()],
            false => steps_of(&value, range, samples),
        };
        axes.push(Axis {
            shows: steps
                .iter()
                .map(|step| shows_fields(&value, step))
                .collect(),
            start: steps.iter().position(|step| *step == value),
            steps,
            parent,
            path,
        });
    }
    axes
}

/// Whether an option above `path` is off and not rolled, so `path` makes no difference.
fn under_an_option_that_stays_off(
    probe: &mut ProjectParamsListWrapper,
    path: &str,
    axes: &[Axis],
) -> bool {
    let segments: Vec<&str> = path.split('.').collect();
    (1..segments.len()).any(|end| {
        let prefix = segments[..end].join(".");
        !axes.iter().any(|axis| axis.path == prefix)
            && matches!(
                probe.value_at_path_mut(&prefix),
                Ok(ProjectParamValue::Optional(optional)) if !optional.enabled
            )
    })
}

/// Whether the fields under `start` are in use when it is `step`.
fn shows_fields(start: &ProjectParamValue, step: &ProjectParamValue) -> bool {
    use ProjectParamValue as V;
    match (start, step) {
        (V::DataEnum(start), V::DataEnum(step)) => step.selected == start.selected,
        (V::Optional(start), V::Optional(step)) => {
            step.enabled && shows_fields(&start.value, &step.value)
        }
        _ => true,
    }
}

/// The values `value` takes in the grid. Only itself when it can't be rolled.
fn steps_of(
    value: &ProjectParamValue,
    range: Option<RollRange>,
    samples: usize,
) -> Vec<ProjectParamValue> {
    use ProjectParamValue as V;
    match value {
        V::Float(_) | V::FloatRanged { .. } | V::Percent { .. } => {
            let Some((lo, hi)) = float_bounds(value, range) else {
                return vec![value.clone()];
            };
            varied(value, floats(lo, hi, samples), |step, x| {
                if let V::Float(val) | V::FloatRanged { val, .. } | V::Percent { val, .. } = step {
                    *val = x;
                }
            })
        }
        V::Int(_) | V::IntRanged { .. } => {
            let Some((lo, hi)) = int_bounds(value, range) else {
                return vec![value.clone()];
            };
            varied(value, ints(lo, hi, samples), |step, x| {
                if let V::Int(val) | V::IntRanged { val, .. } = step {
                    *val = x;
                }
            })
        }
        V::AngleRanged { min, max, .. } => varied(value, floats(*min, *max, samples), set_angle),
        V::Angle { .. } => varied(
            value,
            fractions_exclusive(samples).map(|t| t * TAU),
            set_angle,
        ),
        V::RangeF32 {
            bounds: Some((lo, hi)),
            ..
        } => varied(value, pairs(&floats(*lo, *hi, samples)), |step, (a, b)| {
            if let V::RangeF32 { min, max, .. } = step {
                (*min, *max) = (a, b);
            }
        }),
        V::RangeI32 {
            bounds: Some((lo, hi)),
            ..
        } => varied(value, pairs(&ints(*lo, *hi, samples)), |step, (a, b)| {
            if let V::RangeI32 { min, max, .. } = step {
                (*min, *max) = (a, b);
            }
        }),
        V::Bool(_) => varied(value, [false, true], |step, on| *step = V::Bool(on)),
        V::Enum(param_enum) => varied(value, param_enum.variants.clone(), |step, variant| {
            if let V::Enum(param_enum) = step {
                param_enum.selected = variant;
            }
        }),
        V::DataEnum(data_enum) => varied(value, data_enum.variant_names(), |step, variant| {
            if let V::DataEnum(data_enum) = step {
                data_enum.selected = variant;
            }
        }),
        V::Seed(seed) => {
            let seeds = (0..samples).map(|i| seed.current.wrapping_add(i as i32));
            varied(value, seeds, |step, new| {
                if let V::Seed(seed) = step {
                    seed.set(new);
                }
            })
        }
        V::Optional(optional) => {
            let range = range.filter(|_| takes_roll_range(&optional.value));
            let inner = match is_rollable(&optional.value) || range.is_some() {
                true => steps_of(&optional.value, range, samples),
                false => vec![optional.value.as_ref().clone()],
            };
            let off = std::iter::once((false, optional.value.as_ref().clone()));
            let on = inner.into_iter().map(|inner| (true, inner));
            varied(value, off.chain(on), |step, (enabled, inner)| {
                if let V::Optional(optional) = step {
                    optional.enabled = enabled;
                    *optional.value = inner;
                }
            })
        }
        _ => vec![value.clone()],
    }
}

/// `value` changed by `set` to each of `items`.
fn varied<T>(
    value: &ProjectParamValue,
    items: impl IntoIterator<Item = T>,
    set: impl Fn(&mut ProjectParamValue, T),
) -> Vec<ProjectParamValue> {
    items
        .into_iter()
        .map(|item| {
            let mut step = value.clone();
            set(&mut step, item);
            step
        })
        .collect()
}

fn set_angle(step: &mut ProjectParamValue, x: f32) {
    if let ProjectParamValue::Angle { rad, .. } | ProjectParamValue::AngleRanged { rad, .. } = step
    {
        *rad = x;
    }
}

/// `samples` evenly spaced floats from `lo` to `hi`, both included, without repeats. Computed in
/// f64, so steps like 1, 14, 27, 40 come out as they are.
fn floats(lo: f32, hi: f32, samples: usize) -> Vec<f32> {
    let (lo, hi) = (lo.min(hi), lo.max(hi));
    if !lo.is_finite() || !hi.is_finite() || lo == hi || samples < 2 {
        return vec![lo];
    }
    let mut values: Vec<f32> = spaced(lo as f64, hi as f64, samples)
        .map(|x| (x as f32).clamp(lo, hi))
        .collect();
    values.dedup();
    values
}

/// [`floats`] rounded to ints.
fn ints(lo: i32, hi: i32, samples: usize) -> Vec<i32> {
    let (lo, hi) = (lo.min(hi), lo.max(hi));
    if lo == hi || samples < 2 {
        return vec![lo];
    }
    let mut values: Vec<i32> = spaced(lo as f64, hi as f64, samples)
        .map(|x| (x.round() as i32).clamp(lo, hi))
        .collect();
    values.dedup();
    values
}

fn spaced(lo: f64, hi: f64, samples: usize) -> impl Iterator<Item = f64> {
    let last = (samples - 1) as f64;
    (0..samples).map(move |i| lo + (hi - lo) * i as f64 / last)
}

/// Every range from one end to a later one. Only the one empty range when there is one end.
fn pairs<T: Copy>(ends: &[T]) -> Vec<(T, T)> {
    if let [only] = ends {
        return vec![(*only, *only)];
    }
    ends.iter()
        .enumerate()
        .flat_map(|(i, a)| ends[i + 1..].iter().map(move |b| (*a, *b)))
        .collect()
}

/// Goes through the grid, the last axis fastest, and calls `on_kept` with the steps of each kept
/// permutation and which axes are in use there.
fn walk(axes: &[Axis], keep: f32, mut on_kept: impl FnMut(&[usize], &[bool])) {
    let threshold = keep as f64 / 100.0;
    let mut digits = vec![0; axes.len()];
    let mut active = vec![true; axes.len()];
    let mut index: u64 = 0;
    loop {
        if is_new(axes, &digits, &mut active) && chance(index) < threshold {
            on_kept(&digits, &active);
        }
        if !advance(axes, &mut digits) {
            return;
        }
        index = index.wrapping_add(1);
    }
}

/// Whether the point at `digits` is another drawing than the start params and every point before
/// it. Fills in which axes are in use there; a field of a variant that isn't selected or of an
/// option that is off makes no difference, only its first step counts.
fn is_new(axes: &[Axis], digits: &[usize], active: &mut [bool]) -> bool {
    let mut at_start = true;
    for (k, axis) in axes.iter().enumerate() {
        active[k] = axis
            .parent
            .is_none_or(|parent| active[parent] && axes[parent].shows[digits[parent]]);
        if !active[k] {
            if digits[k] != 0 {
                return false;
            }
        } else if axis.start != Some(digits[k]) {
            at_start = false;
        }
    }
    !at_start
}

/// The next point, `false` after the last.
fn advance(axes: &[Axis], digits: &mut [usize]) -> bool {
    for k in (0..axes.len()).rev() {
        digits[k] += 1;
        if digits[k] < axes[k].steps.len() {
            return true;
        }
        digits[k] = 0;
    }
    false
}

/// The points [`is_new`] lets through, counted without going through them.
fn grid_size(axes: &[Axis]) -> u128 {
    let children: Vec<Vec<usize>> = (0..axes.len())
        .map(|k| {
            (0..axes.len())
                .filter(|c| axes[*c].parent == Some(k))
                .collect()
        })
        .collect();
    let roots: Vec<usize> = (0..axes.len())
        .filter(|k| axes[*k].parent.is_none())
        .collect();
    // different values of an axis and everything under it
    fn subtree(k: usize, axes: &[Axis], children: &[Vec<usize>]) -> u128 {
        let below = children[k].iter().fold(1u128, |product, c| {
            product.saturating_mul(subtree(*c, axes, children))
        });
        axes[k].shows.iter().fold(0u128, |sum, shows| {
            sum.saturating_add(if *shows { below } else { 1 })
        })
    }
    // whether the start params are among those values
    fn has_start(k: usize, axes: &[Axis], children: &[Vec<usize>]) -> bool {
        axes[k].start.is_some_and(|start| {
            !axes[k].shows[start] || children[k].iter().all(|c| has_start(*c, axes, children))
        })
    }
    let all = roots.iter().fold(1u128, |product, k| {
        product.saturating_mul(subtree(*k, axes, &children))
    });
    match roots.iter().all(|k| has_start(*k, axes, &children)) {
        true => all.saturating_sub(1),
        false => all,
    }
}

/// A number in `0..1` for grid point `index`, always the same: splitmix64, independent of `rand`.
fn chance(index: u64) -> f64 {
    let mut z = index.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z >> 11) as f64 / (1u64 << 53) as f64
}
