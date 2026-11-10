use dioxus::prelude::*;
use plottery_server_lib::{plot_setting::SpeedRange, PlotSettings};

#[derive(PartialEq, Props, Clone)]
pub struct PlotSettingsEditorProps {
    plot_settings: SyncSignal<PlotSettings>,
}

type Setter = fn(&mut PlotSettings, f32);

fn number_input(
    mut plot_settings: SyncSignal<PlotSettings>,
    value: f32,
    aria_label: String,
    set: Setter,
) -> Element {
    rsx! {
        input {
            class: "input",
            "aria-label": "{aria_label}",
            value: value.to_string(),
            onchange: move |event| {
                if let Ok(value) = event.value().parse::<f32>() {
                    let mut new_settings = plot_settings.read().clone();
                    set(&mut new_settings, value);
                    plot_settings.set(new_settings);
                }
            },
        }
    }
}

fn speed_row(
    plot_settings: SyncSignal<PlotSettings>,
    label: &str,
    speed: &SpeedRange,
    [set_min, set_max, set_accel]: [Setter; 3],
) -> Element {
    rsx! {
        span { class: "ftable-label", "{label}" }
        {number_input(plot_settings, speed.min, format!("{label} min"), set_min)}
        {number_input(plot_settings, speed.max, format!("{label} max"), set_max)}
        {number_input(plot_settings, speed.accelleration_distance, format!("{label} acceleration distance"), set_accel)}
    }
}

#[component]
pub fn PlotSettingsEditor(props: PlotSettingsEditorProps) -> Element {
    let plot_settings = props.plot_settings;
    let settings = plot_settings.read().clone();

    rsx! {
        style { { include_str!("plot_settings_editor.css") } }
        section { class: "fsection",
            span { class: "caps", "XY" }
            div { class: "ftable",
                span {}
                span { class: "hint", "Min" }
                span { class: "hint", "Max" }
                span { class: "hint", "Accel. distance" }
                {speed_row(plot_settings, "Draw", &settings.speed_draw, [
                    |s, v| s.speed_draw.min = v,
                    |s, v| s.speed_draw.max = v,
                    |s, v| s.speed_draw.accelleration_distance = v,
                ])}
                {speed_row(plot_settings, "Travel", &settings.speed_travel, [
                    |s, v| s.speed_travel.min = v,
                    |s, v| s.speed_travel.max = v,
                    |s, v| s.speed_travel.accelleration_distance = v,
                ])}
            }
            div { class: "fgrid",
                label { class: "flabel",
                    "Corner slowdown power"
                    {number_input(plot_settings, settings.corner_slowdown_power, "Corner slowdown power".to_string(), |s, v| s.corner_slowdown_power = v)}
                    span { class: "hint", "Lower is slower in corners" }
                }
            }
        }
        section { class: "fsection",
            span { class: "caps", "Head" }
            div { class: "ftable",
                span {}
                span { class: "hint", "Min" }
                span { class: "hint", "Max" }
                span { class: "hint", "Accel. distance" }
                {speed_row(plot_settings, "Down", &settings.speed_head_down, [
                    |s, v| s.speed_head_down.min = v,
                    |s, v| s.speed_head_down.max = v,
                    |s, v| s.speed_head_down.accelleration_distance = v,
                ])}
                {speed_row(plot_settings, "Up", &settings.speed_head_up, [
                    |s, v| s.speed_head_up.min = v,
                    |s, v| s.speed_head_up.max = v,
                    |s, v| s.speed_head_up.accelleration_distance = v,
                ])}
            }
            div { class: "fgrid",
                label { class: "flabel",
                    "Extra head travel"
                    span { class: "affix affix-end",
                        {number_input(plot_settings, settings.head_travel_beyond_paper_cm, "Extra head travel in cm".to_string(), |s, v| s.head_travel_beyond_paper_cm = v)}
                        span { "cm" }
                    }
                    span { class: "hint", "Head moves this much further after the pen touches the paper" }
                }
            }
        }
    }
}
