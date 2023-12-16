use std::{
    fs::File,
    io::{self, BufReader, Read, Write},
    path::Path,
};

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectConfig {
    pub name: String,
    pub created_date: DateTime<Utc>,
    pub last_modified_date: DateTime<Utc>,
}

impl PartialEq for ProjectConfig {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.created_date == other.created_date
    }
}

impl ProjectConfig {
    pub fn new(name: String) -> Self {
        Self {
            name,
            created_date: Utc::now(),
            last_modified_date: Utc::now(),
        }
    }

    pub fn new_from_file(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;

        Ok(serde_json::from_str(&contents)?)
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = io::BufWriter::new(file);

        let contents = serde_json::to_string_pretty(&self)?;
        writer.write_all(contents.as_bytes())?;

        Ok(())
    }

    pub fn update_last_modified_date(&mut self) {
        self.last_modified_date = Utc::now();
    }
}
