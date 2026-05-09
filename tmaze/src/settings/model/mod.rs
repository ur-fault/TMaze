pub mod ver_1;
pub mod ver_2;

// Re-export the latest version of the config.
pub use ver_2::*;

/// The current version of the config format.
///
/// This does *not* specify which version to load by default. The game will load the version `1` to
/// remain compatible with existing configs. It's used to prompt users to migrate the config and for
/// the game to automatically migrate the UI config.
pub const CURRENT_VERSION: i32 = FORMAT_VERSION;

pub trait ToCurrentConfig {
    fn to_current_config(self) -> PartialConfig;

    fn format_version(&self) -> i32;
}

impl ToCurrentConfig for PartialConfig {
    fn to_current_config(self) -> PartialConfig {
        self
    }

    fn format_version(&self) -> i32 {
        CURRENT_VERSION
    }
}
