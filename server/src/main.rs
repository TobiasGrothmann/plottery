#[macro_use]
extern crate rocket;

mod accelleration;
mod hardware;
mod pins;
mod task_handler;
mod util;

use std::sync::{Arc, Mutex};

use plottery_server_lib::{server_state::ServerState, task::Task, HOST_PORT};
use rocket::{
    data::{Limits, ToByteUnit},
    Config, State,
};
use task_handler::start_server;
use tokio::sync::mpsc::{channel, Sender};

struct ManagedState {
    task_sender: Sender<Task>,
    server_state: Arc<Mutex<ServerState>>,
}

#[post("/task", data = "<task_data>")]
async fn set_task(managed_state: &State<ManagedState>, task_data: &[u8]) {
    let task = Task::from_binary(task_data).expect("Failed to decode task");
    managed_state
        .task_sender
        .send(task)
        .await
        .expect("Failed to process task");
}

#[get("/state")]
async fn get_state(managed_state: &State<ManagedState>) -> Vec<u8> {
    let state = managed_state.server_state.lock().unwrap();
    state.to_binary().expect("Failed to encode state")
}

#[rocket::main]
async fn main() {
    #[cfg(feature = "raspi")]
    util::system::set_realtime_priority();

    let (task_sender, task_receiver) = channel::<Task>(32);
    let server_state = Arc::new(Mutex::new(ServerState::default()));
    let managed_state = ManagedState {
        task_sender,
        server_state: server_state.clone(),
    };

    match start_server(task_receiver, server_state).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to initialize hardware {:?}", e);
            return;
        }
    }

    let config = Config::figment()
        .merge(("limits", Limits::default().limit("bytes", 1.gigabytes())))
        .merge(("address", "0.0.0.0"))
        .merge(("port", HOST_PORT));

    rocket::custom(config)
        .mount("/", routes![set_task, get_state])
        .manage(managed_state)
        .launch()
        .await
        .unwrap();
}
