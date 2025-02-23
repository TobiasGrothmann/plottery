#[macro_use]
extern crate rocket;
mod accelleration_path;
mod accelleration_path_test;
mod base64;
mod maths;
mod maths_test;
mod server;
mod system;

use plottery_lib::*;
use plottery_server_lib::Task;
use rocket::State;
use server::start_server;
use tokio::sync::mpsc::Sender;

#[post("/plot_shape", data = "<shape_data>")]
async fn plot_shape(task_sender: &State<Sender<Task>>, shape_data: &str) {
    let shape = Shape::new_from_base64(shape_data).expect("Failed to decode base64");
    task_sender
        .send(Task::PlotShape(shape))
        .await
        .expect("Failed to send task");
}

#[post("/plot", data = "<layer_data>")]
async fn plot(task_sender: &State<Sender<Task>>, layer_data: &str) {
    let layer = Layer::new_from_base64(layer_data).expect("Failed to decode base64");
    task_sender
        .send(Task::Plot(layer))
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
    system::set_realtime_priority();

    let (sender, receiver) = tokio::sync::mpsc::channel::<Task>(32);
    start_server(receiver).await;

    rocket::build()
        .mount("/", routes![plot, abort, plot_shape])
        .manage(sender)
}
