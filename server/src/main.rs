#[macro_use]
extern crate rocket;

mod accelleration;
mod hardware;
mod pins;
mod task_handler;
mod util;

use plottery_server_lib::{task::Task, HOST_PORT};
use rocket::{
    data::{Limits, ToByteUnit},
    Config, State,
};
use task_handler::start_server;
use tokio::sync::mpsc::Sender;

#[post("/task", data = "<task_data>")]
async fn task(task_sender: &State<Sender<Task>>, task_data: &[u8]) {
    let task = Task::from_binary(task_data).expect("Failed to decode task");
    task_sender
        .send(task)
        .await
        .expect("Failed to process task");
}

#[rocket::main]
async fn main() {
    #[cfg(feature = "raspi")]
    util::system::set_realtime_priority();

    let (sender, receiver) = tokio::sync::mpsc::channel::<Task>(32);
    match start_server(receiver).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to initialize hardware {:?}", e);
            return;
        }
    }

    let data_limit = 1.gigabytes();

    let config = Config::figment()
        .merge(("limits", Limits::default().limit("bytes", data_limit)))
        .merge(("address", "0.0.0.0"))
        .merge(("port", HOST_PORT));

    rocket::custom(config)
        .mount("/", routes![task])
        .manage(sender)
        .launch()
        .await
        .unwrap();
}
