use masof::{Color, ContentStyle};
use ron::{self, extensions::Extensions};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CameraMode {
    CloseFollow,
    EdgeFollow(i32, i32),
}

impl Default for CameraMode {
    fn default() -> Self {
        CameraMode::CloseFollow
    }
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MazeGenAlgo {
    RandomKruskals,
    DepthFirstSearch,
}

impl Default for MazeGenAlgo {
    fn default() -> Self {
        MazeGenAlgo::RandomKruskals
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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
    pub mazes: Option<Vec<MazePreset>>,
}

#[allow(dead_code)]
impl Settings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn populate(mut self) -> Self {
        self.color_scheme = Some(self.color_scheme.unwrap_or_default());
        self.slow = Some(self.slow.unwrap_or_default());
        self.disable_tower_auto_up = Some(self.disable_tower_auto_up.unwrap_or_default());
        self.camera_mode = Some(self.camera_mode.unwrap_or_default());
        self.default_maze_gen_algo = Some(self.default_maze_gen_algo.unwrap_or_default());
        self.dont_ask_for_maze_algo = Some(self.dont_ask_for_maze_algo.unwrap_or_default());
        self.mazes = Some(self.mazes.unwrap_or_default());

        self
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
        if let Ok(settings_string) = settings_string {
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
        }
    }
}
