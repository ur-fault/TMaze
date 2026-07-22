use std::ops::Deref;

use cmaze::{
    algorithms::{MazeSpec, MazeSpecType},
    dims::{Dims, Offset},
};

use log::Level;
use serde::{Deserialize, Serialize};

use crate::{
    config, impl_lenient_deserialize, impl_lenient_prims, impl_merge_prims,
    settings::{
        config_utils::{ConvertContext, LenientConvert, Mergeable, Value},
        theme::{PartialTerminalColorScheme, TerminalColorScheme},
    },
};

pub const FORMAT_VERSION: i32 = 2;

config! {
    pub struct Config {
        #[nest] general: General,
        #[nest] game: Game,
        #[nest] controls: Controls,
        #[nest] updates: Updates,
        #[nest] audio: Audio,
    }

    pub struct General {
        #[nest] logging: Logging,
        #[nest] appearance: Appearance,
    }

    pub struct Game {
        slow: bool,
        disable_tower_auto_up: bool,
        #[nest] view: GameView,
        #[nest] content: Content,
    }

    pub struct GameView {
        camera_mode: CameraMode,
        camera_smoothing: f64 = 0.5,
        player_smoothing: f64 = 0.5,
        viewport_margin: Dims = Dims(4, 3),
    }

    pub struct Content {
        presets: Presets,
    }

    pub struct Controls {
        #[nest] mouse: Mouse,
    }

    pub struct Mouse {
        enable: bool = true,
        #[nest] dpad: Dpad,
    }

    pub struct Dpad {
        enable: bool,
        landscape_on_left: bool,
        swap_up_down: bool,
        enable_margin: bool,
        enable_highlight: bool = false,
        space: f64 = 2. / 5.,
        min_size: Dims = Dims(10, 5),
        max_size: Dims = Dims(50, 25),
    }

    pub struct Appearance {
        theme: String,
        #[nest] terminal_scheme: TerminalSchemeDef,
    }

    pub struct Logging {
        normal: Level = Level::Info,
        debug: Level = Level::Debug,
        file: Level = Level::Info,
    }

    pub struct Updates {
        check_interval: UpdateCheckInterval,
        show_errors: bool,
    }

    pub struct Audio {
        #[nest] global: Volume,
        #[nest] music: Volume,
    }

    pub struct Volume {
        enable: bool,
        volume: f64,
    }
}

impl_merge_prims! {
    i64
    bool
    f64
    String

    Rgb
    Dims

    Level
    CameraMode
    UpdateCheckInterval
    Volume
}

impl_lenient_prims! {
    i64 => Int,
    bool => Bool,
    f64 => Float Int,
    String => String,
}

impl_lenient_deserialize! {
    Level
    CameraMode
    UpdateCheckInterval
    MazePreset
    Dims
    Rgb
}

type Rgb = (u8, u8, u8);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerminalSchemeDef {
    Named(String),
    Custom(TerminalColorScheme),
}

