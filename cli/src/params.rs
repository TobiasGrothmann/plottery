use plottery_project::{ProjectParam, ProjectParamValue as Value};
use std::collections::HashSet;

/// One printed line: a param or nested field addressed by its dotted path.
struct Row<'a> {
    path: String,
    value: &'a Value,
    default: Option<&'a Value>,
    section: Option<&'a str>,
    description: Option<&'a str>,
}

/// Lines of `plottery params`: one `--set` target per line, `# Section` headers in between.
pub fn format_params(params: &[ProjectParam], defaults: &[ProjectParam]) -> String {
    let mut rows = Vec::new();
    push_list(params, Some(defaults), "", None, &mut rows);

    let mut lines = Vec::new();
    let mut section = None;
    for row in &rows {
        if row.section != section {
            section = row.section;
            if let Some(name) = section {
                lines.push(format!("# {name}"));
            }
        }
        lines.push(format_row(row));
    }
    lines.join("\n")
}

/// The lines of `format_params` that differ after a change, or `No changes`.
pub fn format_changes(
    before: &[ProjectParam],
    after: &[ProjectParam],
    defaults: &[ProjectParam],
) -> String {
    let before = format_params(before, defaults);
    let before: HashSet<&str> = before.lines().collect();
    let after = format_params(after, defaults);
    let changed: Vec<&str> = after
        .lines()
        .filter(|line| !line.starts_with("# ") && !before.contains(line))
        .collect();
    if changed.is_empty() {
        "No changes".to_string()
    } else {
        changed.join("\n")
    }
}

fn push_list<'a>(
    list: &'a [ProjectParam],
    defaults: Option<&'a [ProjectParam]>,
    prefix: &str,
    parent_section: Option<&'a str>,
    rows: &mut Vec<Row<'a>>,
) {
    for param in list {
        let default = defaults
            .and_then(|defaults| defaults.iter().find(|d| d.name == param.name))
            .map(|d| &d.value);
        push_value(
            Row {
                path: format!("{prefix}{}", param.name),
                value: &param.value,
                default,
                section: param.section.as_deref().or(parent_section),
                description: param.description.as_deref(),
            },
            rows,
        );
    }
}

/// Pushes `row` and the rows of its fields or items. Plain structs only show their fields.
fn push_value<'a>(row: Row<'a>, rows: &mut Vec<Row<'a>>) {
    let prefix = format!("{}.", row.path);
    let section = row.section;
    match row.value {
        Value::Struct(param_struct) => {
            let defaults = match row.default {
                Some(Value::Struct(d)) => Some(d.fields.as_slice()),
                _ => None,
            };
            push_list(&param_struct.fields, defaults, &prefix, section, rows);
        }
        Value::Optional(optional) => {
            let default = row.default;
            rows.push(row);
            if let Value::Struct(param_struct) = optional.value.as_ref() {
                let defaults = match default {
                    Some(Value::Optional(d)) => match d.value.as_ref() {
                        Value::Struct(d) => Some(d.fields.as_slice()),
                        _ => None,
                    },
                    _ => None,
                };
                push_list(&param_struct.fields, defaults, &prefix, section, rows);
            }
        }
        Value::Vec(param_vec) => {
            let default_items = match row.default {
                Some(Value::Vec(d)) => d.items.as_slice(),
                _ => &[],
            };
            rows.push(row);
            for (i, item) in param_vec.items.iter().enumerate() {
                push_value(
                    Row {
                        path: format!("{prefix}{i}"),
                        value: item,
                        default: default_items.get(i),
                        section,
                        description: None,
                    },
                    rows,
                );
            }
        }
        _ => rows.push(row),
    }
}

fn format_row(row: &Row) -> String {
    let mut line = format!(
        "{}: {} = {}",
        row.path,
        type_label(row.value),
        row.value.to_text()
    );
    let mut notes: Vec<String> = constraint(row.value).into_iter().collect();
    if let Some(default) = row.default.filter(|d| !same_value(row.value, d)) {
        notes.push(format!("default {}", default.to_text()));
    }
    if !notes.is_empty() {
        line += &format!("   ({})", notes.join(", "));
    }
    if let Some(description) = row.description {
        line += &format!(
            "   {}",
            description.split_whitespace().collect::<Vec<_>>().join(" ")
        );
    }
    line
}

fn type_label(value: &Value) -> String {
    match value {
        Value::Float(_) | Value::FloatRanged { .. } => "f32".to_string(),
        Value::Int(_) | Value::IntRanged { .. } => "i32".to_string(),
        Value::Optional(optional) => format!("option<{}>", type_label(&optional.value)),
        Value::Vec(param_vec) => format!("vec<{}>", type_label(&param_vec.item_prototype)),
        _ => value.type_name(),
    }
}

/// Allowed values: enum variants or a range.
fn constraint(value: &Value) -> Option<String> {
    match value {
        Value::Enum(param_enum) => Some(param_enum.variants.join(" | ")),
        Value::FloatRanged { min, max, .. } => Some(format!("{min}..{max}")),
        Value::IntRanged { min, max, .. } => Some(format!("{min}..{max}")),
        Value::RangeF32 {
            bounds: Some((low, high)),
            ..
        } => Some(format!("within {low}..{high}")),
        Value::RangeI32 {
            bounds: Some((low, high)),
            ..
        } => Some(format!("within {low}..{high}")),
        Value::Optional(optional) => constraint(&optional.value),
        _ => None,
    }
}

/// Equal as far as the user can tell: ignores seed and file history, fields and items are
/// compared on their own rows.
fn same_value(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Seed(a), Value::Seed(b)) => a.current == b.current,
        (Value::Image(a), Value::Image(b)) | (Value::Svg(a), Value::Svg(b)) => a.path == b.path,
        (Value::Optional(a), Value::Optional(b)) => {
            a.enabled == b.enabled
                && (!a.enabled
                    || matches!(a.value.as_ref(), Value::Struct(_))
                    || same_value(&a.value, &b.value))
        }
        (Value::Vec(a), Value::Vec(b)) => a.items.len() == b.items.len(),
        _ => a == b,
    }
}
