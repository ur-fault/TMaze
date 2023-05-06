use ron::{de::from_reader, ser::to_writer};
use std::{
    fs::File,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};

use crate::{
    constants::base_path,
    settings::{Settings, UpdateCheckInterval},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub last_update_check: Option<SystemTime>,
    #[serde(skip_serializing, skip_deserializing)]
    path: PathBuf,
}

impl SaveData {
    pub fn default_path() -> PathBuf {
        base_path().join("data.ron")
    }

    pub fn load() -> Result<Self, ron::Error> {
        match Self::load_from(&Self::default_path()) {
            Ok(data) => Ok(data),
            Err(ron::Error::Io(_)) => Ok(SaveData {
                last_update_check: None,
                path: Self::default_path(),
            }),
            Err(err) => Err(err),
        }
    }

    pub fn load_or() -> Self {
        Self::load().unwrap_or_else(|err| Self {
            last_update_check: None,
            path: Self::default_path(),
        })
    }

    fn load_from(path: &Path) -> Result<Self, ron::Error> {
        Ok(Self {
            path: path.to_owned(),
            ..from_reader(File::open(path)?)?
        })
    }

    pub fn update_last_check(&mut self) -> Result<(), ron::Error> {
        self.last_update_check = Some(SystemTime::now());
        self.write()
    }

    pub fn is_update_checked(&self, settings: &Settings) -> bool {
        use UpdateCheckInterval::*;

        let skip_check = |interval: Duration| {
            if let Some(last_check) = self.last_update_check {
                SystemTime::now()
                    .duration_since(last_check)
                    .map(|d| d < interval)
                    .unwrap_or(false)
            } else {
                false
            }
        };

        match settings.get_check_interval() {
            Never => true,
            Daily => skip_check(Duration::from_secs(24 * 60 * 60)),
            Weekly => skip_check(Duration::from_secs(7 * 24 * 60 * 60)),
            Monthly => skip_check(Duration::from_secs(30 * 24 * 60 * 60)),
            Yearly => skip_check(Duration::from_secs(365 * 24 * 60 * 60)),
            Always => false,
        }
    }

    fn write(&self) -> Result<(), ron::Error> {
        self.write_to(&self.path)
    }

    fn write_to(&self, path: &Path) -> Result<(), ron::Error> {
        to_writer(File::create(path)?, self)
    }
}
