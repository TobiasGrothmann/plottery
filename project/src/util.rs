pub fn export_plottery_home(path: &str) -> Result<(), std::io::Error> {
    set_env::check_or_set("PLOTTERY_HOME", path)
}
