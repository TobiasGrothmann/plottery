//! Rolling: runs the built project once per roll, a few at a time, and stores the results.

use std::collections::VecDeque;
use std::future::Future;
use std::pin::pin;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::task::Poll;

use anyhow::{bail, Result};
use chrono::Utc;
use futures_lite::future::poll_fn;
use plottery_runtime::ProjectParamsListWrapper;
use rand::rngs::StdRng;

use crate::{
    check_rollable, roll_param_sets, rolled_values, Project, RunError, RunEvent, VariationsMeta,
};

/// Most rolls in one set.
pub const MAX_ROLLS: usize = 200;

/// Output lines kept per roll for the error of a failed one.
const OUTPUT_TAIL: usize = 20;

#[derive(Debug, Clone)]
pub enum RollEvent {
    /// The params of all `total` rolls, start included, are stored and the runs begin.
    Prepared { total: usize },
    /// Roll `index` finished and its SVG and PNG are stored.
    Finished {
        index: usize,
        done: usize,
        total: usize,
    },
}

/// How a set of rolls ended. A failed roll is an error instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RollOutcome {
    pub done: usize,
    pub total: usize,
    pub canceled: bool,
}

/// Runs at once: half the cores, 1 to 8.
pub fn default_roll_parallelism() -> usize {
    std::thread::available_parallelism()
        .map_or(2, |cores| cores.get() / 2)
        .clamp(1, 8)
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
    /// replacing the stored variations. Roll 0 runs `start` itself. The first failed roll stops
    /// all others and is the error; only finished rolls stay stored.
    pub async fn roll_async(
        &self,
        release: bool,
        start: &ProjectParamsListWrapper,
        paths: &[String],
        count: usize,
        on_event: impl Fn(RollEvent),
        cancel: impl Future<Output = ()>,
    ) -> Result<RollOutcome> {
        if count == 0 || count > MAX_ROLLS {
            bail!("Roll 1 to {MAX_ROLLS} times, not {count}");
        }
        let paths = check_rollable(start, paths)?;
        let sets = roll_param_sets(start, &paths, count, &mut rand::make_rng::<StdRng>());
        let total = sets.len();
        let meta = VariationsMeta {
            created: Utc::now(),
            params: paths.clone(),
        };
        self.start_variations(&meta, &sets)?;
        on_event(RollEvent::Prepared { total });

        let stop = AtomicBool::new(false);
        let canceled = AtomicBool::new(false);
        let done = AtomicUsize::new(0);
        let failure: Mutex<Option<String>> = Mutex::new(None);
        let fail = |message: String| {
            failure.lock().unwrap().get_or_insert(message);
            stop.store(true, Ordering::SeqCst);
        };
        let cancel = async {
            cancel.await;
            canceled.store(true, Ordering::SeqCst);
        };

        run_concurrently(total, default_roll_parallelism(), &stop, cancel, |index| {
            let (sets, paths, stop, done, fail, on_event) =
                (&sets, &paths, &stop, &done, &fail, &on_event);
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
                let result = self
                    .run_with_params_async(release, &sets[index], None, keep_output, stopped)
                    .await;
                let name = match index {
                    0 => "Roll 0 (the start params)".to_string(),
                    _ => format!("Roll {index}"),
                };
                match result {
                    Ok(layer) => match self.save_roll_output(index, &layer) {
                        Ok(()) => {
                            let done = done.fetch_add(1, Ordering::SeqCst) + 1;
                            on_event(RollEvent::Finished { index, done, total });
                        }
                        Err(e) => fail(format!("{name}: storing its result failed: {e}")),
                    },
                    Err(RunError::Killed { .. }) => {}
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
        if failure.is_some() || canceled {
            self.remove_unfinished_rolls()?;
        }
        if let Some(message) = failure {
            bail!(message);
        }
        Ok(RollOutcome {
            done: done.load(Ordering::SeqCst),
            total,
            canceled,
        })
    }
}
