use plottery_project::{project_param::ProjectParam, project_param_value::ProjectParamValue};

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
    get_param_value_ref_by_path(list, path, false).cloned()
}

/// The default of the leaf at `path`, `Optional` unwrapped. Vec items past the defaults' end use the item prototype.
pub fn get_default_leaf_value(
    defaults: &[ProjectParam],
    path: &[String],
) -> Option<ProjectParamValue> {
    get_param_value_ref_by_path(defaults, path, true)
        .map(|value| match value {
            ProjectParamValue::Optional(optional) => optional.value.as_ref(),
            other => other,
        })
        .cloned()
}

pub fn get_param_value_mut_by_path<'a>(
    list: &'a mut [ProjectParam],
    path: &[String],
) -> Option<&'a mut ProjectParamValue> {
    if path.is_empty() {
        return None;
    }

    let field_name = &path[0];
    let param = list.iter_mut().find(|param| param.name == *field_name)?;

    if path.len() == 1 {
        return Some(&mut param.value);
    }

    get_param_value_mut_in_value(&mut param.value, &path[1..])
}

pub fn get_param_mut_by_path<'a>(
    list: &'a mut [ProjectParam],
    path: &[String],
) -> Option<&'a mut ProjectParam> {
    if path.is_empty() {
        return None;
    }

    let field_name = &path[0];
    let param = list.iter_mut().find(|param| param.name == *field_name)?;

    if path.len() == 1 {
        return Some(param);
    }

    get_param_mut_in_value(&mut param.value, &path[1..])
}

fn get_param_value_ref_by_path<'a>(
    list: &'a [ProjectParam],
    path: &[String],
    prototype_fallback: bool,
) -> Option<&'a ProjectParamValue> {
    if path.is_empty() {
        return None;
    }

    let field_name = &path[0];
    let param = list.iter().find(|param| param.name == *field_name)?;

    if path.len() == 1 {
        return Some(&param.value);
    }

    get_param_value_ref_in_value(&param.value, &path[1..], prototype_fallback)
}

fn get_param_value_ref_in_value<'a>(
    value: &'a ProjectParamValue,
    path: &[String],
    prototype_fallback: bool,
) -> Option<&'a ProjectParamValue> {
    if path.is_empty() {
        return Some(value);
    }

    match value {
        ProjectParamValue::Struct(param_struct) => {
            get_param_value_ref_by_path(&param_struct.fields, path, prototype_fallback)
        }
        ProjectParamValue::Optional(optional) => {
            get_param_value_ref_in_value(optional.value.as_ref(), path, prototype_fallback)
        }
        ProjectParamValue::Vec(vec_value) => {
            let index = parse_index_segment(&path[0])?;
            let item = match vec_value.items.get(index) {
                Some(item) => item,
                None if prototype_fallback => vec_value.item_prototype.as_ref(),
                None => return None,
            };
            get_param_value_ref_in_value(item, &path[1..], prototype_fallback)
        }
        _ => None,
    }
}

fn get_param_value_mut_in_value<'a>(
    value: &'a mut ProjectParamValue,
    path: &[String],
) -> Option<&'a mut ProjectParamValue> {
    if path.is_empty() {
        return Some(value);
    }

    match value {
        ProjectParamValue::Struct(param_struct) => {
            get_param_value_mut_by_path(&mut param_struct.fields, path)
        }
        ProjectParamValue::Optional(optional) => {
            get_param_value_mut_in_value(optional.value.as_mut(), path)
        }
        ProjectParamValue::Vec(vec_value) => {
            let index = parse_index_segment(&path[0])?;
            let item = vec_value.items.get_mut(index)?;
            get_param_value_mut_in_value(item, &path[1..])
        }
        _ => None,
    }
}

fn get_param_mut_in_value<'a>(
    value: &'a mut ProjectParamValue,
    path: &[String],
) -> Option<&'a mut ProjectParam> {
    if path.is_empty() {
        return None;
    }

    match value {
        ProjectParamValue::Struct(param_struct) => {
            get_param_mut_by_path(&mut param_struct.fields, path)
        }
        ProjectParamValue::Optional(optional) => {
            get_param_mut_in_value(optional.value.as_mut(), path)
        }
        ProjectParamValue::Vec(vec_value) => {
            let index = parse_index_segment(&path[0])?;
            let item = vec_value.items.get_mut(index)?;
            get_param_mut_in_value(item, &path[1..])
        }
        _ => None,
    }
}

fn parse_index_segment(segment: &str) -> Option<usize> {
    let index_text = segment.strip_prefix('[')?.strip_suffix(']')?;
    index_text.parse::<usize>().ok()
}
