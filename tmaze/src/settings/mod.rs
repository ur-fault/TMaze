use masof::{Color, ContentStyle};
use ron;
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
pub struct Maze {
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
    pub color_scheme: ColorScheme,
    #[serde(default)]
    pub slow: bool,
    #[serde(default)]
    pub disable_tower_auto_up: bool,
    #[serde(default)]
    pub camera_mode: CameraMode,
    #[serde(default)]
    pub default_maze_gen_algo: MazeGenAlgo,
    #[serde(default)]
    pub dont_ask_for_maze_algo: bool,
    #[serde(default)]
    pub mazes: Vec<Maze>,
}

#[allow(dead_code)]
impl Settings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn color_scheme(mut self, value: ColorScheme) -> Self {
        self.color_scheme = value;
        self
    }

    pub fn slow(mut self, value: bool) -> Self {
        self.slow = value;
        self
    }

    pub fn disable_tower_auto_up(mut self, value: bool) -> Self {
        self.disable_tower_auto_up = value;
        self
    }

    pub fn camera_mode(mut self, value: CameraMode) -> Self {
        self.camera_mode = value;
        self
    }

    pub fn maze_gen_algo(mut self, value: MazeGenAlgo) -> Self {
        self.default_maze_gen_algo = value;
        self
    }

    pub fn load(path: PathBuf) -> Self {
        let default_settings_string = include_str!("./default_settings.ron");

        let settings_string = fs::read_to_string(&path);
        if let Ok(settings_string) = settings_string {
            match ron::de::from_str(&settings_string) {
                Ok(settings) => settings,
                Err(err) => {
                    panic!("Invalid settings file, {}", err);
                }
            }
        } else {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, default_settings_string).unwrap();
            ron::from_str(default_settings_string).unwrap()
        }
    }
}
