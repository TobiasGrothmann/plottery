#[macro_use]
extern crate rocket;
mod server;

use plottery_lib::Layer;
use plottery_server_lib::Task;
use rocket::State;
use server::start_server;
use tokio::sync::mpsc::Sender;

#[post("/plot")]
async fn plot(task_sender: &State<Sender<Task>>) {
    task_sender
        .send(Task::Plot(Layer::new()))
        .await
        .expect("Failed to send task");
}

#[post("/abort")]
async fn abort(task_sender: &State<Sender<Task>>) {
    task_sender
        .send(Task::Abort)
        .await
        .expect("Failed to send task");
}

#[launch]
async fn rocket() -> _ {
    let (sender, receiver) = tokio::sync::mpsc::channel::<Task>(32);
    start_server(receiver).await;

    rocket::build()
        .mount("/", routes![plot, abort])
        .manage(sender)
}
