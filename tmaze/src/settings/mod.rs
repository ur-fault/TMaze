pub mod editable;

use crossterm::style::{Color, ContentStyle};
use derivative::Derivative;
use ron::{self, extensions::Extensions};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use self::editable::EditableField;
pub use self::editable::EditableFieldError;
use crate::{constants::base_path, renderer::Renderer};

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
pub struct Settings {
    // general
    #[serde(default)]
    pub color_scheme: Option<ColorScheme>,
    #[serde(default)]
    // motion
    pub slow: Option<bool>,
    #[serde(default)]
    pub disable_tower_auto_up: Option<bool>,
    #[serde(default)]
    pub camera_mode: Option<CameraMode>,

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
    // since it would require this `struct` to become a `map` in RON.
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
                &format!(" {}", self.path.display()),
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

    pub fn set_color_scheme(mut self, value: ColorScheme) -> Self {
        self.color_scheme = Some(value);
        self
    }

    pub fn get_color_scheme(&self) -> ColorScheme {
        self.color_scheme.clone().unwrap_or_default()
    }

    pub fn set_slow(mut self, value: bool) -> Self {
        self.slow = Some(value);
        self
    }

    pub fn get_slow(&self) -> bool {
        self.slow.unwrap_or_default()
    }

    pub fn set_disable_tower_auto_up(mut self, value: bool) -> Self {
        self.disable_tower_auto_up = Some(value);
        self
    }

    pub fn get_disable_tower_auto_up(&self) -> bool {
        self.disable_tower_auto_up.unwrap_or_default()
    }

    pub fn set_camera_mode(mut self, value: CameraMode) -> Self {
        self.camera_mode = Some(value);
        self
    }

    pub fn get_camera_mode(&self) -> CameraMode {
        self.camera_mode.unwrap_or_default()
    }

    pub fn set_default_maze_gen_algo(mut self, value: MazeGenAlgo) -> Self {
        self.default_maze_gen_algo = Some(value);
        self
    }

    pub fn get_default_maze_gen_algo(&self) -> MazeGenAlgo {
        self.default_maze_gen_algo.unwrap_or_default()
    }

    pub fn set_dont_ask_for_maze_algo(mut self, value: bool) -> Self {
        self.dont_ask_for_maze_algo = Some(value);
        self
    }

    pub fn get_dont_ask_for_maze_algo(&self) -> bool {
        self.dont_ask_for_maze_algo.unwrap_or_default()
    }

    pub fn set_check_interval(mut self, value: UpdateCheckInterval) -> Self {
        self.update_check_interval = Some(value);
        self
    }

    pub fn get_check_interval(&self) -> UpdateCheckInterval {
        self.update_check_interval.unwrap_or_default()
    }

    pub fn get_display_update_check_errors(&self) -> bool {
        self.display_update_check_errors.unwrap_or(true)
    }

    pub fn set_display_update_check_errors(mut self, value: bool) -> Self {
        self.display_update_check_errors = Some(value);
        self
    }

    pub fn get_enable_audio(&self) -> bool {
        self.enable_audio.unwrap_or_default()
    }

    pub fn set_enable_audio(mut self, value: bool) -> Self {
        self.enable_audio = Some(value);
        self
    }

    pub fn get_audio_volume(&self) -> f32 {
        self.audio_volume.unwrap_or_default().clamp(0., 1.)
    }

    pub fn set_audio_volume(mut self, value: f32) -> Self {
        self.audio_volume = Some(value.clamp(0., 1.));
        self
    }

    pub fn get_enable_music(&self) -> bool {
        self.enable_music.unwrap_or_default()
    }

    pub fn set_enable_music(mut self, value: bool) -> Self {
        self.enable_music = Some(value);
        self
    }

    pub fn get_music_volume(&self) -> f32 {
        self.music_volume.unwrap_or_default().clamp(0., 1.)
    }

    pub fn set_music_volume(mut self, value: f32) -> Self {
        self.music_volume = Some(value.clamp(0., 1.));
        self
    }

    pub fn set_mazes(mut self, value: Vec<MazePreset>) -> Self {
        self.mazes = Some(value);
        self
    }

    pub fn get_mazes(&self) -> Vec<MazePreset> {
        self.mazes.clone().unwrap_or_default()
    }

    pub fn load(path: PathBuf) -> Self {
        let default_settings_string = include_str!("./default_settings.ron");

        let settings_string = fs::read_to_string(&path);
        let options = ron::Options::default().with_default_extension(Extensions::IMPLICIT_SOME);
        let mut settings: Self = if let Ok(settings_string) = settings_string {
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

        settings
    }

    pub fn reset(&mut self) {
        let default_settings_string = include_str!("./default_settings.ron");
        let options = ron::Options::default().with_default_extension(Extensions::IMPLICIT_SOME);
        *self = options.from_str(default_settings_string).unwrap();
        self.path = Settings::default_path();

        fs::write(&self.path, default_settings_string).unwrap();
    }

    pub fn reset_config(path: PathBuf) {
        let default_settings_string = include_str!("./default_settings.ron");
        fs::write(path, default_settings_string).unwrap();
    }
}
