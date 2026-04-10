use std::ops::Deref;

use cmaze::{
    algorithms::{MazeSpec, MazeSpecType},
    dims::{Dims, Offset},
};

use log::Level;
use serde::{Deserialize, Serialize};

use crate::{
    config, impl_lenient_deserialize, impl_lenient_prims, impl_merge_prims,
    settings::{
        config_utils::{ConvertContext, LenientConvert, Mergeable, Value},
        theme::{PartialTerminalColorScheme, TerminalColorScheme},
    },
};

config! {
    pub struct Config {
        #[nest] general: General,
        #[nest] game: Game,
        #[nest] controls: Controls,
        #[nest] updates: Updates,
        #[nest] audio: Audio,
    }

    pub struct General {
        #[nest] logging: Logging,
        #[nest] appearance: Appearance,
    }

    pub struct Game {
        slow: bool,
        disable_tower_auto_up: bool,
        camera_mode: CameraMode,
        camera_smoothing: f64 = 0.5,
        player_smoothing: f64 = 0.5,
        viewport_margin: Dims = Dims(4, 3),
        #[nest] content: Content,
    }

    pub struct Content {
        presets: PresetList,
    }

    pub struct Controls {
        #[nest] mouse: Mouse,
    }

    pub struct Mouse {
        enable: bool = true,
        #[nest] dpad: Dpad,
    }

    pub struct Dpad {
        enable: bool,
        landscape_on_left: bool,
        swap_up_down: bool,
        enable_margin: bool,
        enable_highlight: bool = true,
    }

    pub struct Appearance {
        theme: String,
        #[nest] terminal_scheme: TerminalColorScheme,
    }

    pub struct Logging {
        normal: Level = Level::Info,
        debug: Level = Level::Debug,
        file: Level = Level::Info,
    }

    pub struct Updates {
        check_interval: UpdateCheckInterval,
        show_errors: bool,
    }

    pub struct Audio {
        #[nest] global: Volume,
        #[nest] music: Volume,
    }

    pub struct Volume {
        enable: bool,
        volume: f64,
    }
}

impl_merge_prims! {
    i64
    bool
    f64
    String

    Rgb
    Dims

    Level
    CameraMode
    UpdateCheckInterval
    Volume
}

impl_lenient_prims! {
    i64 => Int,
    bool => Bool,
    f64 => Float Int,
    String => String,
}

impl_lenient_deserialize! {
    Level
    CameraMode
    UpdateCheckInterval
    MazePreset
    Dims
    Rgb
}

type Rgb = (u8, u8, u8);

#[derive(Default, Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "mode")]
// TODO: allow fieldless variants to be specified as strings for convenience
pub enum CameraMode {
    #[default]
    CloseFollow,
    EdgeFollow {
        x: Offset,
        y: Offset,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
#[serde(transparent)]
pub struct PresetList(Vec<MazePreset>);

impl Deref for PresetList {
    type Target = [MazePreset];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Mergeable<Self> for PresetList {
    fn merge(&mut self, other: &Self) {
        self.0.extend_from_slice(&other.0);
    }
}

impl LenientConvert for PresetList {
    fn convert(value: Value, context: &mut ConvertContext) -> Option<Self> {
        let Value::List(list) = value else {
            context.err("expected a list of maze presets".to_string());
            return None;
        };

        let mut presets = vec![];
        for (i, item) in list.into_iter().enumerate() {
            context.push_index(i);
            match MazePreset::convert(item, context) {
                Some(preset) => presets.push(preset),
                None => { /* error already recorded */ }
            }
            context.pop();
        }

        Some(PresetList(presets))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MazePreset {
    pub title: String,
    pub description: Option<String>,

    #[serde(default)]
    pub default: bool,

    #[serde(flatten)]
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
