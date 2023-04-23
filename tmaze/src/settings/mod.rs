pub mod editable;

use crossterm::style::{Color, ContentStyle};
use derivative::Derivative;
use dirs::preference_dir;
use ron::{self, extensions::Extensions};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use self::editable::EditableField;
pub use self::editable::EditableFieldError;
use crate::renderer::Renderer;

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

#[derive(Debug, Derivative, Clone, Serialize, Deserialize)]
#[derivative(Default)]
pub struct Settings {
    #[serde(default)]
    pub color_scheme: Option<ColorScheme>,
    #[serde(default)]
    pub slow: Option<bool>,
    #[serde(default)]
    pub disable_tower_auto_up: Option<bool>,
    #[serde(default)]
    pub camera_mode: Option<CameraMode>,
    #[serde(default)]
    pub default_maze_gen_algo: Option<MazeGenAlgo>,
    #[serde(default)]
    pub dont_ask_for_maze_algo: Option<bool>,
    #[serde(default)]
    pub check_for_updates: Option<bool>,
    #[serde(default)]
    pub mazes: Option<Vec<MazePreset>>,
    #[serde(skip)]
    #[derivative(Default(value = "Settings::default_path()"))]
    pub path: PathBuf,
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
        preference_dir().unwrap().join("tmaze").join("settings.ron")
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

    pub fn set_check_for_updates(mut self, value: bool) -> Self {
        self.check_for_updates = Some(value);
        self
    }

    pub fn get_check_for_updates(&self) -> bool {
        self.check_for_updates.unwrap_or(true)
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
