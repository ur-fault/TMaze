use cmaze::dims::Dims;
use serde::{Deserialize, Serialize};

use crate::config;
use crate::settings::model::{Presets, ToCurrentConfig};

// Don't depend on `settings::model` directly to avoid broken code on version update
use crate::settings::model::ver_2::{
    CameraMode, PartialTerminalSchemeDef, TerminalSchemeDef, UpdateCheckInterval,
};

pub const FORMAT_VERSION: i32 = 1;

config! {
    pub struct Config {
        // general
        theme: String,
        logging_level: String,
        debug_logging_level: String,
        file_logging_level: String,
        #[nest] terminal_scheme: TerminalSchemeDef,

        // viewport
        slow: bool,
        disable_tower_auto_up: bool,
        camera_mode: CameraMode,
        camera_smoothing: f64,
        player_smoothing: f64,
        viewport_margin: Dims = Dims(0, 0), // unused default

        // navigation
        enable_mouse: bool,
        enable_dpad: bool,
        landscape_dpad_on_left: bool,
        dpad_swap_up_down: bool,
        enable_margin_around_dpad: bool,
        enable_dpad_highlight: bool,

        // update check
        update_check_interval: UpdateCheckInterval,
        display_update_check_errors: bool,

        // audio
        enable_audio: bool,
        audio_volume: f64,
        enable_music: bool,
        music_volume: f64,

        // presets
        presets: Presets,
    }
}

impl ToCurrentConfig for PartialConfig {
    fn to_current_config(self) -> super::PartialConfig {
        use super::*;

        fn parse_log(level: Option<String>) -> Option<log::Level> {
            Some(
                level
                    .and_then(|l| l.parse().ok())
                    .unwrap_or(log::Level::Info),
            )
        }

        PartialConfig {
            general: Some(PartialGeneral {
                logging: Some(PartialLogging {
                    normal: parse_log(self.logging_level),
                    debug: parse_log(self.debug_logging_level),
                    file: parse_log(self.file_logging_level),
                }),
                appearance: Some(PartialAppearance {
                    theme: self.theme,
                    terminal_scheme: self.terminal_scheme,
                }),
            }),
            game: Some(PartialGame {
                slow: self.slow,
                disable_tower_auto_up: self.disable_tower_auto_up,
                view: Some(PartialGameView {
                    camera_mode: self.camera_mode,
                    camera_smoothing: self.camera_smoothing,
                    player_smoothing: self.player_smoothing,
                    viewport_margin: self.viewport_margin,
                }),
                content: Some(PartialContent {
                    presets: self.presets,
                }),
            }),
            controls: Some(PartialControls {
                mouse: Some(PartialMouse {
                    enable: self.enable_mouse,
                    dpad: Some(PartialDpad {
                        enable: self.enable_dpad,
                        landscape_on_left: self.landscape_dpad_on_left,
                        swap_up_down: self.dpad_swap_up_down,
                        enable_margin: self.enable_margin_around_dpad,
                        enable_highlight: self.enable_dpad_highlight,

                        // other fields don't have equivalents
                        ..PartialDpad::default()
                    }),
                }),
            }),
            updates: Some(PartialUpdates {
                check_interval: self.update_check_interval,
                show_errors: self.display_update_check_errors,
            }),
            audio: Some(PartialAudio {
                global: Some(PartialVolume {
                    enable: self.enable_audio,
                    volume: self.audio_volume,
                }),
                music: Some(PartialVolume {
                    enable: self.enable_music,
                    volume: self.music_volume,
                }),
            }),
        }
    }

    fn format_version(&self) -> i32 {
        FORMAT_VERSION
    }
}
