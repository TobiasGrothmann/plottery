use std::sync::{Arc, Mutex};

use dioxus::signals::{ReadableExt, SyncSignal};
use plottery_lib::rand_range_i;

#[derive(Debug, Clone)]
pub enum ConsoleMessageType {
    Info,
    Error,
    ProjectLog,
}

#[derive(Debug, Clone)]
pub struct ConsoleMessage {
    pub id: i32,
    pub msg: String,
    pub msg_type: ConsoleMessageType,
}

impl ConsoleMessage {
    pub fn new(msg: &str, msg_type: ConsoleMessageType) -> Self {
        Self {
            id: rand_range_i(i32::MIN, i32::MAX),
            msg: msg.to_string(),
            msg_type,
        }
    }
}

impl PartialEq for ConsoleMessage {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Clone)]
pub struct ConsoleMessages {
    entries: Arc<Mutex<Vec<ConsoleMessage>>>,
    change_counter: Arc<Mutex<SyncSignal<u32>>>,
}

impl ConsoleMessages {
    pub fn new(change_counter: SyncSignal<u32>) -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            change_counter: Arc::new(Mutex::new(change_counter)),
        }
    }
    fn trigger_change(&self) {
        let mut change_counter = self
            .change_counter
            .lock()
            .expect("Failed to lock change counter");
        *change_counter += 1;
    }

    pub fn get_change_counter(&self) -> u32 {
        let sig = *self
            .change_counter
            .lock()
            .expect("Failed to lock change counter");
        return *sig.read();
    }

    pub fn clear(&self) {
        self.entries
            .lock()
            .expect("Failed to lock console entries")
            .clear();
        self.trigger_change();
    }

    pub fn project_message(&self, msg: &str) {
        let formatted = format!("> {}", msg);
        self.entries
            .lock()
            .expect("Failed to lock console entries")
            .push(ConsoleMessage::new(
                &formatted,
                ConsoleMessageType::ProjectLog,
            ));
        log::info!("{}", formatted);
        self.trigger_change();
    }
    pub fn info(&self, msg: &str) {
        self.entries
            .lock()
            .expect("Failed to lock console entries")
            .push(ConsoleMessage::new(msg, ConsoleMessageType::Info));
        log::info!("{}", msg);
        self.trigger_change();
    }
    pub fn error(&self, msg: &str) {
        self.entries
            .lock()
            .expect("Failed to lock console entries")
            .push(ConsoleMessage::new(msg, ConsoleMessageType::Error));
        log::error!("{}", msg);
        self.trigger_change();
    }

    pub fn get_messages(&self) -> Vec<ConsoleMessage> {
        self.entries
            .lock()
            .expect("Failed to lock console entries")
            .clone()
    }
}
