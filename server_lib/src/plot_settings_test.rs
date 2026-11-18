#[cfg(test)]
mod tests {
    use crate::PlotSettings;

    fn settings() -> PlotSettings {
        let mut settings = PlotSettings::default();
        settings.speed_draw.max = 1.25;
        settings
    }

    #[test]
    fn round_trip() {
        let bytes = settings().to_file_bytes().unwrap();
        assert_eq!(&bytes[..6], b"PLTS\x01\x00");
        let read = PlotSettings::from_file_bytes(&bytes).unwrap();
        assert_eq!(read.to_file_bytes().unwrap(), bytes);
    }

    #[test]
    fn headerless_file_is_read_as_the_old_layout() {
        let legacy = bincode::serialize(&settings()).unwrap();
        let read = PlotSettings::from_file_bytes(&legacy).unwrap();
        assert_eq!(read.speed_draw.max, 1.25);
    }

    #[test]
    fn newer_version_and_garbage_are_errors() {
        let mut newer = settings().to_file_bytes().unwrap();
        newer[4] = 2;
        let err = PlotSettings::from_file_bytes(&newer).unwrap_err();
        assert!(err.to_string().contains("format version 2"));
        assert!(PlotSettings::from_file_bytes(&[1, 2, 3]).is_err());
    }

    #[test]
    fn save_keeps_a_file_it_cannot_read() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.plots");

        // no file: defaults, and saving creates it
        assert!(PlotSettings::load_file(&path).is_ok());
        settings().save_file(&path).unwrap();
        assert_eq!(PlotSettings::load_file(&path).unwrap().speed_draw.max, 1.25);

        std::fs::write(&path, [1, 2, 3]).unwrap();
        assert!(PlotSettings::load_file(&path).is_err());
        assert!(settings().save_file(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), [1, 2, 3]);
    }
}
