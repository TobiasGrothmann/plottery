#[cfg(test)]
mod test_color {
    use crate::ColorRgb;

    #[test]
    fn conversion() {
        let rgb = ColorRgb {
            r: 0.3,
            g: 0.9,
            b: 0.5,
        };
        let hsv = rgb.hsv();
        let rgb2 = hsv.rgb();
        assert_eq!(rgb, rgb2);
        assert_eq!(rgb, hsv);
        assert_eq!(rgb2, hsv);
    }
}
