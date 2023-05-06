use std::path::PathBuf;

use dirs::preference_dir;

pub fn base_path() -> PathBuf {
    preference_dir().unwrap().join("tmaze")
}