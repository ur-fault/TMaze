use cmaze::{
    algorithms::MazeSpec,
    dims::{Dims, Dims3D, Offset},
};
use paste::paste;
use std::sync::Arc;

use hashbrown::HashMap;

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
            config_layer: PartialConfig {
                general: todo!(),
                viewport: todo!(),
                nagivation: todo!(),
                updates: todo!(),
                audio: todo!(),
            },
            base: Config {
                general: todo!(),
                viewport: todo!(),
                nagivation: todo!(),
                updates: todo!(),
                audio: todo!(),
            },
        }
    }
}

trait Mergeable<O> {
    fn merge(&mut self, other: &O);
}

// trace_macros!(true);

macro_rules! config {
    (@step $name:ident
         [$($fields:tt)*]
         [$($rfields:tt)*]
         [$($pfields:tt)*]
         { $field:ident : $type:ty, $($rest:tt)* }
    ) => {
        config!{ @step
            $name
            [ $($fields)* $field Default::default() ]
            [ $($rfields)* pub $field : $type, ]
            [ $($pfields)* pub $field : Option<$type>, ]
            { $($rest)* }
        }
    };

    (@step $name:ident
         [$($fields:tt)*]
         [$($rfields:tt)*]
         [$($pfields:tt)*]
         { #[nest] $field:ident : $type:ty, $($rest:tt)* }
    ) => {
        config!{ @step
            $name
            [ $($fields)* $field Default::default() ]
            [ $($rfields)* pub $field : $type, ]
            [ $($pfields)* pub $field : Option<[<Partial $type>]>, ]
            { $($rest)* }
        }
    };

    (@step $name:ident
         [$($fields:tt)*]
         [$($rfields:tt)*]
         [$($pfields:tt)*]
         { $field:ident : $type:ty = $def:expr, $($rest:tt)* }
    ) => {
        config!{ @step
            $name
            [ $($fields)* $field ($def) ]
            [ $($rfields)* pub $field : $type, ]
            [ $($pfields)* pub $field : Option<$type>, ]
            { $($rest)* }
        }
    };

    (@step $name:ident
         [$($fields:ident $def_vals:expr)*]
         [$($rfields:tt)*]
         [$($pfields:tt)*]
         { }
    ) => {
        pub struct $name {
            $($rfields)*
        }

        impl ::std::default::Default for $name {
            fn default() -> Self {
                Self {
                    $(
                        $fields : $def_vals,
                    )*
                }
            }
        }

        paste! {
            pub struct [<Partial $name>] {
                $($pfields)*
            }

            impl Mergeable<[<Partial $name>]> for $name {
                fn merge(&mut self, other: &[<Partial $name>]) {
                    $(
                        if let Some(value) = &other.$fields {
                            self.$fields.merge(value);
                        }
                    )*
                }
            }
        }
    };

    ($(pub struct $name:ident { $($body:tt)* })*) => {
        $(config!{ @step $name [] [] [] { $($body)* } })*
    };
}

macro_rules! impl_merge_prims {
    ($($t:ty)*) => {
        $(impl Mergeable<$t> for $t where $t: Clone {
            fn merge(&mut self, other: &$t) {
                *self = other.clone();
            }
        })*
    };
}

type Rgb = (u8, u8, u8);

impl_merge_prims! {
    String
    f64
    i64
    bool
    log::Level
    Rgb
    Dims
    Dims3D
    CameraMode
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
        check_interval: bool,
        include_prereleases: bool,
    }

    pub struct Audio {
        enable_audio: bool,
        audio_volume: f64,
        enable_music: bool,
        music_volume: f64,
    }

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

#[derive(Default, Clone, Copy, Debug)]
pub enum CameraMode {
    #[default]
    CloseFollow,
    EdgeFollow {
        x: Offset,
        y: Offset,
    },
}

#[derive(Debug, Clone, Copy, Default)]
pub enum UpdateCheckInterval {
    Never,
    #[default]
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Always,
}

#[derive(Debug, Clone, Default)]
pub struct PresetList {
    presets: Vec<MazePreset>,
}

impl Mergeable<Self> for PresetList {
    fn merge(&mut self, other: &Self) {
        self.presets.extend_from_slice(&other.presets);
    }
}

#[derive(Debug, Clone)]
pub struct MazePreset {
    pub title: String,
    pub description: Option<String>,

    pub default: bool,

    pub maze_spec: MazeSpec,
}

enum Value {
    Object(HashMap<String, Value>),
    List(Vec<Value>),
    String(String),
    Number(f64),
    Int(i64),
    Bool(bool),
}
