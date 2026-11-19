use std::time::Instant;

use plottery_lib::V2;
use plottery_server_lib::{
    pins::HARDWARE_CONSTS, Activity, PlotOutcome, PlotProgress, PlotResult, PlotTimeline,
    ServerState, TelemetryReading,
};

/// What the plot thread is working on; it writes this at task boundaries only.
#[derive(Debug)]
pub struct JobState {
    phase: Phase,
    last_plot: Option<PlotResult>,
    next_plot_id: u32,
}

#[derive(Debug)]
enum Phase {
    Idle,
    Moving,
    Plot(RunningPlot),
}

#[derive(Debug)]
struct RunningPlot {
    id: u32,
    started: Instant,
    /// `None` while the timeline is being prepared.
    schedule: Option<Schedule>,
}

#[derive(Debug)]
struct Schedule {
    timeline: PlotTimeline,
    /// The hardware's step count when the plot started.
    steps_at_start: u64,
}

impl JobState {
    /// `first_plot_id` should differ between server starts, so clients can tell plots apart across a restart.
    pub fn new(first_plot_id: u32) -> Self {
        Self {
            phase: Phase::Idle,
            last_plot: None,
            next_plot_id: first_plot_id,
        }
    }

    pub fn begin_task(&mut self) {
        self.phase = Phase::Moving;
    }

    pub fn begin_plot(&mut self, now: Instant) {
        self.phase = Phase::Plot(RunningPlot {
            id: self.next_plot_id,
            started: now,
            schedule: None,
        });
        self.next_plot_id = self.next_plot_id.wrapping_add(1);
    }

    /// The timeline is ready and the plot starts stepping.
    pub fn start_plot(&mut self, timeline: PlotTimeline, steps_at_start: u64) {
        if let Phase::Plot(plot) = &mut self.phase {
            plot.schedule = Some(Schedule {
                timeline,
                steps_at_start,
            });
        }
    }

    pub fn finish_task(&mut self, aborted: bool, now: Instant) {
        if let Phase::Plot(plot) = &self.phase {
            self.last_plot = Some(PlotResult {
                id: plot.id,
                outcome: if aborted {
                    PlotOutcome::Aborted
                } else {
                    PlotOutcome::Finished
                },
                duration: now.saturating_duration_since(plot.started),
            });
        }
        self.phase = Phase::Idle;
    }

    /// The state reported by `GET /state`, from this and a telemetry reading.
    pub fn snapshot(&self, reading: TelemetryReading, aborting: bool, now: Instant) -> ServerState {
        let step_cm = HARDWARE_CONSTS.hardware_profile.dist_per_step_axis_cm;

        let plot = match &self.phase {
            Phase::Plot(plot) => plot.schedule.as_ref().map(|schedule| {
                let timeline = &schedule.timeline;
                let steps = reading.steps.saturating_sub(schedule.steps_at_start);
                PlotProgress {
                    id: plot.id,
                    shape: (reading.shape + 1).min(timeline.shapes()),
                    shapes: timeline.shapes(),
                    elapsed: now.saturating_duration_since(plot.started),
                    remaining: timeline.total().saturating_sub(timeline.elapsed_at(steps)),
                }
            }),
            _ => None,
        };

        let activity = match (&self.phase, &plot) {
            (Phase::Idle, _) => Activity::Idle,
            _ if aborting => Activity::Aborting,
            (Phase::Moving, _) => Activity::Moving,
            (Phase::Plot(_), None) => Activity::Preparing,
            (Phase::Plot(_), Some(plot)) if reading.shape >= plot.shapes => Activity::Returning,
            (Phase::Plot(_), Some(_)) if reading.head_down => Activity::Drawing,
            (Phase::Plot(_), Some(_)) => Activity::Traveling,
        };

        ServerState {
            location: V2::new(reading.x as f32 * step_cm, reading.y as f32 * step_cm),
            motors_enabled: reading.enabled,
            head_down: reading.head_down,
            activity,
            plot,
            last_plot: self.last_plot,
        }
    }
}
