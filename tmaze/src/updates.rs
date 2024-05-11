use std::time::Duration;

use chrono::Local;
use crates_io_api::{AsyncClient, Error as CratesError};
use crossterm::event::{Event as TermEvent, KeyCode, KeyEvent};
use semver::{Comparator, Version, VersionReq};

use crate::{
    app::{app::AppData, ActivityHandler, Change, Event},
    data::SaveData,
    helpers::ToDebug,
    settings::Settings,
    ui::Popup,
};

pub async fn get_newer_async() -> Result<Option<Version>, CratesError> {
    let client = AsyncClient::new("tmaze", Duration::from_secs(1)).unwrap();

    // Load the latest version of this crate from crates.io
    let latest_version = client
        .full_crate(env!("CARGO_PKG_NAME"), false)
        .await?
        .max_stable_version
        .unwrap();
    let latest_version = Version::parse(latest_version.as_str()).unwrap();

    // Compare the latest version to the current version
    let current_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    let version_req = VersionReq {
        comparators: vec![Comparator {
            op: semver::Op::Greater,
            major: current_version.major,
            minor: Some(current_version.minor),
            patch: Some(current_version.patch),
            pre: current_version.pre,
        }],
    };

    if version_req.matches(&latest_version) {
        Ok(Some(latest_version))
    } else {
        Ok(None)
    }
}

// TODO: Updates should be checked in the background
// and the user should get a notification if a newer version is available

pub struct UpdateCheckerActivity {
    popup: Popup,
    task: Option<(
        tokio::task::JoinHandle<Result<Option<Version>, CratesError>>,
        tokio::runtime::Runtime,
    )>,
}

impl UpdateCheckerActivity {
    pub fn new(settings: &Settings, save: &SaveData) -> Self {
        let last_check_before = save
            .last_update_check
            .map(|l| Local::now().signed_duration_since(l))
            .map(|d| d.to_std().expect("Failed to convert to std duration"))
            .map(|d| d - Duration::from_nanos(d.subsec_nanos() as u64)) // remove subsec time
            .map(humantime::format_duration);

        let update_interval = format!(
            "Currently checkes {} for updates",
            settings.get_check_interval().to_debug().to_lowercase()
        );

        let popup = Popup::new(
            "Checking for newer version".to_string(),
            vec![
                "Please wait...".to_string(),
                update_interval,
                last_check_before
                    .map(|lc| format!("Last check before: {}", lc))
                    .unwrap_or("Never checked for updates".to_owned()),
                "Press 'q' to cancel or Esc to skip".to_string(),
            ],
        );
        Self { popup, task: None }
    }
}

impl ActivityHandler for UpdateCheckerActivity {
    fn update(
        &mut self,
        events: Vec<crate::app::Event>,
        app_data: &mut AppData,
    ) -> Option<crate::app::Change> {
        for event in events {
            match event {
                Event::Term(TermEvent::Key(KeyEvent {
                    code: KeyCode::Char('q') | KeyCode::Esc,
                    ..
                })) => {
                    return Some(Change::pop_top());
                }
                _ => {}
            }
        }

        if self.task.is_none() {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let handle = rt.spawn(get_newer_async());
            self.task = Some((handle, rt));
            return None;
        }

        let (handle, rt) = self.task.as_mut().unwrap();
        if !handle.is_finished() {
            return None;
        }

        // `block_on` should not block here, since it's already finished
        let result = rt
            .block_on(handle)
            .expect("Failed to join the update check task");

        match result {
            Ok(Some(version)) => {
                app_data
                    .save
                    .update_last_check()
                    .expect("Failed to save the save data");
                log::info!("Newer version found: {}", version);
            }
            Err(err) if app_data.settings.get_display_update_check_errors() => {
                log::error!("Error while checking for updates: {}", err);
            }
            Ok(None) => {
                log::info!("No newer version found");
                app_data
                    .save
                    .update_last_check()
                    .expect("Failed to save the save data");
            }
            Err(_) => {}
        }

        Some(Change::pop_top())
    }

    fn screen(&self) -> &dyn crate::ui::Screen {
        &self.popup
    }
}
