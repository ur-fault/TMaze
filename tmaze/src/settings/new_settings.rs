use std::{fmt::Display, sync::Arc};

use hashbrown::HashMap;

use crate::{helpers::TupleMap, settings::model::{Config, PartialConfig, Value}};

struct Settings {
    inner: Arc<SettingsInner>,
}

impl Settings {
    /// Loads settings from configuration files.
    ///
    /// Returns the loaded settings and a boolean indicating whether any errors/warnings occurred
    /// during loading.
    ///
    /// TODO: Report the actual errors/warnings to the user.
    fn load() -> (Self, bool) {
        SettingsInner::load().map_first(|inner| Self {
            inner: Arc::new(inner),
        })
    }
}

struct SettingsInner {
    // ui_layer: PartialConfig,
    config_layer: PartialConfig,
    base: Config,
}

impl SettingsInner {
    fn load() -> (Self, bool) {
        let mut errored = false;

        let base_config = Config::default();
        let config_layer = match load_config_from_file("config.json5") {
            Ok(config) => config,
            Err(_err) => {
                errored = true;
                PartialConfig::default()
            }
        };

        let config = Self {
            config_layer,
            base: base_config,
        };

        (config, errored)
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

fn load_config_from_file(path: &str) -> Result<PartialConfig, (ConfigLoadError, PartialConfig)> {
    match load_values_from_file(path) {
        Ok(value) => PartialConfig::try_from(value)
            .map_err(|(e, val)| (ConfigLoadError::SettingsFormatError(e), val)),
        Err((e, val)) => Err((e, PartialConfig::try_from(val).unwrap_or_default())),
    }
}

fn load_values_from_file(path: &str) -> Result<Value, (ConfigLoadError, Value)> {
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
    /// TODO: Allow rules customization in the future, for example to support list contatenation.
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
                    let mut base_config = load_values_from_file(file_path)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_deserialize() {
        let json_data = r#"
        {
            "name": "Example",
            "enabled": true,
            "threshold": 10.5,
            "count": 42,
            "items": [1, 2, 3],
            "settings": {
                "option1": "value1",
                "option2": false
            }
        }
        "#;

        let parsed: Value = serde_json::from_str(json_data).unwrap();

        if let Value::Object(map) = parsed {
            assert_eq!(map.get("name"), Some(&Value::String("Example".to_string())));
            assert_eq!(map.get("enabled"), Some(&Value::Bool(true)));
            assert_eq!(map.get("threshold"), Some(&Value::Float(10.5)));
            assert_eq!(map.get("count"), Some(&Value::Int(42)));

            if let Some(Value::List(items)) = map.get("items") {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], Value::Int(1));
                assert_eq!(items[1], Value::Int(2));
                assert_eq!(items[2], Value::Int(3));
            } else {
                panic!("Expected 'items' to be a list");
            }

            if let Some(Value::Object(settings)) = map.get("settings") {
                assert_eq!(
                    settings.get("option1"),
                    Some(&Value::String("value1".to_string()))
                );
                assert_eq!(settings.get("option2"), Some(&Value::Bool(false)));
            } else {
                panic!("Expected 'settings' to be an object");
            }
        } else {
            panic!("Expected top-level value to be an object");
        }
    }
}
