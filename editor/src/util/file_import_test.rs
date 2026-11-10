#[cfg(test)]
mod tests {
    use image::{codecs::jpeg::JpegEncoder, ExtendedColorType, ImageDecoder, ImageEncoder};

    use crate::util::file_import::{import_image, resolve_stored_path, IMAGES_DIR};

    fn png_bytes(width: u32, height: u32, shade: u8) -> Vec<u8> {
        let image = image::RgbImage::from_pixel(width, height, image::Rgb([shade, shade, shade]));
        let mut out = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgb8(image)
            .write_to(&mut out, image::ImageFormat::Png)
            .unwrap();
        out.into_inner()
    }

    #[test]
    fn copies_reuses_and_uniquifies() {
        let project = tempfile::tempdir().unwrap();
        let source_dir = tempfile::tempdir().unwrap();
        let source = source_dir.path().join("pic.png");
        std::fs::write(&source, png_bytes(4, 4, 10)).unwrap();

        let first = import_image(project.path(), &source, false).unwrap();
        assert_eq!(first, format!("{IMAGES_DIR}/pic.png"));
        assert!(resolve_stored_path(project.path(), &first).exists());

        // same file again: reused
        let again = import_image(project.path(), &source, false).unwrap();
        assert_eq!(again, first);

        // different file with the same name: new name
        std::fs::write(&source, png_bytes(4, 4, 200)).unwrap();
        let other = import_image(project.path(), &source, false).unwrap();
        assert_eq!(other, format!("{IMAGES_DIR}/pic_1.png"));
    }

    #[test]
    fn downscales_large_images() {
        let project = tempfile::tempdir().unwrap();
        let source_dir = tempfile::tempdir().unwrap();
        let source = source_dir.path().join("big.png");
        std::fs::write(&source, png_bytes(3000, 1500, 0)).unwrap();

        let stored = import_image(project.path(), &source, true).unwrap();
        let copied = image::open(resolve_stored_path(project.path(), &stored)).unwrap();
        assert_eq!((copied.width(), copied.height()), (2000, 1000));
    }

    /// A jpeg with EXIF `Orientation = 6`, stored sideways like a camera does for portrait photos.
    fn jpeg_rotated_90(width: u32, height: u32) -> Vec<u8> {
        // big-endian TIFF header, one IFD with one entry: tag 0x0112 (orientation), SHORT, count 1, value 6
        let exif = vec![
            0x4d, 0x4d, 0, 42, 0, 0, 0, 8, // header, IFD at offset 8
            0, 1, // one entry
            0x01, 0x12, 0, 3, 0, 0, 0, 1, 0, 6, 0, 0, // orientation = 6
            0, 0, 0, 0, // no next IFD
        ];
        let pixels = image::RgbImage::from_pixel(width, height, image::Rgb([128, 128, 128]));
        let mut out = Vec::new();
        let mut encoder = JpegEncoder::new(&mut out);
        encoder.set_exif_metadata(exif).unwrap();
        encoder
            .write_image(pixels.as_raw(), width, height, ExtendedColorType::Rgb8)
            .unwrap();
        out
    }

    #[test]
    fn downscaled_photos_stay_upright() {
        let project = tempfile::tempdir().unwrap();
        let source_dir = tempfile::tempdir().unwrap();
        let source = source_dir.path().join("portrait.jpg");
        std::fs::write(&source, jpeg_rotated_90(3000, 1500)).unwrap();

        let stored = import_image(project.path(), &source, true).unwrap();
        let mut decoder = image::ImageReader::open(resolve_stored_path(project.path(), &stored))
            .unwrap()
            .into_decoder()
            .unwrap();
        // the pixels themselves are upright, without an orientation tag to rotate them again
        assert_eq!(decoder.dimensions(), (1000, 2000));
        assert_eq!(
            decoder.orientation().unwrap(),
            image::metadata::Orientation::NoTransforms
        );
    }
}
