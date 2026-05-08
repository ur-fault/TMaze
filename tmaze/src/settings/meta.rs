use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::settings::config_utils::{self, LenientConvert};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub website: String,
    #[serde(default = "default_version")]
    pub format_version: i32,
}

impl Default for Meta {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            authors: Vec::new(),
            version: String::new(),
            website: String::new(),
            format_version: default_version(),
        }
    }
}

fn default_version() -> i32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContent<T> {
    #[serde(default)]
    pub meta: Meta,

    #[serde(flatten)]
    pub content: T,
}

impl<T> UserContent<T> {
    pub fn new(meta: Meta, content: T) -> Self {
        Self { meta, content }
    }

    pub fn map_content<U>(self, f: impl FnOnce(T) -> U) -> UserContent<U> {
        UserContent {
            meta: self.meta,
            content: f(self.content),
        }
    }
}

impl<T> Deref for UserContent<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl<T> DerefMut for UserContent<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}

impl<T: LenientConvert> LenientConvert for UserContent<T> {
    fn convert(
        mut value: config_utils::Value,
        context: &mut config_utils::ConvertContext,
    ) -> Option<Self> {
        use config_utils::Value;
        use serde_json::{from_value as from_json, to_value as to_json};

        let Value::Object(map) = &mut value else {
            context.err("expected an object");
            return None;
        };

        let meta = map
            .remove("meta")
            .and_then(|m| to_json(m).ok())
            .and_then(|v| from_json(v).ok())
            .unwrap_or_default();

        let data = T::convert(value, context)?;
        Some(Self {
            meta,
            content: data,
        })
    }
}

impl<T: Default> Default for UserContent<T> {
    fn default() -> Self {
        Self {
            meta: Meta::default(),
            content: T::default(),
        }
    }
}
