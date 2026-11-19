#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use anyhow::Result;

    use crate::PlotteryConfig;

    #[test]
    fn home_survives_a_save_and_a_missing_file_is_the_default() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("nested/config.toml");
        assert_eq!(PlotteryConfig::load_from(&path)?, PlotteryConfig::default());

        let config = PlotteryConfig {
            plottery_home: Some(PathBuf::from("/checkout/plottery")),
        };
        config.save_to(&path)?;
        assert_eq!(PlotteryConfig::load_from(&path)?, config);
        Ok(())
    }

    #[test]
    fn env_var_overrides_the_file_unless_empty() {
        let from_file = PathBuf::from("/from/file");
        let config = PlotteryConfig {
            plottery_home: Some(from_file.clone()),
        };
        assert_eq!(
            config.home_with_override(Some("/from/env".to_string())),
            Some(PathBuf::from("/from/env"))
        );
        assert_eq!(
            config.home_with_override(Some(String::new())),
            Some(from_file.clone())
        );
        assert_eq!(config.home_with_override(None), Some(from_file));
        assert_eq!(PlotteryConfig::default().home_with_override(None), None);
    }
}
