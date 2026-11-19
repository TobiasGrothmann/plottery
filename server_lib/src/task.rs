use plottery_lib::*;

use crate::{plot_settings::PlotSettings, Axis};

use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Task {
    PlotShape {
        shape: Shape,
        sample_settings: SampleSettings,
        plot_settings: PlotSettings,
    },
    Plot {
        layer: Layer,
        sample_settings: SampleSettings,
        plot_settings: PlotSettings,
    },
    Abort,
    SetEnabled(bool),
    SetHead(bool),
    MoveTo(V2, PlotSettings),
    Move(V2, PlotSettings),
    SetOrigin(),
    NoteFreq {
        axis: Axis,
        frequency: f32,
        duration_s: f32,
    },
    Note {
        axis: Axis,
        midi: f32,
        duration_s: f32,
    },
}

/// Version of the task request, sent in front of the task. Changes with the postcard layout of [`Task`].
pub const TASK_PROTOCOL_VERSION: u16 = 2;
const TASK_MAGIC: [u8; 4] = *b"PLTT";

impl Task {
    /// Reads a request body written by [`Task::to_binary`], an error if it is of another protocol version.
    pub fn from_binary(data: &[u8]) -> anyhow::Result<Self> {
        match split_file_header(&TASK_MAGIC, data) {
            Some((TASK_PROTOCOL_VERSION, payload)) => {
                postcard::from_bytes(payload).context("The task could not be decoded")
            }
            Some((version, _)) => bail!(
                "The sender uses plotter protocol version {version}, the server version \
                 {TASK_PROTOCOL_VERSION}. Update the older of the two."
            ),
            None => bail!(
                "The task has no protocol version, the server uses version \
                 {TASK_PROTOCOL_VERSION}. Update the sender."
            ),
        }
    }
    pub fn to_binary(&self) -> anyhow::Result<Vec<u8>> {
        Ok(with_file_header(
            &TASK_MAGIC,
            TASK_PROTOCOL_VERSION,
            &postcard::to_stdvec(self)?,
        ))
    }
}
