use chrono::{DateTime, Datelike, Local, NaiveDate};
use ron::{de::from_reader, ser::to_writer};
use std::{
    fs::File,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    constants::base_path,
    settings::{Settings, UpdateCheckInterval},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub last_update_check: Option<DateTime<Local>>,
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
        Self::load().unwrap_or_else(|_| Self {
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
        self.last_update_check = Some(Local::now());
        self.write()
    }

    pub fn is_update_checked(&self, settings: &Settings) -> bool {
        use UpdateCheckInterval::*;

        match settings.get_check_interval() {
            Never => true,
            Daily => self.check_date(|d| d),
            Weekly => self.check_date(|d| d.iso_week()),
            Monthly => self.check_date(|d| d.with_day(1).unwrap()),
            Yearly => self.check_date(|d| d.with_day(1).unwrap().with_month(1)),
            Always => false,
        }
    }

    fn write(&self) -> Result<(), ron::Error> {
        self.write_to(&self.path)
    }

    fn write_to(&self, path: &Path) -> Result<(), ron::Error> {
        to_writer(File::create(path)?, self)
    }

    fn check_date<E: Eq>(&self, transform: impl Fn(NaiveDate) -> E) -> bool {
        let today = Local::now().date_naive();
        self.last_update_check
            .map(|lc| lc.date_naive())
            .map(|lc| transform(lc) == transform(today))
            .unwrap_or(false)
    }
}
