use std::{thread, time::Duration};

use crates_io_api::{AsyncClient, Error as CratesError};
use semver::{Comparator, Version, VersionReq};

use crate::app::{app::AppData, jobs::Job};

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

    Ok(version_req.matches(&latest_version).then(|| latest_version))
}

pub fn check(app_data: &mut AppData) {
    if app_data.save.is_update_checked(&app_data.settings) {
        return;
    }

    let display_update_errors = app_data.settings.get_display_update_check_errors();

    let qer = app_data.queuer();

    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();

        let handle = rt.spawn(get_newer_async());
        let result = rt
            .block_on(handle)
            .expect("Failed to join the update check task");

        match result {
            Ok(Some(version)) => {
                log::warn!("Newer version found: {}", version);
                qer.queue(Job::new(|data| {
                    data.save
                        .update_last_check()
                        .expect("Failed to save the save data");
                }));
            }
            Ok(None) => {
                log::info!("No newer version found");
                qer.queue(Job::new(|data| {
                    data.save
                        .update_last_check()
                        .expect("Failed to save the save data");
                }));
            }
            Err(err) if display_update_errors => {
                log::error!("Error while checking for updates: {}", err);
            }
            Err(_) => {}
        };
    });
}
