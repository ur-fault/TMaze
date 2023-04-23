use std::time::Duration;

use crates_io_api::{Error as CratesError, SyncClient};
use semver::{Comparator, Version, VersionReq};

pub fn get_newer() -> Result<Option<Version>, CratesError> {
    let client = SyncClient::new("tmaze", Duration::from_secs(1)).unwrap();

    let latest_version = client
        .full_crate(env!("CARGO_PKG_NAME"), false)?
        .max_stable_version
        .unwrap();
    let latest_version = Version::parse(latest_version.as_str()).unwrap();

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
