#[cfg(test)]
mod tests {
    use std::future::{pending, ready};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Mutex;

    use futures_lite::future::{block_on, yield_now};
    use plottery_runtime::ProjectParamsListWrapper;

    use crate::variations_run::run_concurrently;
    use crate::{Project, ProjectConfig};

    #[test]
    fn runs_every_index_once_and_never_more_than_parallel() {
        let active = AtomicUsize::new(0);
        let most = AtomicUsize::new(0);
        let started = Mutex::new(Vec::new());
        let stop = AtomicBool::new(false);
        block_on(run_concurrently(20, 3, &stop, pending(), |index| {
            let (active, most, started) = (&active, &most, &started);
            async move {
                started.lock().unwrap().push(index);
                let now = active.fetch_add(1, Ordering::SeqCst) + 1;
                most.fetch_max(now, Ordering::SeqCst);
                for _ in 0..index % 4 + 1 {
                    yield_now().await;
                }
                active.fetch_sub(1, Ordering::SeqCst);
            }
        }));
        let mut started = started.into_inner().unwrap();
        started.sort();
        assert_eq!(started, (0..20).collect::<Vec<_>>());
        assert_eq!(most.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn nothing_starts_after_a_job_stops_or_a_cancel() {
        let started = Mutex::new(Vec::new());
        let stop = AtomicBool::new(false);
        block_on(run_concurrently(10, 1, &stop, pending(), |index| {
            let (started, stop) = (&started, &stop);
            async move {
                started.lock().unwrap().push(index);
                if index == 2 {
                    stop.store(true, Ordering::SeqCst);
                }
            }
        }));
        assert_eq!(started.into_inner().unwrap(), vec![0, 1, 2]);

        let started = AtomicUsize::new(0);
        block_on(run_concurrently(
            10,
            4,
            &AtomicBool::new(false),
            ready(()),
            |_| {
                let started = &started;
                async move {
                    started.fetch_add(1, Ordering::SeqCst);
                }
            },
        ));
        assert_eq!(started.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn a_running_job_sees_the_stop_of_another() {
        // job 0 waits until job 1 stops everything; it must be polled again to notice
        let stop = AtomicBool::new(false);
        let noticed = AtomicBool::new(false);
        block_on(run_concurrently(2, 2, &stop, pending(), |index| {
            let (stop, noticed) = (&stop, &noticed);
            async move {
                match index {
                    0 => {
                        futures_lite::future::poll_fn(|_| match stop.load(Ordering::SeqCst) {
                            true => std::task::Poll::Ready(()),
                            false => std::task::Poll::Pending,
                        })
                        .await;
                        noticed.store(true, Ordering::SeqCst);
                    }
                    _ => stop.store(true, Ordering::SeqCst),
                }
            }
        }));
        assert!(noticed.load(Ordering::SeqCst));
    }

    #[test]
    fn rolling_can_run_on_a_multithreaded_runtime() {
        fn assert_send<T: Send>(_: T) {}
        let project = Project {
            config: ProjectConfig::new("sketch"),
            dir: std::env::temp_dir(),
        };
        let start = ProjectParamsListWrapper::empty();
        assert_send(project.roll_async(true, &start, &[], 1, |_| {}, pending()));
    }
}
