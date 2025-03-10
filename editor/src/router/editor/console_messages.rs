use std::sync::{Arc, Mutex};

use dioxus::signals::{Readable, SyncSignal};
use plottery_lib::rand_range_i;

#[derive(Debug, Clone)]
pub struct ConsoleMessage {
    pub id: i32,
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

    pub fn info(&self, msg: &str) {
        self.entries
            .lock()
            .expect("Failed to lock console entries")
            .push(ConsoleMessage::new(msg, false));
        log::info!("{}", msg);
        self.trigger_change();
    }
    pub fn error(&self, msg: &str) {
        self.entries
            .lock()
            .expect("Failed to lock console entries")
            .push(ConsoleMessage::new(msg, true));
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
