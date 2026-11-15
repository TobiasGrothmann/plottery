use std::{
    io::Write,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use anyhow::{bail, Context, Result};
use plottery_lib::{Layer, Rect};
use plottery_project::{rasterize, Raster};
use rayon::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VideoFormat {
    Mp4,
    Gif,
}

impl VideoFormat {
    pub fn from_extension(extension: Option<&str>) -> Option<Self> {
        match extension {
            Some("mp4") => Some(VideoFormat::Mp4),
            Some("gif") => Some(VideoFormat::Gif),
            _ => None,
        }
    }
}

pub struct VideoOptions {
    pub output: PathBuf,
    pub format: VideoFormat,
    pub fps: u32,
    /// Seconds the last frame stays at the end.
    pub hold: f32,
    /// Longest side in pixels.
    pub size: u32,
}

/// Fails unless ffmpeg can be started.
pub fn check_ffmpeg() -> Result<()> {
    let run = Command::new("ffmpeg")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    match run {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => bail!("`ffmpeg -version` failed ({status})"),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            bail!("ffmpeg not found, `plottery animate` needs it to write the video. Install it, e.g. with `brew install ffmpeg`")
        }
        Err(e) => Err(e).context("Failed to run ffmpeg"),
    }
}

/// Starts ffmpeg reading raw RGBA frames of `width` x `height` pixels from its stdin.
fn spawn_ffmpeg(options: &VideoOptions, width: u32, height: u32) -> Result<Child> {
    let hold = format!("tpad=stop_mode=clone:stop_duration={}", options.hold);
    let (filter, codec): (String, &[&str]) = match options.format {
        // h264 with yuv420p, which every player shows, needs even sides
        VideoFormat::Mp4 => (
            format!("{hold},pad=ceil(iw/2)*2:ceil(ih/2)*2:color=white"),
            &["-c:v", "libx264", "-pix_fmt", "yuv420p", "-crf", "18"],
        ),
        VideoFormat::Gif => (
            format!("{hold},split[a][b];[a]palettegen[p];[b][p]paletteuse"),
            &[],
        ),
    };
    Command::new("ffmpeg")
        .args(["-y", "-loglevel", "error"])
        .args(["-f", "rawvideo", "-pix_fmt", "rgba"])
        .args(["-s", &format!("{width}x{height}")])
        .args(["-framerate", &options.fps.to_string()])
        .args(["-i", "-", "-vf", &filter])
        .args(codec)
        .arg(&options.output)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .context("Failed to run ffmpeg")
}

/// Turns frames into a video as they come in: collects a few, rasterizes them in parallel and
/// pipes the pixels into ffmpeg. Never holds more than one batch.
pub struct VideoWriter {
    options: VideoOptions,
    /// What every frame shows.
    view: Rect,
    batch: Vec<Layer>,
    batch_size: usize,
    /// Started with the first batch, when the pixel size of the frames is known.
    ffmpeg: Option<Child>,
    frame_count: usize,
}

impl VideoWriter {
    pub fn new(options: VideoOptions, view: Rect) -> Self {
        Self {
            options,
            view,
            batch: Vec::new(),
            // one frame per core rasterizes as fast as a single one
            batch_size: rayon::current_num_threads(),
            ffmpeg: None,
            frame_count: 0,
        }
    }

    /// Frames added so far.
    pub fn frame_count(&self) -> usize {
        self.frame_count
    }

    /// Length of the video so far in seconds, the hold at the end included.
    pub fn duration(&self) -> f32 {
        self.frame_count as f32 / self.options.fps as f32 + self.options.hold
    }

    /// Adds `frame` as the next frame. Renders when the batch is full, which takes a while.
    pub fn push(&mut self, frame: Layer) -> Result<()> {
        self.batch.push(frame);
        self.frame_count += 1;
        if self.batch.len() >= self.batch_size {
            self.flush()?;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        let (view, size) = (self.view, self.options.size);
        let rasters = self
            .batch
            .par_iter()
            .map(|frame| rasterize(frame, view, size))
            .collect::<Result<Vec<Raster>>>()
            .context("Failed to render a frame")?;
        self.batch.clear();
        let Some(first) = rasters.first() else {
            return Ok(());
        };

        if self.ffmpeg.is_none() {
            self.ffmpeg = Some(spawn_ffmpeg(&self.options, first.width, first.height)?);
        }
        let stdin = self
            .ffmpeg
            .as_mut()
            .and_then(|ffmpeg| ffmpeg.stdin.as_mut());
        let stdin = stdin.context("ffmpeg has no stdin")?;
        for raster in &rasters {
            // ffmpeg says on stderr why it stopped reading
            stdin.write_all(&raster.rgba).context("ffmpeg failed")?;
        }
        Ok(())
    }

    /// Renders what is left and waits for ffmpeg to finish the video.
    pub fn finish(mut self) -> Result<()> {
        self.flush()?;
        let Some(mut ffmpeg) = self.ffmpeg.take() else {
            bail!("No frames, no video was written");
        };
        drop(ffmpeg.stdin.take());
        let status = ffmpeg.wait().context("Failed to wait for ffmpeg")?;
        if !status.success() {
            bail!("ffmpeg failed ({status})");
        }
        Ok(())
    }
}

impl Drop for VideoWriter {
    /// Stops ffmpeg when the animation is given up half way, and removes its unfinished video.
    fn drop(&mut self) {
        if let Some(ffmpeg) = &mut self.ffmpeg {
            let _ = ffmpeg.kill();
            let _ = ffmpeg.wait();
            let _ = std::fs::remove_file(&self.options.output);
        }
    }
}
