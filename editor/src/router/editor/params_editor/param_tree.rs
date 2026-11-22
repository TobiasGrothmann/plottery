//! Paths into the params: the param's name, then per level a field name, a Vec's `[index]`, or
//! an enum's variant name followed by a field name. `Optional` takes no segment.

use plottery_runtime::{project_param::ProjectParam, project_param_value::ProjectParamValue};

/// Unwraps `Optional` so widgets can edit the inner leaf value.
pub fn leaf_value_mut(value: &mut ProjectParamValue) -> &mut ProjectParamValue {
    match value {
        ProjectParamValue::Optional(optional) => optional.value.as_mut(),
        other => other,
    }
}

pub fn get_param_value_by_path(
    list: &[ProjectParam],
    path: &[String],
) -> Option<ProjectParamValue> {
    walk(list, path, false).map(|(_, value)| value.clone())
}

/// The default of the leaf at `path`, `Optional` unwrapped. Vec items past the defaults' end use the item prototype.
pub fn get_default_leaf_value(
    defaults: &[ProjectParam],
    path: &[String],
) -> Option<ProjectParamValue> {
    walk(defaults, path, true)
        .map(|(_, value)| match value {
            ProjectParamValue::Optional(optional) => optional.value.as_ref(),
            other => other,
        })
        .cloned()
}

/// `#[no_pick]` of the innermost param on `path`; a Vec item has the flag of its Vec.
pub fn no_pick_at(list: &[ProjectParam], path: &[String]) -> bool {
    walk(list, path, true).is_some_and(|(param, _)| param.no_pick)
}

fn field<'a>(fields: &'a [ProjectParam], name: &str) -> Option<&'a ProjectParam> {
    fields.iter().find(|param| param.name == name)
}

fn field_mut<'a>(fields: &'a mut [ProjectParam], name: &str) -> Option<&'a mut ProjectParam> {
    fields.iter_mut().find(|param| param.name == name)
}

/// The value at `path` and the innermost param on the way to it. With `prototype_fallback`, a Vec
/// item past the end is the item prototype.
fn walk<'a>(
    list: &'a [ProjectParam],
    path: &[String],
    prototype_fallback: bool,
) -> Option<(&'a ProjectParam, &'a ProjectParamValue)> {
    let (name, mut rest) = path.split_first()?;
    let mut param = field(list, name)?;
    let mut value = &param.value;
    while let Some((segment, tail)) = rest.split_first() {
        match value {
            ProjectParamValue::Optional(optional) => {
                value = &optional.value;
                continue;
            }
            ProjectParamValue::Struct(param_struct) => {
                param = field(&param_struct.fields, segment)?;
                value = &param.value;
                rest = tail;
            }
            ProjectParamValue::Vec(vec_value) => {
                let index = parse_index_segment(segment)?;
                value = match vec_value.items.get(index) {
                    Some(item) => item,
                    None if prototype_fallback => &vec_value.item_prototype,
                    None => return None,
                };
                rest = tail;
            }
            ProjectParamValue::DataEnum(data_enum) => {
                let (name, tail) = tail.split_first()?;
                param = field(&data_enum.variant(segment)?.fields, name)?;
                value = &param.value;
                rest = tail;
            }
            _ => return None,
        }
    }
    Some((param, value))
}

pub fn get_param_value_mut_by_path<'a>(
    list: &'a mut [ProjectParam],
    path: &[String],
) -> Option<&'a mut ProjectParamValue> {
    let (name, mut rest) = path.split_first()?;
    let mut value = &mut field_mut(list, name)?.value;
    while let Some((segment, tail)) = rest.split_first() {
        (value, rest) = match value {
            ProjectParamValue::Optional(optional) => (optional.value.as_mut(), rest),
            ProjectParamValue::Struct(param_struct) => (
                &mut field_mut(&mut param_struct.fields, segment)?.value,
                tail,
            ),
            ProjectParamValue::Vec(vec_value) => (
                vec_value.items.get_mut(parse_index_segment(segment)?)?,
                tail,
            ),
            ProjectParamValue::DataEnum(data_enum) => {
                let (name, tail) = tail.split_first()?;
                let variant = data_enum.variant_mut(segment)?;
                (&mut field_mut(&mut variant.fields, name)?.value, tail)
            }
            _ => return None,
        };
    }
    Some(value)
}

fn parse_index_segment(segment: &str) -> Option<usize> {
    let index_text = segment.strip_prefix('[')?.strip_suffix(']')?;
    index_text.parse::<usize>().ok()
}
