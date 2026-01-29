use cmaze::{
    algorithms::MazeSpec,
    dims::{Dims, Dims3D, Offset},
};

use std::sync::Arc;

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use crate::{config, impl_merge_prims, settings::config_utils::Mergeable};

struct Settings {
    inner: Arc<SettingsInner>,
}

impl Settings {
    fn new() -> Self {
        Settings {
            inner: Arc::new(SettingsInner::new()),
        }
    }
}

struct SettingsInner {
    // ui_layer: PartialConfig,
    config_layer: PartialConfig,
    base: Config,
}

impl SettingsInner {
    fn new() -> Self {
        Self {
            config_layer: PartialConfig::default(),
            base: Config::default(),
        }
    }
}

enum ConfigLoadError {
    IoError(std::io::Error),
    ParseError(String),
}

fn load_config_from_file(path: &str) -> Result<PartialConfig, ConfigLoadError> {
    todo!()

}

type Rgb = (u8, u8, u8);

impl_merge_prims! {
    String
    f64
    i64
    bool

    Rgb
    Dims
    Dims3D

    log::Level
    CameraMode
    UpdateCheckInterval
}

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
}

config! {
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
enum Value {
    Object(HashMap<String, Value>),
    List(Vec<Value>),
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_deserialize() {
        let json_data = r#"
        {
            "name": "Example",
            "enabled": true,
            "threshold": 10.5,
            "count": 42,
            "items": [1, 2, 3],
            "settings": {
                "option1": "value1",
                "option2": false
            }
        }
        "#;

        let parsed: Value = serde_json::from_str(json_data).unwrap();

        if let Value::Object(map) = parsed {
            assert_eq!(map.get("name"), Some(&Value::String("Example".to_string())));
            assert_eq!(map.get("enabled"), Some(&Value::Bool(true)));
            assert_eq!(map.get("threshold"), Some(&Value::Float(10.5)));
            assert_eq!(map.get("count"), Some(&Value::Int(42)));

            if let Some(Value::List(items)) = map.get("items") {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Value::Int(1));
                assert_eq!(items[1], Value::Int(2));
                assert_eq!(items[2], Value::Int(3));
            } else {
                panic!("Expected 'items' to be a list");
            }

            if let Some(Value::Object(settings)) = map.get("settings") {
                assert_eq!(settings.get("option1"), Some(&Value::String("value1".to_string())));
                assert_eq!(settings.get("option2"), Some(&Value::Bool(false)));
            } else {
                panic!("Expected 'settings' to be an object");
            }
        } else {
            panic!("Expected top-level value to be an object");
        }
    }
}
