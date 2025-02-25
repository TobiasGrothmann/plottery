#[macro_use]
extern crate rocket;
mod accelleration_path;
mod accelleration_path_test;
mod hardware;
mod maths;
mod maths_test;
mod pins;
mod speed_delay_handler;
mod task_handler;

#[cfg(feature = "raspi")]
mod system;

use plottery_server_lib::{task::Task, SERVER_PORT};
use rocket::State;
use task_handler::start_server;
use tokio::sync::mpsc::Sender;

#[post("/task", data = "<task_data>")]
async fn task(task_sender: &State<Sender<Task>>, task_data: &str) {
    let task = Task::from_base64(task_data).expect("Failed to decode base64");
    task_sender.send(task).await.expect("Failed to send task");
}

#[rocket::main]
async fn main() {
    #[cfg(feature = "raspi")]
    system::set_realtime_priority();

    let (sender, receiver) = tokio::sync::mpsc::channel::<Task>(32);
    match start_server(receiver).await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to initialize hardware {:?}", e);
            return;
        }
    }

    rocket::build()
        .mount("/", routes![task])
        .manage(sender)
        .configure(
            rocket::Config::figment()
                .merge(("port", SERVER_PORT))
                .merge(("address", "0.0.0.0")),
        )
        .launch()
        .await
        .unwrap();
}
