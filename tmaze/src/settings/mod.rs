pub mod attribute;
pub mod meta;
pub mod theme;

pub mod model;

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
use serde::Serialize;

use crate::{
    app::{
        app::EventSink,
        event::{EventReceiver, EventReceiverFn},
        GlobalEvent,
    },
    helpers::{constants::paths, value_if},
    settings::{
        meta::UserContent,
        model::{ver_1, ToCurrentConfig as _, CURRENT_VERSION},
    },
};

use config_utils::{ConvertContext, ConvertError, LenientConvert, Mergeable, Value};
use model::{Config, MazePreset, PartialConfig};

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
        self.inner.compiled.load()
    }

    #[track_caller]
    pub fn update_ui<T>(&self, with: impl FnOnce(&mut PartialConfig) -> T) -> T {
        log::trace!("Updating UI settings from {}", Location::caller());
        let mut new_ui = (**self.inner.ui_layer.load()).clone();
        let x = with(&mut new_ui);
        self.inner.ui_layer.store(Arc::new(new_ui));

        self.inner.rebuild();
        self.notify();
        x
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
            .send(GlobalEvent::SettingsChanged)
            .expect("Event drain should be alive");
    }

    pub fn build_default_config() -> String {
        use upon::*;

        let mut engine = Engine::default();
        const TEMPLATE_NAME: &str = "default_config.json5";
        engine
            .add_template(
                TEMPLATE_NAME,
                include_str!("./files/default_settings.json5"),
            )
            .expect("Default config template should be valid");

        engine.add_function(
            "json",
            |value: &Value| -> std::result::Result<String, String> {
                serde_json::to_string(value)
                    .map_err(|e| format!("failed to serialize value to JSON: {}", e))
            },
        );

        engine.add_function(
            "json_pretty",
            |value: &Value, offset: &str| -> std::result::Result<String, String> {
                let (first_line, remainder) = match offset.split_once(":") {
                    Some((l, r)) => {
                        let remainder = r.parse().map_err(|_| "invalid `json_pretty` argument")?;
                        let first_line = if l.is_empty() {
                            remainder
                        } else {
                            l.parse().map_err(|_| "invalid `json_pretty` argument")?
                        };
                        (first_line, remainder)
                    }
                    None => {
                        let val = offset
                            .parse()
                            .map_err(|_| "invalid `json_pretty` argument")?;
                        (val, val)
                    }
                };

                let first_indent = " ".repeat(first_line);
                let base_indent = " ".repeat(remainder);
                let indent = " ".repeat(4);

                let mut out_buf = Vec::new();
                value
                    .serialize(&mut serde_json::Serializer::with_formatter(
                        &mut out_buf,
                        serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes()),
                    ))
                    .map_err(|e| format!("failed to serialize value to JSON: {}", e))?;

                let out_str = String::from_utf8(out_buf)
                    .map_err(|e| format!("failed to convert JSON output to string: {}", e))?;

                let lines = out_str.lines();
                let indented = lines
                    .clone()
                    .take(1)
                    .map(|line| format!("{}{}", first_indent, line))
                    .chain(lines.skip(1).map(|line| format!("{}{}", base_indent, line)))
                    .collect::<Vec<_>>()
                    .join("\n");

                Ok(indented)
            },
        );

        let mut context =
            serde_json::to_value(Config::default()).expect("Default config should be serializable");
        match context {
            serde_json::Value::Object(ref mut map) => {
                map.insert(
                    "__presets".into(),
                    serde_json::to_value(&*DEFAULT_PRESETS)
                        .expect("Default presets should be serializable"),
                );
            }
            _ => panic!("Context should be a JSON object"),
        }

        engine
            .template(TEMPLATE_NAME)
            .render(&context)
            .to_string()
            .expect("Default config should render correctly")
    }
}

impl EventReceiver for &Settings {
    fn register(self) -> EventReceiverFn {
        let settings = self.clone();
        Box::new(move |event, _| {
            if matches!(event, GlobalEvent::SettingsChanged) {
                log::trace!("Writing UI settings to file from event");
                settings.write_ui();
            }
        })
    }
}

type UserConfig<C = PartialConfig> = UserContent<C>;

struct SettingsInner {
    config_layer: UserConfig,
    ui_layer: ArcSwap<PartialConfig>,
    compiled: ArcSwap<Config>,
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
            compiled: ArcSwap::default(),
        };

        settings.rebuild();

        let errors = value_if(!errors.is_empty(), || Some(errors));
        let warnings = value_if(!warnings.is_empty(), || Some(warnings));

        (settings, errors, warnings)
    }

    fn rebuild(&self) {
        let mut config = Config::default();
        config.merge(&self.config_layer);
        config.merge(&self.ui_layer.load());

        self.compiled.store(Arc::new(config));
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
    Empty,
}

