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

    #[test]
    fn name() {
        assert_eq!(
            ColorRgb {
                r: 0.6,
                g: 0.4,
                b: 0.8
            }
            .get_name()
            .name,
            "Amethyst"
        );
        assert_eq!(
            ColorRgb {
                r: 0.63,
                g: 0.405,
                b: 0.79
            }
            .get_name()
            .name,
            "Amethyst"
        );
        assert_eq!(
            ColorRgb {
                r: 0.343,
                g: 0.1,
                b: 0.8
            }
            .get_name(),
            ColorRgb {
                r: 0.32,
                g: 0.09,
                b: 0.81
            }
            .get_name()
        );
    }
}
