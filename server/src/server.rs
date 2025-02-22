use plottery_server_lib::Task;
use tokio::sync::mpsc;
use tokio::task;

pub async fn start_server(mut receiver: mpsc::Receiver<Task>) {
    task::spawn(async move {
        while let Some(task) = receiver.recv().await {
            println!("Received task: {:?}", task);
        }
    });
}
