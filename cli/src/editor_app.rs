use std::path::PathBuf;

const APP_NAME: &str = "PlotteryEditor.app";

/// The installed editor in `/Applications` or `~/Applications`.
pub fn find_editor_app() -> Option<PathBuf> {
    let system = PathBuf::from("/Applications").join(APP_NAME);
    let user = std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join("Applications").join(APP_NAME));
    std::iter::once(system)
        .chain(user)
        .find(|path| path.exists())
}
