#[cfg(test)]
mod tests {
    use super::super::roll_tile::neat_value;

    #[test]
    fn values_read_short() {
        assert_eq!(neat_value("34.628deg"), "34.63°");
        assert_eq!(neat_value("0.08543266..0.25299454"), "0.09..0.25");
        assert_eq!(neat_value("-1.5,2.004"), "-1.5,2");
        assert_eq!(neat_value("2..5"), "2..5");
        assert_eq!(neat_value("Sunflower"), "Sunflower");
        assert_eq!(neat_value("491"), "491");
    }
}
