#[cfg(test)]
mod test_image {
    use ::image::{codecs::jpeg::JpegEncoder, ExtendedColorType, ImageEncoder, RgbImage};

    use crate::{ColorRgb, Image, Rect, V2i, V2};

    /// A jpeg with EXIF `Orientation = 6`, stored sideways like a camera does for portrait photos.
    fn jpeg_rotated_90(pixels: &RgbImage) -> Vec<u8> {
        // big-endian TIFF header, one IFD with one entry: tag 0x0112 (orientation), SHORT, count 1, value 6
        let exif = vec![
            0x4d, 0x4d, 0, 42, 0, 0, 0, 8, // header, IFD at offset 8
            0, 1, // one entry
            0x01, 0x12, 0, 3, 0, 0, 0, 1, 0, 6, 0, 0, // orientation = 6
            0, 0, 0, 0, // no next IFD
        ];
        let mut out = Vec::new();
        let mut encoder = JpegEncoder::new_with_quality(&mut out, 100);
        encoder.set_exif_metadata(exif).unwrap();
        encoder
            .write_image(
                pixels.as_raw(),
                pixels.width(),
                pixels.height(),
                ExtendedColorType::Rgb8,
            )
            .unwrap();
        out
    }

    #[test]
    fn applies_exif_orientation() {
        // stored 4x2 with a dark left column, viewed upright it is 2x4 with a dark top row
        let stored = RgbImage::from_fn(4, 2, |x, _| {
            let value = if x == 0 { 0 } else { 255 };
            ::image::Rgb([value, value, value])
        });
        let image = Image::from_bytes(&jpeg_rotated_90(&stored)).unwrap();
        assert_eq!(image.pixel_size(), V2i::new(2, 4));
        for x in 0..2 {
            let top = image.pixel(V2i::new(x, 0)).brightness();
            let bottom = image.pixel(V2i::new(x, 3)).brightness();
            assert!(top < 0.3, "top {top}");
            assert!(bottom > 0.7, "bottom {bottom}");
        }
    }

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
