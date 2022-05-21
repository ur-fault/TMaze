// use ronfig::*;
// use serde::{Deserialize, Serialize};
use masof::ContentStyle;

#[derive(Debug, Clone, Copy)]
pub enum CameraMode {
    CloseFollow,
    EdgeFollow(i32, i32),
}

impl Default for CameraMode {
    fn default() -> Self {
        CameraMode::CloseFollow
    }
}

#[derive(Debug, Default, Clone)]
pub struct ColorScheme {
    pub normal: ContentStyle,
    pub player: ContentStyle,
    pub goal: ContentStyle,
}

impl ColorScheme {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn normal(mut self, value: ContentStyle) -> Self {
        self.normal = value;
        self
    }

    pub fn player(mut self, value: ContentStyle) -> Self {
        self.player = value;
        self
    }

    pub fn goal(mut self, value: ContentStyle) -> Self {
        self.goal = value;
        self
    }
}

#[derive(Debug, Default, Clone)]
pub struct Settings {
    pub color_scheme: ColorScheme,
    pub slow: bool,
    pub disable_tower_auto_up: bool,
    pub camera_mode: CameraMode,
}

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
}
