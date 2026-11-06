#[cfg(test)]
mod test_image {
    use ::image::RgbImage;

    use crate::{ColorRgb, Image, Rect, V2};

    #[test]
    fn histogram_in_region() {
        let image = Image::from_rgb(RgbImage::from_fn(16, 8, |x, _| {
            let value = (x * 255 / 15) as u8;
            ::image::Rgb([value, value, value])
        }));
        let left_half = Rect::new(V2::zero(), V2::new(8.0, 8.0));
        let histogram = image.histogram_in(left_half, 8, ColorRgb::brightness);
        assert_eq!(histogram.total(), 64);
        assert!(histogram.max() < 0.6, "{}", histogram.max());
    }
}
