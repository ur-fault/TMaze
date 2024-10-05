use std::{fmt::Display, ops, path::PathBuf};

use crossterm::style::{Attributes, ContentStyle};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    helpers::constants::paths::theme_file_path, settings::attribute::deserialize_attributes,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    styles: HashMap<String, Style>,
}

impl Theme {
    pub fn get(&self, key: &str) -> Style {
        let Some(style) = self.styles.get(key) else {
            panic!("style not found: {}", key);
        };

        *style
    }

    pub fn extract<const N: usize>(&self, keys: [&str; N]) -> [Style; N] {
        keys.map(|key| self.get(key))
    }
}

impl ops::Index<&str> for Theme {
    type Output = Style;

    fn index(&self, key: &str) -> &Self::Output {
        &self.styles[key]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ThemeDefinition {
    meta: Option<HashMap<String, String>>,
    styles: HashMap<String, StyleIdent>,
}

impl ThemeDefinition {
    pub fn load_by_name(path: &str) -> Result<Self, LoadError> {
        Self::load_by_path(theme_file_path(path))
    }

    pub fn load_by_path(path: PathBuf) -> Result<Self, LoadError> {
        log::debug!("Loading theme from {:?}", path);

        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .expect("No extension");

        match ext {
            "toml" => Self::load_toml(path),
            "json" | "json5" => Self::load_json(path),
            _ => Err(LoadError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Unknown file extension",
            ))),
        }
    }

    fn load_json(name: PathBuf) -> Result<Self, LoadError> {
        let content = std::fs::read_to_string(name)?;
        let theme: Self = json5::from_str(&content)?;
        Ok(theme)
    }

    fn load_toml(name: PathBuf) -> Result<Self, LoadError> {
        let content = std::fs::read_to_string(name)?;
        let theme: Self = toml::from_str(&content)?;
        Ok(theme)
    }