fn load_ui_config_from_file(path: &Path) -> PartialConfig {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn load_config_from_source(
    source: &ConfigSource<'_>,
) -> (UserConfig, Vec<ConvertError>, Vec<ConvertError>) {
    let mut context = ConvertContext::new();
    let config = match load_values_from_source(source) {
        Ok(value) => match extract_format_version(value.clone()) {
            1 => {
                context.warn("Config format version 1 is deprecated, please update your config to the latest format");
                UserConfig::<ver_1::PartialConfig>::convert(value, &mut context)
                    .map(|c| c.map_content(|partial| partial.to_current_config()))
            }
            2 => UserConfig::convert(value, &mut context),
            v => {
                context.warn(format!(
                    "Config format version {v} is not supported, interpreting as latest supported version ({CURRENT_VERSION})"
                ));
                UserConfig::convert(value, &mut context)
            }
        },
        Err((e, val)) => {
            context.err(format!("Failed to load config: {}", e));
            UserConfig::convert(val, &mut context)
        }
    };

    let (errors, warnings) = context.extract();
    (config.unwrap_or_default(), errors, warnings)
}

fn extract_format_version(config_value: Value) -> i32 {
    let with_meta =
        UserContent::<()>::convert(config_value, &mut ConvertContext::new()).unwrap_or_default();

    with_meta.meta.format_version
}

fn load_values_from_source(source: &ConfigSource) -> Result<Value, (ConfigLoadError, Value)> {
    use meta::Meta;
    use serde_json::{from_value, to_value};

    let empty = || {
        from_value::<Value>(to_value(UserConfig::new(Meta::with_version(2), ())).unwrap()).unwrap()
    };

    macro_rules! pack_error {
        ($err:expr) => {
            match $err {
                Ok(val) => Ok(val),
                Err(e) => Err((e.into(), empty())),
            }
        };
    }

    match source {
        ConfigSource::Path(path) => {
            // json5 doesn't support reading from reader
            let content = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(err) => {
                    if err.kind() == std::io::ErrorKind::NotFound {
                        return Ok(empty());
                    } else {
                        return pack_error!(Err(err));
                    }
                }
            };
            let config_value = pack_error!(json5::from_str::<Value>(&content))?;
            let values_with_extensions = load_extension_blocks(config_value)?;
            Ok(values_with_extensions)
        }
        ConfigSource::User => load_values_from_source(&ConfigSource::Path(&paths::config())),
        ConfigSource::Empty => Ok(empty()),
        ConfigSource::String(s) => {
            let config_value = pack_error!(json5::from_str(s))?;
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
    use serde_json::json;

    use crate::settings::{
        config_utils::LenientConvert as _,
        model::{MazePreset, PresetGroupItem, Presets},
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

    #[test]
    fn test_config_presets_group() {
        let x = serde_json::from_value(json! ([{
            "group": "A",
            "items": [],
        }]))
        .unwrap();

        let mut context = super::ConvertContext::new();
        let presets = Presets::convert(x, &mut context);
        let (errors, warnings) = context.extract();

        assert_eq!(
            errors,
            vec![],
            "Presets conversion should not produce errors"
        );
        assert_eq!(
            warnings,
            vec![],
            "Presets conversion should not produce warnings"
        );
        assert!(presets.is_some(), "Presets should be converted correctly");
    }

    #[test]
    fn test_config_presets_preset() {
        let x = serde_json::from_value(json! ({
            "title": "Preset 1",
            "type": "simple",
        }))
        .unwrap();

        let mut context = super::ConvertContext::new();
        let presets = MazePreset::convert(x, &mut context);
        let (errors, warnings) = context.extract();

        assert_eq!(
            errors,
            vec![],
            "Presets conversion should not produce errors"
        );
        assert_eq!(
            warnings,
            vec![],
            "Presets conversion should not produce warnings"
        );
        assert!(presets.is_some(), "Presets should be converted correctly");
    }

    #[test]
    fn test_no_null_in_config() {
        let config_str = super::Settings::build_default_config();
        let config_value = json5::from_str::<super::Value>(&config_str)
            .expect("Default config should be valid JSON5");

        fn assert_no_null(value: &super::Value, path: &str) {
            match value {
                super::Value::Nil => panic!("Config value at path '{}' should not be null", path),
                super::Value::Object(map) => {
                    for (key, val) in map {
                        assert_no_null(val, &format!("{}.{}", path, key));
                    }
                }
                super::Value::List(arr) => {
                    for (i, val) in arr.iter().enumerate() {
                        assert_no_null(val, &format!("{}[{}]", path, i));
                    }
                }
                _ => {}
            }
        }

        assert_no_null(&config_value, "config");
    }
}
