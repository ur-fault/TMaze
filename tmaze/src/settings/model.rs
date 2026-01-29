use cmaze::{algorithms::MazeSpec, dims::{Dims, Offset}};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use crate::{config, impl_merge_prims, settings::config_utils::Mergeable};

config! {
    pub struct Config {
        #[nest] general: General,
        #[nest] viewport: Viewport,
        #[nest] nagivation: Navigation,
        #[nest] updates: Updates,
        #[nest] audio: Audio,
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

config! {
    pub struct Presets {
        presets: PresetList,
    }

    pub struct TerminalColorScheme {
        primary_fg: Rgb,
        primary_bg: Rgb,
        black: Rgb,     // grey
        dark_grey: Rgb, // dark grey
        red: Rgb,
        dark_red: Rgb,
        green: Rgb,
        dark_green: Rgb,
        yellow: Rgb,
        dark_yellow: Rgb,
        blue: Rgb,
        dark_blue: Rgb,
        magenta: Rgb,
        dark_magenta: Rgb,
        cyan: Rgb,
        dark_cyan: Rgb,
        white: Rgb,
        grey: Rgb,
    }
}
impl_merge_prims! {
    String
    f64
    i64
    bool

    Rgb
    Dims

    log::Level
    CameraMode
    UpdateCheckInterval
}

type Rgb = (u8, u8, u8);

#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(tag = "mode")]
pub enum CameraMode {
    #[default]
    CloseFollow,
    EdgeFollow {
        x: Offset,
        y: Offset,
    },
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
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
    presets: Vec<MazePreset>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
// Note: order of variants matters for correct deserialization
pub enum Value {
    Object(HashMap<String, Value>),
    List(Vec<Value>),
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}
