use dioxus::prelude::*;

/// What a click on the preview sets in the param at a [`Pick`]'s path.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PickPart {
    /// A `V2`.
    Point,
    /// A corner of a `Rect`.
    BottomLeft,
    TopRight,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Pick {
    pub path: Vec<String>,
    pub part: PickPart,
}

/// The param waiting for a click on the preview, provided as context by the editor so the
/// param widgets and the preview image can share it.
#[derive(Clone, Copy, PartialEq)]
pub struct PickTarget {
    pub target: Signal<Option<Pick>>,
}

impl PickTarget {
    pub fn is_active(&self) -> bool {
        self.target.read().is_some()
    }

    pub fn is_target(&self, path: &[String], part: PickPart) -> bool {
        self.target
            .read()
            .as_ref()
            .is_some_and(|pick| pick.path == path && pick.part == part)
    }

    pub fn toggle(&mut self, path: &[String], part: PickPart) {
        let new = (!self.is_target(path, part)).then(|| Pick {
            path: path.to_vec(),
            part,
        });
        self.target.set(new);
    }

    /// Returns and clears the target.
    pub fn take(&mut self) -> Option<Pick> {
        self.target.write().take()
    }
}
