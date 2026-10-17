//! Sends the Phase 0.2 test patterns from `server/docs/kernel_side_stepping.md`
//! to the plotter server. Runs on the laptop and talks to `otter.local`.
//!
//! ```sh
//! cargo run -p plottery_server --example test_patterns -- square-up 20 2
//! cargo run -p plottery_server --example test_patterns -- square-down 20 2
//! cargo run -p plottery_server --example test_patterns -- head 50
//! cargo run -p plottery_server --example test_patterns -- state     # what the server reports
//! cargo run -p plottery_server --example test_patterns -- enable    # motors on (should lock/hum)
//! cargo run -p plottery_server --example test_patterns -- disable
//! PLOTTERY_SPEED_FACTOR=0.5 cargo run -p plottery_server --example test_patterns -- square-down 20 2
//! ```
//!
//! Before each run: put the pen over the start mark and press "set origin"
//! in the editor's Remote page. Every pattern ends back at the origin.
//! `square-up` is sent as one `MoveTo` task per corner, so the server prints
//! one stats line per corner; `square-down` is one continuous pen-down path
//! and prints one line. Defaults: 20 cm sides, 2 laps, 50 head cycles.

use std::time::Duration;

use plottery_lib::{Layer, Path, SampleSettings, V2};
use plottery_server_lib::{
    plot_setting::PlotSettings,
    server_state::request_server_state,
    task::{send_task, Task},
};

fn main() -> anyhow::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(run())
}

async fn run() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let factor = speed_factor()?;
    let settings = scaled_settings(factor);

    match args.first().map(String::as_str) {
        Some("square-up") => {
            let (size, laps) = size_and_laps(&args)?;
            println!("square, pen up: {size} cm sides, {laps} laps, speed factor {factor}");
            for _ in 0..laps {
                for corner in corners(size) {
                    send_task(Task::MoveTo(corner, settings.clone())).await?;
                }
            }
        }
        Some("square-down") => {
            let (size, laps) = size_and_laps(&args)?;
            println!("square, pen down: {size} cm sides, {laps} laps, speed factor {factor}");
            let mut points = Vec::with_capacity(laps as usize * 4 + 1);
            points.push(V2::zero());
            for _ in 0..laps {
                points.extend(corners(size));
            }
            let mut layer = Layer::new();
            layer.push(Path::new_shape_from(points));
            send_task(Task::Plot {
                layer,
                sample_settings: SampleSettings::default(),
                plot_settings: settings,
            })
            .await?;
        }
        Some("head") => {
            let cycles: u32 = args
                .get(1)
                .map(|value| value.parse())
                .transpose()?
                .unwrap_or(50);
            println!("head: {cycles} down/up cycles (server-side default speeds)");
            for _ in 0..cycles {
                send_task(Task::SetHead(true)).await?;
                send_task(Task::SetHead(false)).await?;
            }
        }
        Some("state") => {
            let state = request_server_state(Some(Duration::from_secs(2))).await?;
            println!("{state:#?}");
        }
        Some("enable") => send_task(Task::SetEnabled(true)).await?,
        Some("disable") => send_task(Task::SetEnabled(false)).await?,
        _ => {
            eprintln!(
                "usage: test_patterns square-up <size_cm> <laps> | square-down <size_cm> <laps> | head <cycles> | state | enable | disable"
            );
            eprintln!("       PLOTTERY_SPEED_FACTOR=0.5 scales every max speed (min speeds stay)");
        }
    }
    Ok(())
}

fn speed_factor() -> anyhow::Result<f32> {
    match std::env::var("PLOTTERY_SPEED_FACTOR") {
        Ok(value) => Ok(value.parse()?),
        Err(_) => Ok(1.0),
    }
}

fn scaled_settings(factor: f32) -> PlotSettings {
    let mut settings = PlotSettings::default();
    for range in [
        &mut settings.speed_draw,
        &mut settings.speed_travel,
        &mut settings.speed_head_down,
        &mut settings.speed_head_up,
    ] {
        range.max = (range.max * factor).max(range.min);
    }
    settings
}

fn size_and_laps(args: &[String]) -> anyhow::Result<(f32, u32)> {
    let size = args
        .get(1)
        .map(|value| value.parse::<f32>())
        .transpose()?
        .unwrap_or(20.0);
    let laps = args
        .get(2)
        .map(|value| value.parse::<u32>())
        .transpose()?
        .unwrap_or(2);
    Ok((size, laps))
}

fn corners(size: f32) -> [V2; 4] {
    [
        V2::new(size, 0.0),
        V2::new(size, size),
        V2::new(0.0, size),
        V2::zero(),
    ]
}
