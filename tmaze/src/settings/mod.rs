pub mod attribute;
pub mod model;
pub mod theme;

// mod old_settings;

mod config_utils;

use std::{
    fmt::Display,
    ops::Deref,
    path::Path,
    sync::{Arc, Mutex},
};

use hashbrown::HashMap;

use crate::{
    helpers::{constants::paths, TupleMap},
    settings::config_utils::{ConvertContext, ConvertError, LenientConvert, Mergeable, Value},
};

use model::{Config, PartialConfig};

#[derive(Clone)]
pub struct Settings {
    inner: Arc<SettingsInner>,
}

impl Settings {
    /// Loads settings from configuration files.
    ///
    /// Returns the loaded settings and a boolean indicating whether any warnings occurred during
    /// loading.
    ///
    /// TODO: Report the actual errors/warnings to the user.
    pub fn load() -> (Self, Option<Vec<String>>) {
        SettingsInner::load().map_first(|inner| Self {
            inner: Arc::new(inner),
        })
    }

    pub fn read(&self) -> impl Deref<Target = Config> + use<'_> {
        self.inner.config.lock().unwrap()
    }

    pub fn update_ui(&self, with: impl FnOnce(&mut PartialConfig)) {
        let mut ui_layer = self.inner.ui_layer.lock().unwrap();
        with(&mut ui_layer);
        self.inner.rebuild();
    }
}

struct SettingsInner {
    config_layer: PartialConfig,
    ui_layer: Mutex<PartialConfig>,
    config: Mutex<Config>,
}

impl SettingsInner {
    fn load() -> (Self, Option<Vec<String>>) {
        let mut errors = vec![];

        let (config_layer, load_errors) = load_config_from_file(&paths::config());
        errors.extend(load_errors.iter().map(ConvertError::to_string));

        let ui_layer = load_ui_config_from_file(&paths::managed::ui_settings());

        let settings = Self {
            config_layer,
            ui_layer: Mutex::new(ui_layer),
            config: Mutex::new(Config::default()),
        };

        settings.rebuild();

        let errors = if errors.is_empty() {
            None
        } else {
            Some(errors)
        };

        (settings, errors)
    }

    fn rebuild(&self) {
        let mut config = Config::default();
        config.merge(&self.config_layer);
        config.merge(&self.ui_layer.lock().unwrap());

        *self.config.lock().unwrap() = config;
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

fn load_ui_config_from_file(path: &Path) -> PartialConfig {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn load_config_from_file(path: &Path) -> (PartialConfig, Vec<ConvertError>) {
    let mut context = ConvertContext::new();
    let config = match load_values_from_file(path) {
        Ok(value) => PartialConfig::convert(value, &mut context),
        Err((e, val)) => {
            context.err(format!("Failed to load config: {}", e));
            PartialConfig::convert(val, &mut context)
        }
    };

    (config.unwrap_or_default(), context.errors())
}

fn load_values_from_file(path: &Path) -> Result<Value, (ConfigLoadError, Value)> {
    macro_rules! pack_error {
        ($err:expr) => {
            match $err {
                Ok(val) => Ok(val),
                Err(e) => Err((e.into(), Value::Object(HashMap::new()))),
            }
        };
    }

    // json5 doesn't support reading from reader
    let content = pack_error!(std::fs::read_to_string(path))?;
    let config_value = pack_error!(json5::from_str::<Value>(&content))?;
    let values_with_extensions = load_extension_blocks(config_value)?;
    Ok(values_with_extensions)
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
                    let mut base_config = load_values_from_file(Path::new(file_path))?;
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
