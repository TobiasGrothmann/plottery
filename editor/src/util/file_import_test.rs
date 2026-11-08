#[cfg(test)]
mod tests {
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
}
