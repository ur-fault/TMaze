use std::ops::Deref;

use cmaze::{
    algorithms::{MazeSpec, MazeSpecType},
    dims::{Dims, Offset},
};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use crate::{
    config, impl_merge_prims,
    settings::{
        config_utils::Mergeable,
        theme::{PartialTerminalColorScheme, TerminalColorScheme},
    },
};

config! {
    pub struct Config {
        #[nest] general: General,
        #[nest] viewport: Viewport,
        #[nest] nagivation: Navigation,
        #[nest] updates: Updates,
        #[nest] audio: Audio,
        presets: PresetList,
    }

    pub struct General {
        theme: String,
        logging_level: log::Level = log::Level::Info,
        debug_logging_level: log::Level = log::Level::Info,
        file_logging_level: log::Level = log::Level::Info,
        #[nest] terminal_scheme: TerminalColorScheme,
    }

    pub struct Viewport {
        slow: bool,
        disable_tower_auto_up: bool,
        camera_mode: CameraMode,
        camera_smoothing: f64 = 0.5,
        player_smoothing: f64 = 0.5,
        viewport_margin: Dims = Dims(4, 3),
    }

    pub struct Navigation {
        enable_mouse: bool = true,
        enable_dpad: bool,
        landscape_dpad_on_left: bool,
        dpad_swap_up_down: bool,
        enable_margin_around_dpad: bool,
        enable_dpad_highlight: bool = true,
    }

    pub struct Updates {
        check_interval: UpdateCheckInterval,
        display_update_check_errors: bool,
    }

    pub struct Audio {
        enable_audio: bool,
        audio_volume: f64,
        enable_music: bool,
        music_volume: f64,
    }
}

impl_merge_prims! {
    i64
    bool
    f64
    String

    Rgb
    Dims

    log::Level
    CameraMode
    UpdateCheckInterval
}

type Rgb = (u8, u8, u8);

#[derive(Default, Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "mode")]
pub enum CameraMode {
    #[default]
    CloseFollow,
    EdgeFollow {
        x: Offset,
        y: Offset,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub enum UpdateCheckInterval {
    Never,
    #[default]
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Always,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PresetList {
    #[serde(flatten)]
    presets: Vec<MazePreset>,
}

impl Deref for PresetList {
    type Target = [MazePreset];

    fn deref(&self) -> &Self::Target {
        &self.presets
    }
}

impl Mergeable<Self> for PresetList {
    fn merge(&mut self, other: &Self) {
        self.presets.extend_from_slice(&other.presets);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MazePreset {
    pub title: String,
    pub description: Option<String>,

    pub default: bool,

    pub maze_spec: MazeSpec,
}

impl MazePreset {
    pub fn short_desc(&self) -> Option<String> {
        let (size, cells): (_, usize) = match &self.maze_spec.inner_spec {
            MazeSpecType::Regions { regions, .. } => (
                self.maze_spec.size()?,
                regions.iter().map(|r| r.mask.enabled_count()).sum(),
            ),
            MazeSpecType::Simple { mask, .. } => (
                self.maze_spec.size()?,
                mask.as_ref()
                    .map(|m| m.enabled_count())
                    .unwrap_or(self.maze_spec.size()?.product() as usize),
            ),
        };

        if size.2 == 1 {
            Some(format!(
                "{}: {}x{} ({} cells)",
                self.title, size.0, size.1, cells
            ))
        } else {
            Some(format!(
                "{}: {}x{}x{} ({} cells)",
                self.title, size.0, size.1, size.2, cells
            ))
        }
    }
}
