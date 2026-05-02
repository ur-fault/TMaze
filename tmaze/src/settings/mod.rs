pub mod attribute;
pub mod model;
pub mod theme;

mod config_utils;

use std::{
    fmt::Display,
    ops::Deref,
    panic::Location,
    path::Path,
    sync::{Arc, LazyLock},
};

use arc_swap::ArcSwap;
use cmaze::algorithms::{MazeSpec, MazeSpecType, MazeType};
use hashbrown::HashMap;
use serde::Serialize;
use tera::Tera;

use crate::{
    app::{
        app::{AppData, EventSink},
        event::EventReceiver,
        Event,
    },
    helpers::constants::paths,
    settings::model::MazePreset,
};

use config_utils::{ConvertContext, ConvertError, LenientConvert, Mergeable, Value};
use model::{Config, PartialConfig};

#[derive(Clone)]
pub struct Settings {
    inner: Arc<SettingsInner>,
    event_sink: EventSink,
}

impl Settings {
    /// Loads settings from configuration files.
    ///
    /// Returns the loaded settings and any errors and warnings that occurred during
    /// loading.
    pub fn load(
        event_sink: EventSink,
        source: &ConfigSource,
    ) -> (Self, Option<Vec<String>>, Option<Vec<String>>) {
        let (inner, load_errors, load_warnings) = SettingsInner::load(source);
        (
            Self {
                inner: Arc::new(inner),
                event_sink,
            },
            load_errors,
            load_warnings,
        )
    }

    pub fn read(&self) -> impl Deref<Target = Arc<Config>> + use<'_> {
        self.inner.config.load()
    }

    #[track_caller]
    pub fn update_ui(&self, with: impl FnOnce(&mut PartialConfig)) {
        log::trace!("Updating UI settings from {}", Location::caller());
        let mut new_ui = (**self.inner.ui_layer.load()).clone();
        with(&mut new_ui);
        self.inner.ui_layer.store(Arc::new(new_ui));

        self.inner.rebuild();
        self.notify();
    }

    fn write_ui(&self) {
        log::trace!("Writing UI settings to file");
        std::fs::write(
            paths::managed::ui_settings(),
            serde_json::to_string_pretty(&**self.inner.ui_layer.load())
                .expect("UI settings should be serializable"),
        )
        .expect("Failed to write UI settings to file");
    }

    fn notify(&self) {
        self.event_sink
            .send(Event::SettingsChanged)
            .expect("Event drain should be alive");
    }

    pub fn build_default_config() -> String {
        let mut tera = Tera::default();
        const TEMPLATE_NAME: &str = "default_config.json5";
        tera.add_raw_template(
            TEMPLATE_NAME,
            include_str!("./files/default_settings.json5"),
        )
        .expect("Default config template should be valid");
        tera.register_function(
            "repeat",
            |args: &std::collections::HashMap<String, tera::Value>| {
                let string = args.get("string").and_then(|v| v.as_str()).ok_or_else(|| {
                    tera::Error::msg("repeat filter requires a string argument of type `string`")
                })?;
                let times: usize = args.get("times").and_then(|v| v.as_u64()).ok_or_else(|| {
                    tera::Error::msg("repeat filter requires a times argument of type `number`")
                })? as usize;
                let result = string.repeat(times);
                Ok(tera::Value::String(result))
            },
        );
        tera.register_filter(
            "json_encode_2",
            |value: &tera::Value, args: &std::collections::HashMap<_, tera::Value>| {
                let base_indent = args
                    .get("base_indent")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize;
                let block_indent = args
                    .get("block_indent")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(4) as usize;
                let indent = args.get("indent").and_then(|v| v.as_str()).unwrap_or(" ");

                let base_indent = " ".repeat(base_indent);
                let indent = indent.repeat(block_indent);

                let mut out_buf = Vec::new();
                value.serialize(&mut serde_json::Serializer::with_formatter(
                    &mut out_buf,
                    serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes()),
                ))?;

                let out_str = String::from_utf8(out_buf).map_err(|e| {
                    tera::Error::msg(format!("Failed to convert JSON output to string: {}", e))
                })?;

                let indented = out_str
                    .lines()
                    .map(|line| {
                        if line.is_empty() {
                            line.into()
                        } else {
                            format!("{}{}", base_indent, line)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                Ok(tera::Value::String(indented))
            },
        );

        let mut context = tera::Context::from_serialize(Config::default())
            .expect("Default config should be serializable");
        context.insert("__presets", &*DEFAULT_PRESETS);

        // panic!("{}", tera.render(TEMPLATE_NAME, &context).unwrap())
        tera.render(TEMPLATE_NAME, &context)
            .expect("Default config should render correctly")
    }
}

impl EventReceiver for &Settings {
    fn register(self) -> Box<dyn FnMut(&Event, &mut AppData)> {
        let settings = self.clone();
        Box::new(move |event, _| {
            if let Event::SettingsChanged = event {
                log::trace!("Writing UI settings to file from event");
                settings.write_ui();
            }
        })
    }
}

struct SettingsInner {
    config_layer: PartialConfig,
    ui_layer: ArcSwap<PartialConfig>,
    config: ArcSwap<Config>,
}

impl SettingsInner {
    fn load(source: &ConfigSource<'_>) -> (Self, Option<Vec<String>>, Option<Vec<String>>) {
        let mut errors = vec![];
        let mut warnings = vec![];

        let (config_layer, load_errors, load_warnings) = load_config_from_source(source);
        errors.extend(load_errors.iter().map(ConvertError::to_string));
        warnings.extend(load_warnings.iter().map(ConvertError::to_string));

        let ui_layer = load_ui_config_from_file(&paths::managed::ui_settings());

        let settings = Self {
            config_layer,
            ui_layer: ArcSwap::from_pointee(ui_layer),
            config: ArcSwap::default(),
        };

        settings.rebuild();

        let errors = if errors.is_empty() {
            None
        } else {
            Some(errors)
        };

        let warnings = if warnings.is_empty() {
            None
        } else {
            Some(warnings)
        };

        (settings, errors, warnings)
    }

    fn rebuild(&self) {
        let mut config = Config::default();
        config.merge(&self.config_layer);
        config.merge(&self.ui_layer.load());

        self.config.store(Arc::new(config));
    }
}

#[derive(Debug, thiserror::Error)]
enum ConfigLoadError {
    IoError(#[from] std::io::Error),
    JsonError(#[from] json5::Error),
    SettingsFormatError(String),
}

impl Display for ConfigLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigLoadError::IoError(e) => write!(f, "I/O error: {}", e),
            ConfigLoadError::JsonError(e) => write!(f, "Parse error: {}", e),
            ConfigLoadError::SettingsFormatError(e) => write!(f, "Settings format error: {}", e),
        }
    }
}

