use dioxus::prelude::*;
use plottery_server_lib::PlotSettings;

#[derive(PartialEq, Props, Clone)]
pub struct PlotSettingsEditorProps {
    plot_settings: SyncSignal<PlotSettings>,
}

#[component]
pub fn PlotSettingsEditor(props: PlotSettingsEditorProps) -> Element {
    let mut plot_settings = props.plot_settings;
    let settings = plot_settings.read().clone();

    rsx! {
        style { { include_str!("plot_settings_editor.css") } }
        div { class: "PlotSettingsEditor",
            h2 { class: "title", "Plot settings" }
            button {
                class: "reset_button",
                onclick: move |_| {
                    plot_settings.set(PlotSettings::default());
                },
                "reset defaults"
            }

            p { "corner slowdown power" }
            input {
                value: settings.corner_slowdown_power.to_string(),
                onchange: move |event| {
                    if let Ok(value) = event.value().parse::<f32>() {
                        let mut new_settings = plot_settings.read().clone();
                        new_settings.corner_slowdown_power = value;
                        plot_settings.set(new_settings);
                    }
                }
            }

            p { "head travel beyond paper (cm)" }
            input {
                value: settings.head_travel_beyond_paper_cm.to_string(),
                onchange: move |event| {
                    if let Ok(value) = event.value().parse::<f32>() {
                        let mut new_settings = plot_settings.read().clone();
                        new_settings.head_travel_beyond_paper_cm = value;
                        plot_settings.set(new_settings);
                    }
                }
            }

            p { class: "section_title", "draw speed" }
            div { class: "section_spacer" }

            p { "draw min" }
            input {
                value: settings.speed_draw.min.to_string(),
                onchange: move |event| {
                    if let Ok(value) = event.value().parse::<f32>() {
                        let mut new_settings = plot_settings.read().clone();
                        new_settings.speed_draw.min = value;
                        plot_settings.set(new_settings);
                    }
                }
            }
            p { "draw max" }
            input {
                value: settings.speed_draw.max.to_string(),
                onchange: move |event| {
                    if let Ok(value) = event.value().parse::<f32>() {
                        let mut new_settings = plot_settings.read().clone();
                        new_settings.speed_draw.max = value;
                        plot_settings.set(new_settings);
                    }
                }
            }
            p { "draw acceleration distance" }
            input {
                value: settings.speed_draw.accelleration_distance.to_string(),
                onchange: move |event| {
                    if let Ok(value) = event.value().parse::<f32>() {
                        let mut new_settings = plot_settings.read().clone();
                        new_settings.speed_draw.accelleration_distance = value;
                        plot_settings.set(new_settings);
                    }
                }
            }

            p { class: "section_title", "travel speed" }
            div { class: "section_spacer" }

            p { "travel min" }
            input {
                value: settings.speed_travel.min.to_string(),
                onchange: move |event| {
                    if let Ok(value) = event.value().parse::<f32>() {
                        let mut new_settings = plot_settings.read().clone();
                        new_settings.speed_travel.min = value;
                        plot_settings.set(new_settings);
                    }
                }
            }
            p { "travel max" }
            input {
                value: settings.speed_travel.max.to_string(),
                onchange: move |event| {
                    if let Ok(value) = event.value().parse::<f32>() {
                        let mut new_settings = plot_settings.read().clone();
                        new_settings.speed_travel.max = value;
                        plot_settings.set(new_settings);
                    }
                }
            }
            p { "travel acceleration distance" }
            input {
                value: settings.speed_travel.accelleration_distance.to_string(),
                onchange: move |event| {
                    if let Ok(value) = event.value().parse::<f32>() {
                        let mut new_settings = plot_settings.read().clone();
                        new_settings.speed_travel.accelleration_distance = value;
                        plot_settings.set(new_settings);
                    }
                }
            }

            p { class: "section_title", "head down speed" }
            div { class: "section_spacer" }

            p { "head down min" }
            input {
                value: settings.speed_head_down.min.to_string(),
                onchange: move |event| {
                    if let Ok(value) = event.value().parse::<f32>() {
                        let mut new_settings = plot_settings.read().clone();
                        new_settings.speed_head_down.min = value;
                        plot_settings.set(new_settings);
                    }
                }
            }
            p { "head down max" }
            input {
                value: settings.speed_head_down.max.to_string(),
                onchange: move |event| {
                    if let Ok(value) = event.value().parse::<f32>() {
                        let mut new_settings = plot_settings.read().clone();
                        new_settings.speed_head_down.max = value;
                        plot_settings.set(new_settings);
                    }
                }
            }
            p { "head down acceleration distance" }
            input {
                value: settings.speed_head_down.accelleration_distance.to_string(),
                onchange: move |event| {
                    if let Ok(value) = event.value().parse::<f32>() {
                        let mut new_settings = plot_settings.read().clone();
                        new_settings.speed_head_down.accelleration_distance = value;
                        plot_settings.set(new_settings);
                    }
                }
            }

            p { class: "section_title", "head up speed" }
            div { class: "section_spacer" }

            p { "head up min" }
            input {
                value: settings.speed_head_up.min.to_string(),
                onchange: move |event| {
                    if let Ok(value) = event.value().parse::<f32>() {
                        let mut new_settings = plot_settings.read().clone();
                        new_settings.speed_head_up.min = value;
                        plot_settings.set(new_settings);
                    }
                }
            }
            p { "head up max" }
            input {
                value: settings.speed_head_up.max.to_string(),
                onchange: move |event| {
                    if let Ok(value) = event.value().parse::<f32>() {
                        let mut new_settings = plot_settings.read().clone();
                        new_settings.speed_head_up.max = value;
                        plot_settings.set(new_settings);
                    }
                }
            }
            p { "head up acceleration distance" }
            input {
                value: settings.speed_head_up.accelleration_distance.to_string(),
                onchange: move |event| {
                    if let Ok(value) = event.value().parse::<f32>() {
                        let mut new_settings = plot_settings.read().clone();
                        new_settings.speed_head_up.accelleration_distance = value;
                        plot_settings.set(new_settings);
                    }
                }
            }
        }
    }
}
