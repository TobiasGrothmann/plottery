use std::fmt;

use serde::{Deserialize, Serialize};

/// Seed for the global random number generator, see [`plottery_lib::seed`].
///
/// ```
/// # use plottery_project::Seed;
/// let seed = Seed::new(42);
/// seed.apply(); // all plottery_lib random functions are now deterministic
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Seed {
    pub value: i32,
}

impl Seed {
    pub fn new(value: i32) -> Self {
        Self { value }
    }

    /// Seeds the global random number generator of `plottery_lib`.
    pub fn apply(&self) {
        plottery_lib::seed(self.value);
    }
}

impl fmt::Display for Seed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Editor value of a [`Seed`] param: the current seed plus recently used seeds, newest first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectParamSeed {
    pub current: i32,
    pub history: Vec<i32>,
}

impl ProjectParamSeed {
    pub const HISTORY_CAP: usize = 10;

    pub fn new(current: i32) -> Self {
        Self {
            current,
            history: Vec::new(),
        }
    }

    /// Sets a new seed and remembers the previous one.
    pub fn set(&mut self, new: i32) {
        if new == self.current {
            return;
        }
        let previous = std::mem::replace(&mut self.current, new);
        self.history.retain(|s| *s != previous && *s != new);
        self.history.insert(0, previous);
        self.history.truncate(Self::HISTORY_CAP);
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}