impl Default for TerminalSchemeDef {
    fn default() -> Self {
        TerminalSchemeDef::Custom(TerminalColorScheme::default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PartialTerminalSchemeDef {
    Named(String),
    Custom(PartialTerminalColorScheme),
}

impl Default for PartialTerminalSchemeDef {
    fn default() -> Self {
        PartialTerminalSchemeDef::Custom(PartialTerminalColorScheme::default())
    }
}

impl LenientConvert for PartialTerminalSchemeDef {
    fn convert(value: Value, context: &mut ConvertContext) -> Option<Self> {
        let (named_opt, named_branch) =
            context.branch("named".into(), |ctx| String::convert(value.clone(), ctx));

        let (custom_opt, custom_branch) = context.branch("custom".into(), |ctx| {
            PartialTerminalColorScheme::convert(value, ctx)
        });

        match (named_opt, custom_opt) {
            (Some(named), _) => Some(PartialTerminalSchemeDef::Named(named)),
            (_, Some(custom)) => Some(PartialTerminalSchemeDef::Custom(custom)),
            _ => {
                named_branch.apply(context);
                custom_branch.apply(context);
                context.err("expected a terminal scheme definition with either a 'named' or 'custom' scheme");
                None
            }
        }
    }
}

impl Mergeable<PartialTerminalSchemeDef> for TerminalSchemeDef {
    fn merge(&mut self, other: &PartialTerminalSchemeDef) {
        match (self, other) {
            (TerminalSchemeDef::Named(self_name), PartialTerminalSchemeDef::Named(named)) => {
                *self_name = named.clone();
            }
            (
                TerminalSchemeDef::Custom(custom),
                PartialTerminalSchemeDef::Custom(partial_custom),
            ) => {
                custom.merge(partial_custom);
            }
            (this @ TerminalSchemeDef::Named(_), other @ PartialTerminalSchemeDef::Custom(_)) => {
                *this = TerminalSchemeDef::Custom(Default::default());
                this.merge(other);
            }
            (this @ TerminalSchemeDef::Custom(_), PartialTerminalSchemeDef::Named(name)) => {
                *this = TerminalSchemeDef::Named(name.clone());
            }
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "mode")]
// TODO: allow fieldless variants to be specified as strings for convenience
pub enum CameraMode {
    #[default]
    CloseFollow,
    EdgeFollow {
        x: Offset,
        y: Offset,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
#[serde(transparent)]
pub struct Presets(pub Vec<PresetGroupItem>);

impl Presets {
    pub fn default_indeces(&self) -> Option<Vec<usize>> {
        let mut idxs = self.0.iter().enumerate().find_map(|(i, item)| match item {
            PresetGroupItem::Preset(p) if p.default => Some(vec![i]),
            PresetGroupItem::Group(g) => {
                let mut idxs = g.default_preset_index()?;
                idxs.push(i);
                Some(idxs)
            }
            _ => None,
        })?;

        idxs.reverse();
        Some(idxs)
    }

    // Iterate through the group hierarchy according to the given indices, returning the preset
    // group at the end if it exists.
    //
    // If `indices` is empty, returns `self`.
    pub fn get_group(&self, indices: &[usize]) -> Option<&PresetGroup> {
        match self.0.get(indices[0])? {
            PresetGroupItem::Group(g) => g.get_group(&indices[1..]),
            PresetGroupItem::Preset(_) => None,
        }
    }
}

impl Mergeable<Self> for Presets {
    fn merge(&mut self, other: &Self) {
        self.0.extend_from_slice(&other.0);
    }
}

impl LenientConvert for Presets {
    fn convert(value: Value, context: &mut ConvertContext) -> Option<Self> {
        let value = Value::Object(
            [
                ("items".to_string(), value),
                ("group".to_string(), Value::String("".into())),
            ]
            .into_iter()
            .collect(),
        );

        Some(Presets(PresetGroup::convert(value, context)?.items))
    }
}

impl Deref for Presets {
    type Target = Vec<PresetGroupItem>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PresetGroup {
    pub group: String,
    pub items: Vec<PresetGroupItem>,
}

impl PresetGroup {
    fn default_preset_index(&self) -> Option<Vec<usize>> {
        self.items.iter().enumerate().find_map(|(i, g)| match g {
            PresetGroupItem::Preset(p) if p.default => Some(vec![i]),
            PresetGroupItem::Group(g) => {
                let mut sub_indices = g.default_preset_index()?;
                sub_indices.push(i);
                Some(sub_indices)
            }
            _ => None,
        })
    }

    pub(self) fn get_group(&self, indices: &[usize]) -> Option<&PresetGroup> {
        let mut group = self;
        for &index in indices {
            match group.items.get(index)? {
                PresetGroupItem::Group(g) => group = g,
                PresetGroupItem::Preset(_) => return None,
            }
        }
        Some(group)
    }
}

impl LenientConvert for PresetGroup {
    fn convert(value: Value, context: &mut ConvertContext) -> Option<Self> {
        let Value::Object(mut obj) = value else {
            context.err("expected an object for maze preset group");
            return None;
        };

        let Some(Value::String(group)) = obj.remove("group") else {
            context.err("expected a name (`group`: string) for maze preset group");
            return None;
        };

        let Some(Value::List(list)) = obj.remove("items") else {
            context.err("expected a list of items (`items`: list)");
            return None;
        };

        for (key, _) in obj.into_iter() {
            context.warn(format!("unexpected field '{}' in maze preset group", key));
        }

        let mut items = vec![];
        for (i, item) in list.into_iter().enumerate() {
            context.push_index(i);
            let (preset_opt, preset_branch) = context.branch("preset".into(), |ctx| {
                MazePreset::convert(item.clone(), ctx)
            });

            let (group_opt, group_branch) =
                context.branch("group".into(), |ctx| PresetGroup::convert(item, ctx));

            match (preset_opt, group_opt) {
                (Some(preset), _) => items.push(PresetGroupItem::Preset(preset)),
                (_, Some(group)) => items.push(PresetGroupItem::Group(group)),
                _ => {
                    preset_branch.apply(context);
                    group_branch.apply(context);
                    context.err("expected a maze preset or group");
                }
            }

            context.pop();
        }

        Some(PresetGroup { group, items })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PresetGroupItem {
    Preset(MazePreset),
    Group(PresetGroup),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MazePreset {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(default)]
    pub default: bool,

    #[serde(flatten)]
    pub maze_spec: MazeSpec,
}

impl MazePreset {
    pub fn short_desc(&self) -> Option<String> {
        let (size, cells): (_, usize) = match &self.maze_spec.inner_spec {
            MazeSpecType::Regions { regions, .. } => (
                self.maze_spec.size()?,
                regions.iter().map(|r| r.mask.enabled_count()).sum(),
            ),
            MazeSpecType::Simple { mask, .. } => (
                self.maze_spec.size()?,
                mask.as_ref()
                    .map(|m| m.enabled_count())
                    .unwrap_or(self.maze_spec.size()?.product() as usize),
            ),
        };

        if size.2 == 1 {
            Some(format!(
                "{}: {}x{} ({} cells)",
                self.title, size.0, size.1, cells
            ))
        } else {
            Some(format!(
                "{}: {}x{}x{} ({} cells)",
                self.title, size.0, size.1, size.2, cells
            ))
        }
    }
}
