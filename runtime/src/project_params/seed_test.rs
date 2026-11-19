#[cfg(test)]
mod tests {
    use crate::ProjectParamSeed;

    #[test]
    fn history_is_newest_first_deduplicated_and_capped() {
        let mut seed = ProjectParamSeed::new(0);
        for i in 1..=12 {
            seed.set(i);
        }
        assert_eq!(seed.current, 12);
        assert_eq!(seed.history.len(), ProjectParamSeed::HISTORY_CAP);
        assert_eq!(seed.history[0], 11);

        seed.set(5); // jump back: 12 goes to history, 5 leaves it
        assert_eq!(seed.history[0], 12);
        assert_eq!(seed.history.iter().filter(|s| **s == 5).count(), 0);

        seed.set(5); // no-op
        assert_eq!(seed.history[0], 12);
    }
}
