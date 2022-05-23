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
pub struct ColorScheme {
    pub normal: Color,
    pub player: Color,
    pub goal: Color,
}

impl ColorScheme {
    pub fn normals(&self) -> ContentStyle {
        ContentStyle {
            foreground_color: Some(self.normal),
            ..Default::default()
        }
    }

    pub fn players(&self) -> ContentStyle {
        ContentStyle {
            foreground_color: Some(self.player),
            ..Default::default()
        }
    }

    pub fn goals(&self) -> ContentStyle {
        ContentStyle {
            foreground_color: Some(self.goal),
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
        }
    }
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
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub color_scheme: ColorScheme,
    pub slow: bool,
    pub disable_tower_auto_up: bool,
    pub camera_mode: CameraMode,
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

    pub fn load(path: PathBuf) -> Self {
        let settings_string = fs::read_to_string(&path);
        if let Ok(settings_string) = settings_string {
            if let Ok(settings) = ron::de::from_str(&settings_string) {
                settings
            } else {
                panic!("Invalid settings file");
            }
        } else {
            let default_settings_string = include_str!("./default_settings.ron");
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, default_settings_string).unwrap();
            ron::from_str(default_settings_string).unwrap()
        }
    }
}
