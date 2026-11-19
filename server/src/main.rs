mod gpio_executor;
mod gpio_executor_test;
mod job;
mod task_handler;
mod util;

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use job::JobState;
use plottery_server_lib::{
    discovery::{self, Advertisement},
    task::Task,
    Telemetry,
};
use task_handler::start_server;
use tokio::sync::mpsc::{channel, Sender};

struct ManagedState {
    task_sender: Sender<Task>,
    telemetry: Arc<Telemetry>,
    job: Arc<Mutex<JobState>>,
    abort: Arc<AtomicBool>,
}

async fn set_task(
    State(managed_state): State<Arc<ManagedState>>,
    task_data: Bytes,
) -> Result<(), (StatusCode, String)> {
    let task = Task::from_binary(&task_data)
        .map_err(|err| (StatusCode::BAD_REQUEST, format!("{err:#}")))?;
    if matches!(task, Task::Abort) {
        managed_state.abort.store(true, Ordering::Relaxed);
        return Ok(());
    }
    managed_state.task_sender.send(task).await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "The task handler has stopped, restart the server.".to_string(),
        )
    })
}

async fn get_state(State(managed_state): State<Arc<ManagedState>>) -> Vec<u8> {
    let reading = managed_state.telemetry.read();
    let aborting = managed_state.abort.load(Ordering::Relaxed);
    let state = managed_state
        .job
        .lock()
        .expect("Failed to acquire job state lock")
        .snapshot(reading, aborting, Instant::now());
    state
        .to_binary()
        .expect("Failed to encode server state to binary format")
}

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build tokio runtime");

    runtime.block_on(async_main());
}

async fn async_main() {
    let (task_sender, task_receiver) = channel::<Task>(32);
    let telemetry = Arc::new(Telemetry::default());
    // Plot ids continue from the start time, so they differ from those before a restart.
    let first_plot_id = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since_epoch| since_epoch.as_secs() as u32);
    let job = Arc::new(Mutex::new(JobState::new(first_plot_id)));
    let abort = Arc::new(AtomicBool::new(false));
    let managed_state = ManagedState {
        task_sender,
        telemetry: telemetry.clone(),
        job: job.clone(),
        abort: abort.clone(),
    };

    match start_server(task_receiver, telemetry, job, abort) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to initialize hardware {:?}", e);
            return;
        }
    }

    // Dropped when the server returns, e.g. on SIGTERM, which tells browsers the server is gone.
    let _advertisement = match Advertisement::start() {
        Ok(advertisement) => advertisement,
        Err(e) => {
            eprintln!("Failed to advertise the server with mDNS {:?}", e);
            return;
        }
    };

    let app = Router::new()
        .route("/task", post(set_task))
        .route("/state", get(get_state))
        .layer(DefaultBodyLimit::max(1 << 30))
        .with_state(Arc::new(managed_state));
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", discovery::PORT))
        .await
        .expect("Failed to bind the server port");
    println!("Listening on port {}", discovery::PORT);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Failed to run the server");
}

/// Resolves on Ctrl-C or SIGTERM (systemd stop).
async fn shutdown_signal() {
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("Failed to listen for SIGTERM");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = terminate.recv() => {}
    }
}
