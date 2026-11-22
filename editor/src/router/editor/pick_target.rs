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
    /// The pickable field under the mouse, marked on the preview.
    pub hovered: Signal<Option<Pick>>,
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

    pub fn hover(&mut self, path: &[String], part: PickPart) {
        self.hovered.set(Some(Pick {
            path: path.to_vec(),
            part,
        }));
    }

    pub fn unhover(&mut self) {
        self.hovered.set(None);
    }

    /// Clears the hover if it is on `path`, for a field that unmounts under the mouse.
    pub fn unhover_path(&mut self, path: &[String]) {
        if self
            .hovered
            .peek()
            .as_ref()
            .is_some_and(|pick| pick.path == path)
        {
            self.hovered.set(None);
        }
    }

    /// What the preview marks: the hovered field, else the one waiting for a click.
    pub fn marked(&self) -> Option<Pick> {
        self.hovered
            .read()
            .clone()
            .or_else(|| self.target.read().clone())
    }

    /// Returns and clears the target.
    pub fn take(&mut self) -> Option<Pick> {
        self.target.write().take()
    }
}
