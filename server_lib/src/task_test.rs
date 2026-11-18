#[cfg(test)]
mod tests {
    use crate::{Task, TASK_PROTOCOL_VERSION};

    #[test]
    fn round_trip() {
        let binary = Task::SetHead(true).to_binary().unwrap();
        assert!(matches!(
            Task::from_binary(&binary).unwrap(),
            Task::SetHead(true)
        ));
    }

    #[test]
    fn other_version_names_both() {
        let mut binary = Task::Abort.to_binary().unwrap();
        binary[4..6].copy_from_slice(&(TASK_PROTOCOL_VERSION + 1).to_le_bytes());
        let msg = Task::from_binary(&binary).unwrap_err().to_string();
        assert!(msg.contains(&format!("version {}", TASK_PROTOCOL_VERSION + 1)));
        assert!(msg.contains(&format!("server version {TASK_PROTOCOL_VERSION}")));
    }

    #[test]
    fn unversioned_and_damaged_tasks_are_errors() {
        // a task as sent before requests had a version
        let unversioned = bincode::serialize(&Task::Abort).unwrap();
        let msg = Task::from_binary(&unversioned).unwrap_err().to_string();
        assert!(msg.contains("no protocol version"));

        let mut damaged = Task::SetHead(true).to_binary().unwrap();
        damaged.truncate(8);
        assert!(Task::from_binary(&damaged).is_err());
    }
}
