#[cfg(test)]
mod test_histogram {
    use crate::{Histogram, Rect, V2};

    fn assert_close(a: f32, b: f32, tolerance: f32) {
        assert!((a - b).abs() < tolerance, "{a} vs {b}");
    }

    #[test]
    fn quantile_inverts_cdf() {
        let histogram = Histogram::from_values((0..10_000).map(|i| i as f32 / 10_000.0));
        assert_close(histogram.quantile(0.25), 0.25, 0.01);
        assert_close(histogram.cdf(0.5), 0.5, 0.01);
        assert_close(histogram.quantile(histogram.cdf(0.7)), 0.7, 0.01);
    }

    #[test]
    fn single_bin() {
        let histogram = Histogram::from_values(std::iter::repeat_n(0.3, 100));
        for p in [0.0, 0.5, 1.0] {
            assert_close(histogram.quantile(p), 0.3, 0.01);
        }
    }

    #[test]
    fn quantile_in_empty_gap() {
        let values = std::iter::repeat_n(0.2, 500).chain(std::iter::repeat_n(0.8, 500));
        let histogram = Histogram::from_values(values);
        let cut = histogram.levels(2)[0];
        assert!((0.2..=0.8).contains(&cut), "{cut}");
        assert_close(histogram.cdf(0.5), 0.5, 0.01);
    }

    #[test]
    fn stretch_hits_exact_min_max() {
        let histogram = Histogram::from_values((0..1000).map(|i| 0.2 + 0.4 * i as f32 / 1000.0));
        let stretch = histogram.stretch_fn(0.0, 1.0);
        assert_close(stretch(histogram.min()), 0.0, 1e-6);
        assert_close(stretch(histogram.max()), 1.0, 1e-6);
    }

    #[test]
    fn sample_field_grid() {
        let wide = Rect::new(V2::zero(), V2::new(20.0, 10.0));
        let histogram = Histogram::sample_field(wide, 10, |p| p.x / 20.0);
        assert_eq!(histogram.total(), 50);
        assert_close(histogram.mean(), 0.5, 0.01);

        let tall = Rect::new(V2::zero(), V2::new(1.0, 30.0));
        assert_eq!(Histogram::sample_field(tall, 10, |_| 0.5).total(), 10);
    }
}