    pub fn get(&self, key: &str) -> Option<StyleIdent> {
        if let Some(style) = self.styles.get(key) {
            Some(style.clone())
        } else if key == "default" {
            Some(StyleIdent::Style(Style::default()))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum StyleIdent {
    Style(Style),
    Ref(String),
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct Style {
    pub bg: Option<Color>,
    pub fg: Option<Color>,
    #[serde(deserialize_with = "deserialize_attributes", default)]
    pub attr: Attributes,
}

impl Style {
    pub fn fg(color: Color) -> Self {
        Self {
            fg: Some(color),
            ..Self::default()
        }
    }

    pub fn bg(color: Color) -> Self {
        Self {
            bg: Some(color),
            ..Self::default()
        }
    }

    pub fn swap(self) -> Self {
        Style {
            bg: self.fg,
            fg: self.bg,
            ..self
        }
    }

    pub fn invert(self) -> Self {
        Style {
            fg: Some(self.bg.unwrap_or(Color::Named(NamedColor::Black))),
            bg: Some(self.fg.unwrap_or(Color::Named(NamedColor::White))),
            ..self
        }
    }

    pub fn to_cross(self) -> ContentStyle {
        self.into()
    }
}

impl From<Style> for ContentStyle {
    fn from(value: Style) -> Self {
        ContentStyle {
            foreground_color: value.fg.map(|c| c.into()),
            background_color: value.bg.map(|c| c.into()),
            attributes: value.attr,
            ..ContentStyle::default()
        }
    }
}

impl ops::BitOr for Style {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Self {
            bg: self.bg.or(rhs.bg),
            fg: self.fg.or(rhs.fg),
            attr: self.attr | rhs.attr,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Color {
    // TODO: 256
    RGB(u8, u8, u8),
    Named(NamedColor),
    #[serde(deserialize_with = "deserialize_hex")]
    Hex(u8, u8, u8),
}

impl From<Color> for crossterm::style::Color {
    fn from(value: Color) -> Self {
        use crossterm::style::Color as CsColor;
        use NamedColor as NmColor;

        // could use `mem::transmute` here, but this is more future proof
        match value {
            Color::Named(named) => match named {
                NmColor::Black => CsColor::Black,
                NmColor::DarkGrey => CsColor::DarkGrey,
                NmColor::Red => CsColor::Red,
                NmColor::DarkRed => CsColor::DarkRed,
                NmColor::Green => CsColor::Green,
                NmColor::DarkGreen => CsColor::DarkGreen,
                NmColor::Yellow => CsColor::Yellow,
                NmColor::DarkYellow => CsColor::DarkYellow,
                NmColor::Blue => CsColor::Blue,
                NmColor::DarkBlue => CsColor::DarkBlue,
                NmColor::Magenta => CsColor::Magenta,
                NmColor::DarkMagenta => CsColor::DarkMagenta,
                NmColor::Cyan => CsColor::Cyan,
                NmColor::DarkCyan => CsColor::DarkCyan,
                NmColor::White => CsColor::White,
                NmColor::Grey => CsColor::Grey,
            },
            Color::RGB(r, g, b) | Color::Hex(r, g, b) => CsColor::Rgb { r, g, b },
        }
    }
}

pub fn deserialize_hex<'de, D>(deserializer: D) -> Result<(u8, u8, u8), D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if !(s.len() == 7 || s.len() == 4) {
        panic!("invalid hex color: {:?}", s);
    }
    let s = s.trim_start_matches('#');
    assert!(s.len() == 6 || s.len() == 3, "invalid hex color: {:?}", s);

    let r = u8::from_str_radix(&s[0..2], 16).map_err(serde::de::Error::custom)?;
    let g = u8::from_str_radix(&s[2..4], 16).map_err(serde::de::Error::custom)?;
    let b = u8::from_str_radix(&s[4..6], 16).map_err(serde::de::Error::custom)?;

    Ok((r, g, b))
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NamedColor {
    Black,
    DarkGrey,
    Red,
    DarkRed,
    Green,
    DarkGreen,
    Yellow,
    DarkYellow,
    Blue,
    DarkBlue,
    Magenta,
    DarkMagenta,
    Cyan,
    DarkCyan,
    White,
    Grey,
}

#[derive(Debug, Default)]
pub struct ThemeResolver(HashMap<String, String>);

#[allow(dead_code)]
impl ThemeResolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn link<S: Into<String>>(&mut self, key: S, based_on: S) -> &mut Self {
        self.0.insert(key.into(), based_on.into());
        self
    }

    pub fn get(&self, key: &str) -> &str {
        self.0.get(key).map(|s| s.as_str()).unwrap_or("default")
    }

    pub fn resolve(&self, definition: &ThemeDefinition) -> Theme {
        let mut resolved = HashMap::new();
        for (key, _) in &self.0 {
            let style = self.resolve_style(definition, key);
            resolved.insert(key.clone(), style);
        }
        Theme { styles: resolved }
    }

    fn resolve_style<'a>(&'a self, definition: &'a ThemeDefinition, key: &'a str) -> Style {
        let mut key = key.to_string();
        let mut used = vec![key.clone()];
        loop {
            let style = definition.get(&key);
            match style {
                Some(StyleIdent::Style(style)) => return style,
                Some(StyleIdent::Ref(new_key)) => key = new_key,
                None => key = self.get(&key).to_string(),
            }

            if used.contains(&key) {
                used.push(key.clone());
                panic!("loop detected: {:?}", used);
            }

            used.push(key.clone());
        }
    }

    /// Combine two resolvers into one.
    ///
    /// This will add all the keys from `other` to `self`.
    /// If a key already exists in `self`, it will be overwritten.
    ///
    /// Used as ad-hoc support for modules. For example, `settings` can
    /// add a its own resolver to the global resolver.
    ///
    /// It returns a mutable reference to `self` for chaining.
    pub fn extend(&mut self, other: Self) -> &mut Self {
        self.0.extend(other.0);
        self
    }
}

#[derive(Debug, Error)]
pub enum LoadError {
    Io(#[from] std::io::Error),
    Toml(#[from] toml::de::Error),
    Json(#[from] json5::Error),
}

impl Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LoadError::Io(e) => write!(f, "IO error: {}", e),
            LoadError::Toml(e) => write!(f, "TOML parse error: {}", e),
            LoadError::Json(e) => write!(f, "JSON parse error: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolver() {
        let mut resolver = ThemeResolver::new();
        resolver.link("text", "");
        resolver.link("border", "text");
        resolver.link("item", "unknown");

        // resolver.link("loop A", "loop B");
        // resolver.link("loop B", "loop A");

        let default_style = Style {
            bg: None,
            fg: None,
            attr: Attributes::default(),
        };
        let default_style = Some(&default_style);
        let text_style = Style {
            bg: Some(Color::Named(NamedColor::Black)),
            fg: Some(Color::Named(NamedColor::White)),
            attr: Attributes::default(),
        };

        let definition = ThemeDefinition {
            styles: [("text".into(), StyleIdent::Style(text_style))]
                .iter()
                .cloned()
                .collect(),
            meta: None,
        };

        let theme = resolver.resolve(&definition);

        assert_eq!(
            theme.styles.get("text"),
            Some(&Style {
                bg: Some(Color::Named(NamedColor::Black)),
                fg: Some(Color::Named(NamedColor::White)),
                attr: Attributes::default()
            })
        );

        assert_eq!(theme.styles.get("unknown"), None);

        assert_eq!(theme.styles.get("border"), Some(&text_style));

        assert_eq!(theme.styles.get("item"), default_style);
    }

    #[test]
    fn resolver_loop() {
        use std::panic;

        let mut resolver = ThemeResolver::new();
        resolver.link("loop A", "loop B");
        resolver.link("loop B", "loop A");

        let definition = ThemeDefinition {
            styles: HashMap::new(),
            meta: None,
        };

        let result = panic::catch_unwind(|| resolver.resolve(&definition));

        assert!(result.is_err());
    }
}