pub enum ConfigSource<'a> {
    User,
    Path(&'a Path),
    String(String),
    Default,
}

fn load_ui_config_from_file(path: &Path) -> PartialConfig {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn load_config_from_source(
    source: &ConfigSource<'_>,
) -> (PartialConfig, Vec<ConvertError>, Vec<ConvertError>) {
    let mut context = ConvertContext::new();
    let config = match load_values_from_source(source) {
        Ok(value) => PartialConfig::convert(value, &mut context),
        Err((e, val)) => {
            context.err(format!("Failed to load config: {}", e));
            PartialConfig::convert(val, &mut context)
        }
    };

    let (errors, warnings) = context.extract();
    (config.unwrap_or_default(), errors, warnings)
}

fn load_values_from_source(source: &ConfigSource) -> Result<Value, (ConfigLoadError, Value)> {
    macro_rules! pack_error {
        ($err:expr) => {
            match $err {
                Ok(val) => Ok(val),
                Err(e) => Err((e.into(), Value::Object(HashMap::new()))),
            }
        };
    }

    match source {
        ConfigSource::Path(path) => {
            // json5 doesn't support reading from reader
            let content = pack_error!(std::fs::read_to_string(path))?;
            let config_value = pack_error!(json5::from_str::<Value>(&content))?;
            let values_with_extensions = load_extension_blocks(config_value)?;
            Ok(values_with_extensions)
        }
        ConfigSource::User => load_values_from_source(&ConfigSource::Path(&paths::config())),
        ConfigSource::Default => Ok(Value::Object(HashMap::new())),
        ConfigSource::String(s) => {
            let config_value = pack_error!(json5::from_str::<Value>(s))?;
            let values_with_extensions = load_extension_blocks(config_value)?;
            Ok(values_with_extensions)
        }
    }
}

mod config_file_constants {
    pub const IMPORT_KEY: &str = "#from";
}

fn load_extension_blocks(config: Value) -> Result<Value, (ConfigLoadError, Value)> {
    use config_file_constants::IMPORT_KEY;

    /// Merges `ext` into `base`. In case of conflict, `ext` takes precedence.
    /// Note that in this case, `base` is file behing `#from`, and `ext` is the current file.
    ///
    /// For objects, merging is done recursively.
    ///
    /// TODO: Allow rules customization in the future, for example to support list contatenation
    /// instead of replacement.
    fn merge(base: &mut Value, ext: Value) {
        match (base, ext) {
            (Value::Object(base_map), Value::Object(ext_map)) => {
                for (key, ext_value) in ext_map {
                    if key == IMPORT_KEY {
                        continue; // skip #from key during merge
                    }

                    if let Some(base_value) = base_map.get_mut(&key) {
                        merge(base_value, ext_value);
                    } else {
                        base_map.insert(key, ext_value);
                    }
                }
            }
            (base_val, ext) => {
                *base_val = ext;
            }
        }
    }

    let value = match config {
        Value::Object(map) => {
            if map.contains_key(IMPORT_KEY) {
                if let Value::String(file_path) = &map[IMPORT_KEY] {
                    let mut base_config =
                        load_values_from_source(&ConfigSource::Path(Path::new(file_path)))?;
                    merge(&mut base_config, Value::Object(map));
                    base_config
                } else {
                    return Err((
                        ConfigLoadError::SettingsFormatError(format!(
                            "{IMPORT_KEY} value must be a string",
                        )),
                        Value::Object(map),
                    ));
                }
            } else {
                Value::Object(map)
            }
        }

        val => val,
    };

    Ok(value)
}

static DEFAULT_PRESETS: LazyLock<Vec<MazePreset>> = LazyLock::new(|| {
    fn simple_maze(size: (i32, i32, i32), tower: bool) -> MazePreset {
        MazePreset {
            title: match (size.2, tower) {
                (1, false) => format!("{}x{}", size.0, size.1),
                (1, true) => format!("{}x{} Tower", size.0, size.1),
                (z, false) => format!("{}x{}x{}", size.0, size.1, z),
                (z, true) => format!("{}x{}x{} Tower", size.0, size.1, z),
            },
            description: None,
            default: false,
            maze_spec: MazeSpec {
                inner_spec: MazeSpecType::Simple {
                    size: Some(size.into()),
                    start: None,
                    end: None,
                    mask: None,
                    splitter: None,
                    generator: None,
                },
                seed: None,
                maze_type: if tower { Some(MazeType::Tower) } else { None },
            },
        }
    }

    vec![
        MazePreset {
            default: true,
            ..simple_maze((10, 5, 1), false)
        },
        simple_maze((20, 10, 1), false),
        simple_maze((60, 30, 1), false),
        simple_maze((200, 100, 1), false),
        simple_maze((6, 3, 3), false),
        simple_maze((10, 5, 5), false),
        simple_maze((12, 6, 5), true),
        simple_maze((40, 20, 10), true),
    ]
});

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::settings::{
        model::{PresetGroup, PresetGroupItem, Presets},
        theme::TerminalColorScheme,
    };

    #[test]
    fn test_config_build_default() {
        let config_str = super::Settings::build_default_config();
        let mut config_value = json5::from_str::<super::Value>(&config_str)
            .expect("Default config should be valid JSON5");

        use super::Value::*;
        match &mut config_value {
            Object(map) => match map.get_mut("general") {
                Some(Object(map)) => match map.get_mut("appearance") {
                    Some(Object(map)) => {
                        map.insert(
                            "terminal_scheme".into(),
                            json5::from_str(
                                &json5::to_string(&TerminalColorScheme::default()).unwrap(),
                            )
                            .unwrap(),
                        );
                    }
                    _ => panic!("Appearance should be an object"),
                },
                _ => panic!("General should be an object"),
            },
            _ => panic!("Config should be an object"),
        }

        assert_eq!(
            config_value,
            json5::from_str(
                json5::to_string(&{
                    let mut config = super::Config::default();
                    config.game.content.presets = Presets(
                        super::DEFAULT_PRESETS
                            .iter()
                            .cloned()
                            .map(PresetGroupItem::Preset)
                            .collect(),
                    );
                    config
                })
                .unwrap()
                .as_str()
            )
            .unwrap(),
            "Default config should be a JSON object"
        );
    }

    #[test]
    fn test_config_default_format() {
        let config = super::Settings::build_default_config();

        for (i, line) in config.lines().enumerate() {
            assert!(
                !line.contains('\t'),
                "Default config should not contain tabs for indentation: {i}: {line}"
            );
            assert!(
                line.find(|c: char| !c.is_whitespace()).unwrap_or(0) % 4 == 0,
                "Default config should be indented with multiples of 4 spaces\n{config}"
            );
            assert!(
                !line.ends_with(' '),
                "Default config should not have trailing spaces: {i}: {line}, config:\n{config}"
            );
            assert!(
                line.len() <= 120,
                "Default config lines should not exceed 120 characters: {i}: {line}"
            );
        }
    }
}
