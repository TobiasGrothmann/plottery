#[macro_use]
extern crate rocket;

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

use job::JobState;
use plottery_server_lib::{
    discovery::{self, Advertisement},
    task::Task,
    Telemetry,
};
use rocket::{
    data::{Limits, ToByteUnit},
    http::Status,
    Config, State,
};
use task_handler::start_server;
use tokio::sync::mpsc::{channel, Sender};

struct ManagedState {
    task_sender: Sender<Task>,
    telemetry: Arc<Telemetry>,
    job: Arc<Mutex<JobState>>,
    abort: Arc<AtomicBool>,
}

#[post("/task", data = "<task_data>")]
async fn set_task(
    managed_state: &State<ManagedState>,
    task_data: &[u8],
) -> Result<(), (Status, String)> {
    let task =
        Task::from_binary(task_data).map_err(|err| (Status::BadRequest, format!("{err:#}")))?;
    if matches!(task, Task::Abort) {
        managed_state.abort.store(true, Ordering::Relaxed);
        return Ok(());
    }
    managed_state.task_sender.send(task).await.map_err(|_| {
        (
            Status::InternalServerError,
            "The task handler has stopped, restart the server.".to_string(),
        )
    })
}

#[get("/state")]
async fn get_state(managed_state: &State<ManagedState>) -> Vec<u8> {
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

    let config = Config::figment()
        .merge(("limits", Limits::default().limit("bytes", 1.gigabytes())))
        .merge(("address", "0.0.0.0"))
        .merge(("port", discovery::PORT));

    // Dropped after launch returns, e.g. on SIGTERM, which tells browsers the server is gone.
    let _advertisement = match Advertisement::start() {
        Ok(advertisement) => advertisement,
        Err(e) => {
            eprintln!("Failed to advertise the server with mDNS {:?}", e);
            return;
        }
    };

    rocket::custom(config)
        .mount("/", routes![set_task, get_state])
        .manage(managed_state)
        .launch()
        .await
        .expect("Failed to launch Rocket server");
}
