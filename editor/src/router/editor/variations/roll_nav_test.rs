#[cfg(test)]
mod tests {
    use crate::router::editor::variations::roll_nav::step_roll;

    #[test]
    fn steps_past_tiles_that_cant_be_picked() {
        // the current drawing, the start, a skipped roll, rolls 2 and 3
        let tiles = [None, Some(0), None, Some(2), Some(3)];
        assert_eq!(step_roll(&tiles, 0, 1), Some(0));
        assert_eq!(step_roll(&tiles, 1, 1), Some(2));
        assert_eq!(step_roll(&tiles, 3, -1), Some(0));
        assert_eq!(step_roll(&tiles, 1, -1), None);
        assert_eq!(step_roll(&tiles, 4, 1), None);
    }

    #[test]
    fn steps_a_row() {
        let tiles: Vec<Option<usize>> = (0..7).map(Some).collect();
        assert_eq!(step_roll(&tiles, 1, 3), Some(4));
        assert_eq!(step_roll(&tiles, 4, -3), Some(1));
        assert_eq!(step_roll(&tiles, 2, -3), None);
        assert_eq!(step_roll(&tiles, 5, 3), None);
    }
}
