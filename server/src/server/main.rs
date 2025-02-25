#[macro_use]
extern crate rocket;
mod accelleration_path;
mod accelleration_path_test;
mod hardware;
mod maths;
mod maths_test;
mod pins;
mod speed_delay_handler;
mod system;
mod task_handler;

use plottery_server_lib::task::Task;
use rocket::State;
use task_handler::start_server;
use tokio::sync::mpsc::Sender;

#[post("/task", data = "<task_data>")]
async fn task(task_sender: &State<Sender<Task>>, task_data: &str) {
    let task = Task::from_base64(task_data).expect("Failed to decode base64");
    task_sender.send(task).await.expect("Failed to send task");
}

#[launch]
async fn rocket() -> _ {
    system::set_realtime_priority();

    let (sender, receiver) = tokio::sync::mpsc::channel::<Task>(32);
    start_server(receiver).await;

    rocket::build().mount("/", routes![task]).manage(sender)
}
