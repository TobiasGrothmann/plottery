//! Rolling: runs the built project once per roll, a few at a time, and stores the results.

use std::collections::VecDeque;
use std::future::Future;
use std::pin::pin;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::task::Poll;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use chrono::Utc;
use futures_lite::{future::poll_fn, StreamExt};
use plottery_runtime::ProjectParamsListWrapper;
use rand::rngs::StdRng;

use crate::{
    check_roll_mode, check_rollable, roll_param_sets, rolled_values, takes_roll_range, Project,
    RollMode, RollRanges, RunError, RunEvent, VariationsMeta,
};

/// Most rolls in one set.
pub const MAX_ROLLS: usize = 1000;

/// A roll is killed and skipped after this many times the slowest finished one, ...
const TIME_LIMIT_FACTOR: u32 = 5;
/// ... but never before this.
const MIN_TIME_LIMIT: Duration = Duration::from_secs(20);

/// How long roll runs may take: `factor` times the slowest finished one, at least `min`, no limit
/// before one has finished.
pub(crate) fn time_limit(slowest: Option<Duration>) -> Option<Duration> {
    slowest.map(|slowest| (slowest * TIME_LIMIT_FACTOR).max(MIN_TIME_LIMIT))
}

/// Output lines kept per roll for the error of a failed one.
const OUTPUT_TAIL: usize = 20;

#[derive(Debug, Clone)]
pub enum RollEvent {
    /// The runs begin: `total` rolls plus roll 0, the start params, which the counts leave out.
    Prepared { total: usize },
    /// Roll `index` finished and is stored. `done` counts finished and skipped rolls.
    Finished {
        index: usize,
        done: usize,
        total: usize,
    },
    /// Roll `index` ran longer than `limit`, was killed and is stored as skipped. The others go on.
    Skipped {
        index: usize,
        values: String,
        limit: Duration,
        done: usize,
        total: usize,
    },
}

/// How a set of rolls ended. A failed roll is an error instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RollOutcome {
    /// Finished and stored, roll 0 left out like in `total`.
    pub done: usize,
    /// Killed for running too long.
    pub skipped: usize,
    pub total: usize,
    pub canceled: bool,
}

/// Runs at once: one per core, at most 8. 2 when the cores can't be counted.
pub fn default_roll_parallelism() -> usize {
    std::thread::available_parallelism()
        .map_or(2, |cores| cores.get())
        .min(8)
}

/// Runs `job(i)` for `i` in `0..count` in order, at most `parallel` at once. No job starts once
/// `stop` is set; `cancel` sets it. Jobs see `stop` too, to end early.
pub(crate) async fn run_concurrently<F: Future<Output = ()>>(
    count: usize,
    parallel: usize,
    stop: &AtomicBool,
    cancel: impl Future<Output = ()>,
    job: impl Fn(usize) -> F,
) {
    let next = AtomicUsize::new(0);
    let worker = || async {
        while !stop.load(Ordering::SeqCst) {
            let index = next.fetch_add(1, Ordering::SeqCst);
            if index >= count {
                break;
            }
            job(index).await;
        }
    };
    let mut workers: Vec<_> = (0..parallel.clamp(1, count.max(1)))
        .map(|_| Some(Box::pin(worker())))
        .collect();
    let mut cancel = pin!(cancel);
    let mut cancel_done = false;
    poll_fn(|cx| {
        let was_stopped = stop.load(Ordering::SeqCst);
        if !cancel_done && cancel.as_mut().poll(cx).is_ready() {
            cancel_done = true;
            stop.store(true, Ordering::SeqCst);
        }
        for slot in workers.iter_mut() {
            if slot
                .as_mut()
                .is_some_and(|w| w.as_mut().poll(cx).is_ready())
            {
                *slot = None;
            }
        }
        // Jobs check `stop` only when polled: poll all of them again once it was set.
        if !was_stopped && stop.load(Ordering::SeqCst) {
            cx.waker().wake_by_ref();
        }
        match workers.iter().all(Option::is_none) {
            true => Poll::Ready(()),
            false => Poll::Pending,
        }
    })
    .await
}

fn format_values(values: &[(String, String)]) -> String {
    values
        .iter()
        .map(|(path, value)| format!("{path}={value}"))
        .collect::<Vec<_>>()
        .join(", ")
}

