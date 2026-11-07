pub mod dropdown;
pub mod number_input;
pub mod segmented;
pub mod slider_input;
pub mod text_input;

/// Formats with up to five decimals and no trailing zeros, so `0.25` stays `0.25`.
pub fn format_number(value: f32) -> String {
    format!("{:.5}", value)
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}
