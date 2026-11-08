use plottery_lib::{Layer, Paper, Rect};
use plottery_project::{ProjectParam, ProjectParamValue};

/// Counts, lengths and bounds of `layer`, checked against the first top-level paper param.
pub fn format_stats(layer: &Layer, params: &[ProjectParam]) -> String {
    let stats = layer.plot_stats();
    let counts = format!(
        "shapes {} in {} {}, draw {}, travel {}",
        stats.shapes,
        stats.layers,
        if stats.layers == 1 { "layer" } else { "layers" },
        length(stats.draw_length),
        length(stats.travel_length)
    );
    let Some(bounds) = stats.bounds else {
        return format!("{counts}\nbounds: empty drawing");
    };
    let mut bounds_line = format!(
        "bounds {:.1},{:.1} .. {:.1},{:.1} cm",
        bounds.bl().x,
        bounds.bl().y,
        bounds.tr().x,
        bounds.tr().y
    );
    let paper = params.iter().find_map(|param| match &param.value {
        ProjectParamValue::Paper(paper) => Some(paper),
        _ => None,
    });
    if let Some(paper) = paper {
        bounds_line += &format!(", {}", paper_check(&bounds, paper));
    }
    format!("{counts}\n{bounds_line}")
}

fn paper_check(bounds: &Rect, paper: &Paper) -> String {
    const TOLERANCE: f32 = 0.001;
    let outside_paper = overshoot(bounds, &paper.rect());
    let outside_frame = overshoot(bounds, &paper.frame().inner_rect());
    if outside_paper > TOLERANCE {
        format!("outside paper by {outside_paper:.1} cm")
    } else if outside_frame > TOLERANCE {
        format!("outside frame by {outside_frame:.1} cm")
    } else {
        "inside frame".to_string()
    }
}

/// How far `bounds` reaches past `area` on its worst side, 0 when inside.
fn overshoot(bounds: &Rect, area: &Rect) -> f32 {
    [
        area.bl().x - bounds.bl().x,
        area.bl().y - bounds.bl().y,
        bounds.tr().x - area.tr().x,
        bounds.tr().y - area.tr().y,
    ]
    .into_iter()
    .fold(0.0, f32::max)
}

fn length(cm: f32) -> String {
    if cm >= 100.0 {
        format!("{:.1} m", cm / 100.0)
    } else {
        format!("{cm:.1} cm")
    }
}
