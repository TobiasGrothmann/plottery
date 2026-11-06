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

    fn assert_close(a: f32, b: f32, tolerance: f32) {
        assert!((a - b).abs() < tolerance, "{a} vs {b}");
    }

    #[test]
    fn oklab_black_white() {
        let black = ColorRgb::black().oklab();
        assert_close(black.l, 0.0, 1e-3);
        let white = ColorRgb::white().oklab();
        assert_close(white.l, 1.0, 1e-3);
        assert_close(white.a, 0.0, 1e-3);
        assert_close(white.b, 0.0, 1e-3);
    }

    #[test]
    fn gray_metrics() {
        let gray = ColorRgb::gray_scale(0.5);
        assert_close(gray.luminance(), 0.214, 1e-3);
        assert_close(gray.lightness(), 0.598, 1e-3);
        assert_close(gray.oklab().chroma(), 0.0, 1e-4);
    }

    #[test]
    fn oklab_round_trip() {
        let colors = [
            ColorRgb::red(),
            ColorRgb::green(),
            ColorRgb::blue(),
            ColorRgb::yellow(),
            ColorRgb::cyan(),
            ColorRgb::magenta(),
            ColorRgb::gray(),
            ColorRgb::new(0.3, 0.9, 0.5),
            ColorRgb::new(0.12, 0.05, 0.7),
        ];
        for color in colors {
            let back = color.oklab().rgb();
            assert_close(back.r, color.r, 1e-3);
            assert_close(back.g, color.g, 1e-3);
            assert_close(back.b, color.b, 1e-3);
        }
    }
}
