use std::{
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard},
};

use anyhow::Result;
use plottery_project::{project_params_list_wrapper::ProjectParamsListWrapper, Project};

/// The editor's params file, telling its own writes apart from changes by the CLI.
#[derive(Clone)]
pub struct ParamsFile {
    project: Project,
    /// Bytes this editor last wrote or read.
    last: Arc<Mutex<Vec<u8>>>,
}

impl ParamsFile {
    pub fn new(project: Project) -> Self {
        Self {
            project,
            last: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn path(&self) -> PathBuf {
        self.project.get_editor_params_path()
    }

    fn lock(&self) -> MutexGuard<'_, Vec<u8>> {
        self.last
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Writes unless the bytes equal the last ones written or read.
    pub fn save(&self, params: &ProjectParamsListWrapper) -> Result<()> {
        let bytes = bincode::serialize(params)?;
        let mut last = self.lock();
        if *last != bytes {
            self.project.save_editor_params(params)?;
            *last = bytes;
        }
        Ok(())
    }

    /// The params on disk if someone else changed them since the last save or read.
    pub fn read_external(&self) -> Result<Option<ProjectParamsListWrapper>> {
        let mut last = self.lock();
        let bytes = std::fs::read(self.project.get_editor_params_path())?;
        if *last == bytes {
            return Ok(None);
        }
        let params = bincode::deserialize(&bytes)?;
        *last = bytes;
        Ok(Some(params))
    }
}
