use std::time::Duration;

use plottery_server_lib::{PlotOutcome, ServerState};

/// What happened to a plot between two polls of the plotter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotEvent {
    Finished(Duration),
    Aborted,
    /// The running plot is gone without a result: the server restarted.
    Lost,
}

/// Follows the plotter's plots across polls and reports each plot's end once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlotWatch {
    /// False until the first state arrived; what that one shows is old news.
    primed: bool,
    active: Option<u32>,
    last_result: Option<u32>,
}

impl PlotWatch {
    /// A plot was running at the last successful poll.
    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }

    /// Takes one poll result, `None` when the plotter did not answer.
    pub fn observe(self, state: Option<&ServerState>) -> (Self, Option<PlotEvent>) {
        let Some(state) = state else {
            return (self, None);
        };
        let next = Self {
            primed: true,
            active: state.plot.map(|plot| plot.id),
            last_result: state.last_plot.map(|result| result.id),
        };
        if !self.primed {
            return (next, None);
        }

        let event = match state.last_plot {
            Some(result) if Some(result.id) != self.last_result => Some(match result.outcome {
                PlotOutcome::Finished => PlotEvent::Finished(result.duration),
                PlotOutcome::Aborted => PlotEvent::Aborted,
            }),
            _ if self.active.is_some() && next.active != self.active => Some(PlotEvent::Lost),
            _ => None,
        };
        (next, event)
    }
}
