use plottery_project::{project_param::ProjectParam, project_param_value::ProjectParamValue};

pub fn get_param_value_by_path(
    list: &[ProjectParam],
    path: &[String],
) -> Option<ProjectParamValue> {
    get_param_by_path(list, path).map(|param| param.value.clone())
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

    match &mut param.value {
        ProjectParamValue::Struct(param_struct) => {
            get_param_mut_by_path(&mut param_struct.fields, &path[1..])
        }
        _ => None,
    }
}

fn get_param_by_path<'a>(list: &'a [ProjectParam], path: &[String]) -> Option<&'a ProjectParam> {
    if path.is_empty() {
        return None;
    }

    let field_name = &path[0];
    let param = list.iter().find(|param| param.name == *field_name)?;

    if path.len() == 1 {
        return Some(param);
    }

    match &param.value {
        ProjectParamValue::Struct(param_struct) => {
            get_param_by_path(&param_struct.fields, &path[1..])
        }
        _ => None,
    }
}
