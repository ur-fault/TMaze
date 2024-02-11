pub mod editable;

use crossterm::style::{Color, ContentStyle};
use derivative::Derivative;
use ron::{self, extensions::Extensions};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use self::editable::EditableField;
pub use self::editable::EditableFieldError;
use crate::{constants::base_path, renderer::Renderer};

const DEFAULT_SETTINGS: &str = include_str!("./default_settings.ron");

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum CameraMode {
    #[default]
    CloseFollow,
    EdgeFollow(i32, i32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MazePreset {
    pub title: String,
    pub width: u16,
    pub height: u16,
    #[serde(default = "default_depth")]
    pub depth: u16,
    #[serde(default)]
    pub tower: bool,
    #[serde(default)]
    pub default: bool,
}

fn default_depth() -> u16 {
    1
}

impl EditableField for MazePreset {
    fn print(&self) -> String {
        self.title.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub normal: Color,
    pub player: Color,
    pub goal: Color,
    pub text: Color,
}

#[allow(dead_code)]
impl ColorScheme {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn normal(mut self, value: Color) -> Self {
        self.normal = value;
        self
    }

    pub fn player(mut self, value: Color) -> Self {
        self.player = value;
        self
    }

    pub fn goal(mut self, value: Color) -> Self {
        self.goal = value;
        self
    }

    pub fn text(mut self, value: Color) -> Self {
        self.text = value;
        self
    }

    pub fn normals(&self) -> ContentStyle {
        ContentStyle {
            foreground_color: Some(self.normal),
            background_color: None,
            ..Default::default()
        }
    }

    pub fn players(&self) -> ContentStyle {
        ContentStyle {
            foreground_color: Some(self.player),
            background_color: None,
            ..Default::default()
        }
    }

    pub fn goals(&self) -> ContentStyle {
        ContentStyle {
            foreground_color: Some(self.goal),
            background_color: None,
            ..Default::default()
        }
    }

    pub fn texts(&self) -> ContentStyle {
        ContentStyle {
            foreground_color: Some(self.text),
            background_color: None,
            ..Default::default()
        }
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        ColorScheme {
            normal: Color::White,
            player: Color::White,
            goal: Color::White,
            text: Color::White,
        }
    }
}

impl EditableField for ColorScheme {
    fn print(&self) -> String {
        todo!();
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum MazeGenAlgo {
    #[default]
    RandomKruskals,
    DepthFirstSearch,
}

impl EditableField for MazeGenAlgo {
    fn print(&self) -> String {
        match self {
            MazeGenAlgo::RandomKruskals => "Random Kruskals".to_string(),
            MazeGenAlgo::DepthFirstSearch => "Depth First Search".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum UpdateCheckInterval {
    Never,
    #[default]
    Daily,
    Weekly,
    Monthly,
    Yearly,
    Always,
}

#[derive(Debug, Derivative, Clone, Serialize, Deserialize)]
#[derivative(Default)]
#[serde(rename = "Settings")]
pub struct SettingsInner {
    // general
    #[serde(default)]
    pub color_scheme: Option<ColorScheme>,
    #[serde(default)]

    // motion
    pub slow: Option<bool>,
    #[serde(default)]
    // TODO: rename to disable_tower_auto_advance
    pub disable_tower_auto_up: Option<bool>,
    #[serde(default)]
    pub camera_mode: Option<CameraMode>,
    #[serde(default)]
    pub blink_duration: Option<f64>,

    // game config
    #[serde(default)]
    pub default_maze_gen_algo: Option<MazeGenAlgo>,
    #[serde(default)]
    pub dont_ask_for_maze_algo: Option<bool>,
    #[serde(default)]
    // update check
    pub update_check_interval: Option<UpdateCheckInterval>,
    #[serde(default)]
    pub display_update_check_errors: Option<bool>,

    // audio
    #[serde(default)]
    pub enable_audio: Option<bool>,
    #[serde(default)]
    pub audio_volume: Option<f32>,
    #[serde(default)]
    pub enable_music: Option<bool>,
    #[serde(default)]
    pub music_volume: Option<f32>,

    // mazes
    #[serde(default)]
    pub mazes: Option<Vec<MazePreset>>,

    // other
    #[serde(skip)]
    #[derivative(Default(value = "Settings::default_path()"))]
    pub path: PathBuf,
    // TODO: it's not possible in RON to have a HashMap with flattened keys,
    // so we will support it in different way formats
    // once we support them - this would mean dropping RON support
    // https://github.com/ron-rs/ron/issues/115
    // pub unknown_fields: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct Settings(Arc<RwLock<SettingsInner>>);

impl Default for Settings {
    fn default() -> Self {
        let settings = SettingsInner::default();
        Self(Arc::new(RwLock::new(settings)))
    }
}

impl EditableField for Settings {
    fn print(&self) -> String {
        String::from("Settings")
    }

    fn edit(
        &mut self,
        renderer: &mut Renderer,
        color_scheme: ColorScheme,
    ) -> Result<bool, EditableFieldError> {
        crate::ui::popup(
            renderer,
            color_scheme.normals(),
            color_scheme.texts(),
            "Edit settings",
            &[
                "Path to the current settings",
                &format!(" {}", self.read().path.display()),
            ],
        )
        .map(|_| false)
        .map_err(|e| e.into())
    }
}

#[allow(dead_code)]
impl Settings {
    pub fn default_path() -> PathBuf {
        base_path().join("settings.ron")
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(&self) -> std::sync::RwLockReadGuard<SettingsInner> {
        self.0.read().unwrap()
    }

    pub fn write(&mut self) -> std::sync::RwLockWriteGuard<SettingsInner> {
        self.0.write().unwrap()
    }

    pub fn get_color_scheme(&self) -> ColorScheme {
        self.read().color_scheme.clone().unwrap_or_default()
    }

    pub fn set_color_scheme(mut self, value: ColorScheme) -> Self {
        self.write().color_scheme = Some(value);
        self
    }

    pub fn set_slow(mut self, value: bool) -> Self {
        self.write().slow = Some(value);
        self
    }

    pub fn get_slow(&self) -> bool {
        self.read().slow.unwrap_or_default()
    }

    pub fn set_disable_tower_auto_up(mut self, value: bool) -> Self {
        self.write().disable_tower_auto_up = Some(value);
        self
    }

    pub fn get_disable_tower_auto_up(&self) -> bool {
        self.read().disable_tower_auto_up.unwrap_or_default()
    }

    pub fn set_camera_mode(mut self, value: CameraMode) -> Self {
        self.write().camera_mode = Some(value);
        self
    }

    pub fn get_camera_mode(&self) -> CameraMode {
        self.read().camera_mode.unwrap_or_default()
    }

    pub fn get_blink_duration(&self) -> f64 {
        self.read().blink_duration.unwrap_or(0.5)
    }

    pub fn set_blink_duration(mut self, value: f64) -> Self {
        self.write().blink_duration = Some(value);
        self
    }

    pub fn set_default_maze_gen_algo(mut self, value: MazeGenAlgo) -> Self {
        self.write().default_maze_gen_algo = Some(value);
        self
    }

    pub fn get_default_maze_gen_algo(&self) -> MazeGenAlgo {
        self.read().default_maze_gen_algo.unwrap_or_default()
    }

    pub fn set_dont_ask_for_maze_algo(mut self, value: bool) -> Self {
        self.write().dont_ask_for_maze_algo = Some(value);
        self
    }

    pub fn get_dont_ask_for_maze_algo(&self) -> bool {
        self.read().dont_ask_for_maze_algo.unwrap_or_default()
    }

    pub fn set_check_interval(mut self, value: UpdateCheckInterval) -> Self {
        self.write().update_check_interval = Some(value);
        self
    }

    pub fn get_check_interval(&self) -> UpdateCheckInterval {
        self.read().update_check_interval.unwrap_or_default()
    }

    pub fn get_display_update_check_errors(&self) -> bool {
        self.read().display_update_check_errors.unwrap_or(true)
    }

    pub fn set_display_update_check_errors(mut self, value: bool) -> Self {
        self.write().display_update_check_errors = Some(value);
        self
    }

    pub fn get_enable_audio(&self) -> bool {
        self.read().enable_audio.unwrap_or_default()
    }

    pub fn set_enable_audio(mut self, value: bool) -> Self {
        self.write().enable_audio = Some(value);
        self
    }

    pub fn get_audio_volume(&self) -> f32 {
        self.read().audio_volume.unwrap_or_default().clamp(0., 1.)
    }

    pub fn set_audio_volume(mut self, value: f32) -> Self {
        self.write().audio_volume = Some(value.clamp(0., 1.));
        self
    }

    pub fn get_enable_music(&self) -> bool {
        self.read().enable_music.unwrap_or_default()
    }

    pub fn set_enable_music(mut self, value: bool) -> Self {
        self.write().enable_music = Some(value);
        self
    }

    pub fn get_music_volume(&self) -> f32 {
        self.read().music_volume.unwrap_or_default().clamp(0., 1.)
    }

    pub fn set_music_volume(mut self, value: f32) -> Self {
        self.write().music_volume = Some(value.clamp(0., 1.));
        self
    }

    pub fn set_mazes(mut self, value: Vec<MazePreset>) -> Self {
        self.write().mazes = Some(value);
        self
    }

    pub fn get_mazes(&self) -> Vec<MazePreset> {
        self.read().mazes.clone().unwrap_or_default()
    }
}

impl Settings {
    pub fn load(path: PathBuf) -> Self {
        let default_settings_string = DEFAULT_SETTINGS;

        let settings_string = fs::read_to_string(&path);
        let options = ron::Options::default().with_default_extension(Extensions::IMPLICIT_SOME);
        let mut settings: SettingsInner = if let Ok(settings_string) = settings_string {
            match options.from_str(&settings_string) {
                Ok(settings) => settings,
                Err(err) => {
                    panic!("Error reading settings file ({:?}), {}", path, err);
                }
            }
        } else {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, default_settings_string).unwrap();
            options.from_str(default_settings_string).unwrap()
        };

        settings.path = path;

        Self(Arc::new(RwLock::new(settings)))
    }

    pub fn reset(&mut self) {
        let default_settings_string = DEFAULT_SETTINGS;
        let options = ron::Options::default().with_default_extension(Extensions::IMPLICIT_SOME);
        *self.write() = options.from_str(default_settings_string).unwrap();

        let path = Settings::default_path();
        fs::write(&path, default_settings_string).unwrap();

        self.write().path = path;
    }

    pub fn reset_config(path: PathBuf) {
        let default_settings_string = DEFAULT_SETTINGS;
        fs::write(path, default_settings_string).unwrap();
    }
}
