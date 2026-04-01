use std::fs;
use std::io::{self, ErrorKind};
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::data::UserProfile;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SaveData {
    pub profiles: Vec<UserProfile>,
}

pub struct Storage;

impl Storage {
    pub fn load() -> SaveData {
        let path = Self::data_path();
        match fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => SaveData::default(),
        }
    }

    pub fn save(data: &SaveData) -> io::Result<()> {
        let path = Self::data_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let serialized = serde_json::to_string_pretty(data)
            .map_err(|err| io::Error::new(ErrorKind::Other, err))?;
        fs::write(path, serialized)?;
        Ok(())
    }

    fn data_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("org", "hackdle", "Hackdle") {
            proj_dirs.data_dir().join("save.json")
        } else {
            PathBuf::from("hackdle_save.json")
        }
    }
}
