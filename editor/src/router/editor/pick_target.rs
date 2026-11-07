use dioxus::prelude::*;

/// Param path of the `V2` field waiting for a click on the preview, provided as context by
/// the editor so the V2 widget and the preview image can share it.
#[derive(Clone, Copy, PartialEq)]
pub struct PickTarget {
    pub path: Signal<Option<Vec<String>>>,
}

impl PickTarget {
    pub fn is_active(&self) -> bool {
        self.path.read().is_some()
    }

    pub fn is_target(&self, path: &[String]) -> bool {
        self.path.read().as_deref() == Some(path)
    }

    pub fn toggle(&mut self, path: &[String]) {
        let new = if self.is_target(path) {
            None
        } else {
            Some(path.to_vec())
        };
        self.path.set(new);
    }

    /// Returns and clears the target.
    pub fn take(&mut self) -> Option<Vec<String>> {
        self.path.write().take()
    }
}
