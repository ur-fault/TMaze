use std::time::Duration;

use crates_io_api::{AsyncClient, Error as CratesError};
use semver::{Comparator, Version, VersionReq};

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
