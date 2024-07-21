use std::sync::{Arc, Mutex};

use dioxus::{hooks::use_signal_sync, signals::SyncSignal};
use plottery_lib::rand_range_i;
use tokio::{sync::mpsc::channel, task::JoinHandle};

#[derive(Debug, Clone)]
pub struct ConsoleMessage {
    id: i32,
    pub msg: String,
    pub is_error: bool,
}

impl ConsoleMessage {
    pub fn new(msg: &str, is_error: bool) -> Self {
        Self {
            id: rand_range_i(i32::MIN, i32::MAX),
            msg: msg.to_string(),
            is_error,
        }
    }
}

impl PartialEq for ConsoleMessage {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Clone)]
pub struct EditorConsole {
    entries: Arc<Mutex<Vec<ConsoleMessage>>>,
    thread: Arc<Mutex<JoinHandle<()>>>,
}

impl EditorConsole {
    pub fn new() -> Self {
        let (sender, mut receiver) = channel::<()>(512);
        let thread = tokio::spawn(async move { while let Some(msg) = receiver.recv().await {} });

        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            thread: Arc::new(Mutex::new(thread)),
        }
    }

    pub fn add(&self, msg: ConsoleMessage) {
        self.entries
            .lock()
            .expect("Failed to lock console entries")
            .push(msg);
    }
    pub fn info(&self, msg: &str) {
        self.entries
            .lock()
            .expect("Failed to lock console entries")
            .push(ConsoleMessage::new(msg, false));
        log::info!("{}", msg);
    }
    pub fn error(&self, msg: &str) {
        self.entries
            .lock()
            .expect("Failed to lock console entries")
            .push(ConsoleMessage::new(msg, true));
        log::error!("{}", msg);
    }

    pub fn get_messages(&self) -> Vec<ConsoleMessage> {
        self.entries
            .lock()
            .expect("Failed to lock console entries")
            .clone()
    }
}
