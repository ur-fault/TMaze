use cmaze::{game::GeneratorFn, gameboard::algorithms::MazeAlgorithm};
use crossterm::style::{Color, ContentStyle};
use derivative::Derivative;
use ron::{self, extensions::Extensions};
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use crate::{
    app::{self, app::AppData, Activity, ActivityHandler, Change},
    constants::base_path,
    helpers::constants::colors,
    menu_actions,
    renderer::MouseGuard,
    ui::{split_menu_actions, style_with_attribute, Menu, MenuAction, MenuConfig, Popup, Screen},
};

#[cfg(feature = "sound")]
use crate::sound::create_audio_settings;

const DEFAULT_SETTINGS: &str = include_str!("./default_settings.ron");

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Offset {
    Abs(i32),
    Rel(f32),
}

impl Offset {
    pub fn to_abs(self, size: i32) -> i32 {
        match self {
            Offset::Rel(ratio) => (size as f32 * ratio).round() as i32,
            Offset::Abs(chars) => chars,
        }
    }
}

impl Default for Offset {
    fn default() -> Self {
        Offset::Rel(0.25)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum CameraMode {
    #[default]
    CloseFollow,
    EdgeFollow(Offset, Offset),
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
    #[serde(default = "colors::fun::white")]
    pub normal: Color,
    #[serde(default = "colors::fun::white")]
    pub player: Color,
    #[serde(default = "colors::fun::white")]
    pub goal: Color,
    #[serde(default = "colors::fun::white")]
    pub text: Color,
    #[serde(default = "colors::fun::white")]
    pub highlight: Color,
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

    pub fn highlight(mut self, value: Color) -> Self {
        self.highlight = value;
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

    pub fn highlights(&self) -> ContentStyle {
        ContentStyle {
            foreground_color: Some(self.highlight),
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
            highlight: Color::White,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum MazeGenAlgo {
    #[default]
    RandomKruskals,
    DepthFirstSearch,
}

impl MazeGenAlgo {
    pub fn to_fn(&self) -> GeneratorFn {
        match self {
            MazeGenAlgo::RandomKruskals => cmaze::gameboard::algorithms::RndKruskals::generate,
            MazeGenAlgo::DepthFirstSearch => {
                cmaze::gameboard::algorithms::DepthFirstSearch::generate
            }
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
    pub disable_tower_auto_up: Option<bool>,
    #[serde(default)]
    pub camera_mode: Option<CameraMode>,
    #[serde(default)]
    pub camera_smoothing: Option<f32>,
    #[serde[default]]
    pub player_smoothing: Option<f32>,

    // navigation
    #[serde(default)]
    pub enable_mouse: Option<bool>,

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

#[allow(dead_code)]
impl Settings {
    pub fn default_path() -> PathBuf {
        base_path().join("settings.ron")
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn path(&self) -> PathBuf {
        self.0.read().unwrap().path.clone()
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

    pub fn set_color_scheme(&mut self, value: ColorScheme) -> &mut Self {
        self.write().color_scheme = Some(value);
        self
    }

    pub fn get_slow(&self) -> bool {
        self.read().slow.unwrap_or_default()
    }

    pub fn set_slow(&mut self, value: bool) -> &mut Self {
        self.write().slow = Some(value);
        self
    }

    pub fn get_disable_tower_auto_up(&self) -> bool {
        self.read().disable_tower_auto_up.unwrap_or_default()
    }

    pub fn set_disable_tower_auto_up(&mut self, value: bool) -> &mut Self {
        self.write().disable_tower_auto_up = Some(value);
        self
    }

    pub fn get_camera_mode(&self) -> CameraMode {
        self.read().camera_mode.unwrap_or_default()
    }

    pub fn set_camera_mode(&mut self, value: CameraMode) -> &mut Self {
        self.write().camera_mode = Some(value);
        self
    }

    pub fn get_camera_smoothing(&self) -> f32 {
        self.read().camera_smoothing.unwrap_or(0.5).clamp(0.5, 1.0)
    }

    pub fn set_camera_smoothing(&mut self, value: f32) -> &mut Self {
        self.write().camera_smoothing = Some(value.clamp(0.5, 1.0));
        self
    }

    pub fn get_player_smoothing(&self) -> f32 {
        self.read().player_smoothing.unwrap_or(0.8).clamp(0.5, 1.0)
    }

    pub fn set_player_smoothing(&mut self, value: f32) -> &mut Self {
        self.write().player_smoothing = Some(value.clamp(0.5, 1.0));
        self
    }

    pub fn get_enable_mouse(&self) -> bool {
        self.read().enable_mouse.unwrap_or(true)
    }

    pub fn set_enable_mouse(&mut self, value: bool) -> &mut Self {
        self.write().enable_mouse = Some(value);
        self
    }

    pub fn set_default_maze_gen_algo(&mut self, value: MazeGenAlgo) -> &mut Self {
        self.write().default_maze_gen_algo = Some(value);
        self
    }

    pub fn get_default_maze_gen_algo(&self) -> MazeGenAlgo {
        self.read().default_maze_gen_algo.unwrap_or_default()
    }

    pub fn set_dont_ask_for_maze_algo(&mut self, value: bool) -> &mut Self {
        self.write().dont_ask_for_maze_algo = Some(value);
        self
    }

    pub fn get_dont_ask_for_maze_algo(&self) -> bool {
        self.read().dont_ask_for_maze_algo.unwrap_or_default()
    }

    pub fn set_check_interval(&mut self, value: UpdateCheckInterval) -> &mut Self {
        self.write().update_check_interval = Some(value);
        self
    }

    pub fn get_check_interval(&self) -> UpdateCheckInterval {
        self.read().update_check_interval.unwrap_or_default()
    }

    pub fn get_display_update_check_errors(&self) -> bool {
        self.read().display_update_check_errors.unwrap_or(true)
    }

    pub fn set_display_update_check_errors(&mut self, value: bool) -> &mut Self {
        self.write().display_update_check_errors = Some(value);
        self
    }

    pub fn get_enable_audio(&self) -> bool {
        self.read().enable_audio.unwrap_or_default()
    }

    pub fn set_enable_audio(&mut self, value: bool) -> &mut Self {
        self.write().enable_audio = Some(value);
        self
    }

    pub fn get_audio_volume(&self) -> f32 {
        self.read().audio_volume.unwrap_or_default().clamp(0., 1.)
    }

    pub fn set_audio_volume(&mut self, value: f32) -> &mut Self {
        self.write().audio_volume = Some(value.clamp(0., 1.));
        self
    }

    pub fn get_enable_music(&self) -> bool {
        self.read().enable_music.unwrap_or_default()
    }

    pub fn set_enable_music(&mut self, value: bool) -> &mut Self {
        self.write().enable_music = Some(value);
        self
    }

    pub fn get_music_volume(&self) -> f32 {
        self.read().music_volume.unwrap_or_default().clamp(0., 1.)
    }

    pub fn set_music_volume(&mut self, value: f32) -> &mut Self {
        self.write().music_volume = Some(value.clamp(0., 1.));
        self
    }

    pub fn set_mazes(&mut self, value: Vec<MazePreset>) -> &mut Self {
        self.write().mazes = Some(value);
        self
    }

    pub fn get_mazes(&self) -> Vec<MazePreset> {
        self.read().mazes.clone().unwrap_or_default()
    }
}

impl Settings {
    pub fn load(path: PathBuf) -> io::Result<Self> {
        let default_settings_string = DEFAULT_SETTINGS;

        let settings_string = fs::read_to_string(&path);
        let options = ron::Options::default().with_default_extension(Extensions::IMPLICIT_SOME);
        let mut settings: SettingsInner = if let Ok(settings_string) = settings_string {
            options
                .from_str(&settings_string)
                .expect("Could not parse settings file")
        } else {
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(&path, default_settings_string)?;
            options.from_str(default_settings_string).unwrap()
        };

        settings.path = path;

        Ok(Self(Arc::new(RwLock::new(settings))))
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

struct OtherSettingsPopup(Popup, MouseGuard);

impl OtherSettingsPopup {
    fn new(settings: &Settings) -> Self {
        let popup = Popup::new(
            "Other settings".to_string(),
            vec![
                "Path to the current settings:".to_string(),
                format!(" {}", settings.path().to_string_lossy().to_string()),
                "".to_string(),
                "Other settings are not implemented in UI yet.".to_string(),
                "Please edit the settings file directly.".to_string(),
            ],
        )
        .styles_from_settings(settings);

        Self(popup, MouseGuard::new().unwrap())
    }
}

impl ActivityHandler for OtherSettingsPopup {
    fn update(&mut self, events: Vec<app::Event>, data: &mut AppData) -> Option<Change> {
        self.0.update(events, data)
    }

    fn screen(&self) -> &dyn Screen {
        &self.0
    }
}

pub struct SettingsActivity {
    actions: Vec<MenuAction<Change>>,
    menu: Menu,
}

impl SettingsActivity {
    fn other_settings_popup(settings: &Settings) -> Activity {
        Activity::new_base_boxed("settings".to_string(), OtherSettingsPopup::new(settings))
    }
}

impl SettingsActivity {
    pub fn new(settings: &Settings) -> Self {
        let options = menu_actions!(
            "Audio" on "sound" -> data => Change::push(create_audio_settings(data)),
            "Other settings" -> data => Change::push(SettingsActivity::other_settings_popup(&data.settings)),
            "Back" -> _ => Change::pop_top(),
        );

        let (options, actions) = split_menu_actions(options);

        let menu_config = MenuConfig::new("Settings", options)
            .styles_from_settings(settings)
            .subtitle("Changes are not saved")
            .subtitle_style(style_with_attribute(
                settings.get_color_scheme().texts(),
                crossterm::style::Attribute::Dim,
            ));

        Self {
            actions,
            menu: Menu::new(menu_config),
        }
    }

    pub fn new_activity(settings: &Settings) -> Activity {
        Activity::new_base_boxed("settings".to_string(), Self::new(settings))
    }
}

impl ActivityHandler for SettingsActivity {
    fn update(&mut self, events: Vec<app::Event>, data: &mut AppData) -> Option<Change> {
        match self.menu.update(events, data)? {
            Change::Pop {
                res: Some(sub_activity),
                ..
            } => {
                let index = *sub_activity
                    .downcast::<usize>()
                    .expect("menu should return index");
                Some((self.actions[index])(data))
            }
            res => Some(res),
        }
    }

    fn screen(&self) -> &dyn Screen {
        &self.menu
    }
}