impl Project {
    /// Rolls `paths` of `start` `count` times and runs the already built project with each set,
    /// replacing the stored variations. Numbers with a range in `ranges` roll within it, `mode`
    /// says how far from `start`. Roll 0 runs `start` itself. The first failed roll stops all
    /// others and is the error. Every roll is stored when it finishes.
    #[allow(clippy::too_many_arguments)]
    pub async fn roll_async(
        &self,
        release: bool,
        start: &ProjectParamsListWrapper,
        paths: &[String],
        ranges: &RollRanges,
        mode: RollMode,
        count: usize,
        on_event: impl Fn(RollEvent),
        cancel: impl Future<Output = ()>,
    ) -> Result<RollOutcome> {
        if count == 0 || count > MAX_ROLLS {
            bail!("Roll 1 to {MAX_ROLLS} times, not {count}");
        }
        let paths = check_rollable(start, paths, ranges)?;
        check_roll_mode(start, &paths, mode)?;
        // only what this roll uses is stored with it
        let mut probe = start.clone();
        let ranges: RollRanges = paths
            .iter()
            .filter_map(|path| {
                let range = ranges.get(path)?;
                let value = probe.value_at_path_mut(path).ok()?;
                takes_roll_range(value).then(|| (path.clone(), *range))
            })
            .collect();
        let sets = roll_param_sets(
            start,
            &paths,
            &ranges,
            mode,
            count,
            &mut rand::make_rng::<StdRng>(),
        );
        let runs = sets.len();
        let total = count;
        let meta = VariationsMeta {
            created: Utc::now(),
            params: paths.clone(),
            ranges,
            mode,
            total: count,
            ended: false,
        };
        self.start_variations(&meta)?;
        on_event(RollEvent::Prepared { total });

        let stop = AtomicBool::new(false);
        let canceled = AtomicBool::new(false);
        let done = AtomicUsize::new(0);
        let skipped = AtomicUsize::new(0);
        // the slowest finished roll in milliseconds, 0 before the first
        let slowest = AtomicU64::new(0);
        let limit = || {
            let slowest = slowest.load(Ordering::SeqCst);
            time_limit((slowest > 0).then(|| Duration::from_millis(slowest)))
        };
        let failure: Mutex<Option<String>> = Mutex::new(None);
        let fail = |message: String| {
            failure.lock().unwrap().get_or_insert(message);
            stop.store(true, Ordering::SeqCst);
        };
        let cancel = async {
            cancel.await;
            canceled.store(true, Ordering::SeqCst);
        };

        run_concurrently(runs, default_roll_parallelism(), &stop, cancel, |index| {
            let (sets, paths, stop, done, skipped, slowest, limit, fail, on_event) = (
                &sets, &paths, &stop, &done, &skipped, &slowest, &limit, &fail, &on_event,
            );
            async move {
                let output: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
                let keep_output = |event: RunEvent| {
                    if let RunEvent::Output(line) = event {
                        let mut output = output.lock().unwrap();
                        output.push_back(line);
                        if output.len() > OUTPUT_TAIL {
                            output.pop_front();
                        }
                    }
                };
                let stopped = poll_fn(|_| match stop.load(Ordering::SeqCst) {
                    true => Poll::Ready(()),
                    false => Poll::Pending,
                });
                let started = Instant::now();
                let timed_out: Mutex<Option<Duration>> = Mutex::new(None);
                // the limit grows while this runs, as slower rolls finish
                let too_long = async {
                    let mut ticks = async_io::Timer::interval(Duration::from_millis(250));
                    loop {
                        ticks.next().await;
                        if let Some(limit) = limit().filter(|limit| started.elapsed() > *limit) {
                            *timed_out.lock().unwrap() = Some(limit);
                            return;
                        }
                    }
                };
                let result = self
                    .run_with_params_async(
                        release,
                        &sets[index],
                        None,
                        keep_output,
                        futures_lite::future::or(stopped, too_long),
                    )
                    .await;
                // the run alone, without storing it
                let took = started.elapsed().as_millis().max(1) as u64;
                // roll 0 runs too but is not one of the `count` rolls
                let counted = usize::from(index > 0);
                let name = match index {
                    0 => "Roll 0 (the start params)".to_string(),
                    _ => format!("Roll {index}"),
                };
                // storing renders a png: on a thread of its own, or the other rolls wait for it
                let (project, params) = (self.clone(), sets[index].clone());
                match result {
                    Ok(layer) => {
                        match blocking::unblock(move || project.save_roll(index, &params, &layer))
                            .await
                        {
                            Ok(()) => {
                                slowest.fetch_max(took, Ordering::SeqCst);
                                let done = done.fetch_add(counted, Ordering::SeqCst)
                                    + counted
                                    + skipped.load(Ordering::SeqCst);
                                on_event(RollEvent::Finished { index, done, total });
                            }
                            Err(e) => fail(format!("{name}: storing its result failed: {e}")),
                        }
                    }
                    Err(RunError::Killed { .. }) => {
                        let timed_out = *timed_out.lock().unwrap();
                        if let Some(limit) = timed_out {
                            let stored = blocking::unblock(move || {
                                project.save_skipped_roll(index, &params)
                            })
                            .await;
                            if let Err(e) = stored {
                                return fail(format!("{name}: storing it as skipped failed: {e}"));
                            }
                            let done = skipped.fetch_add(counted, Ordering::SeqCst)
                                + counted
                                + done.load(Ordering::SeqCst);
                            let values = format_values(&rolled_values(&sets[index], paths));
                            on_event(RollEvent::Skipped {
                                index,
                                values,
                                limit,
                                done,
                                total,
                            });
                        }
                    }
                    Err(e) => {
                        let values = format_values(&rolled_values(&sets[index], paths));
                        let output = Vec::from(output.into_inner().unwrap());
                        let tail = match output.is_empty() {
                            true => String::new(),
                            false => format!("\n{}", output.join("\n")),
                        };
                        fail(format!("{name} failed ({values}): {e}{tail}"));
                    }
                }
            }
        })
        .await;

        let failure = failure.into_inner().unwrap();
        let canceled = canceled.load(Ordering::SeqCst);
        let ended = self.end_variations(&meta);
        if let Some(message) = failure {
            bail!(message);
        }
        ended.context("Failed to mark the variations as ended")?;
        Ok(RollOutcome {
            done: done.load(Ordering::SeqCst),
            skipped: skipped.load(Ordering::SeqCst),
            total,
            canceled,
        })
    }
}
