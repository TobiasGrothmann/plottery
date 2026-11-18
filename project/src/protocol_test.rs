#[cfg(test)]
mod tests {
    use crate::protocol::check_protocol_answer;
    use plottery_runtime::PROTOCOL_VERSION;

    #[test]
    fn same_version_is_accepted() {
        let answer = format!("{PROTOCOL_VERSION}\n");
        assert!(check_protocol_answer(true, answer.as_bytes()).is_ok());
    }

    #[test]
    fn other_version_names_both() {
        let other = PROTOCOL_VERSION + 1;
        let err = check_protocol_answer(true, format!("{other}\n").as_bytes()).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains(&format!("built with protocol version {other}")));
        assert!(msg.contains(&format!("uses protocol version {PROTOCOL_VERSION}")));
        assert!(msg.contains("Rebuild the project"));
    }

    #[test]
    fn binary_without_the_subcommand_is_a_mismatch() {
        // clap exits with an error and prints nothing to stdout
        for (success, stdout) in [(false, ""), (true, ""), (true, "hello"), (false, "1")] {
            let err = check_protocol_answer(success, stdout.as_bytes()).unwrap_err();
            assert!(err.to_string().contains("unknown protocol version"));
        }
    }
}
