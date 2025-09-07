use std::{borrow::Cow, collections::BTreeMap, fmt::Display, ops, path::PathBuf, rc::Rc};

use crossterm::style::{Attributes, ContentStyle};
use hashbrown::HashMap;
use serde::{de::Error, Deserialize, Serialize};
use thiserror::Error;

use crate::{
    helpers::{constants::paths::theme_file_path, ToDebug},
    settings::attribute::deserialize_attributes,
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
        let Some(style) = self.styles.get(key) else {
            panic!("style not found: {}", key);
        };

        style
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ThemeDefinition {
    meta: Option<HashMap<String, String>>,
    styles: HashMap<String, StyleIdent>,
}

// For some reason, Rust concat! doesn't allow const, so we have to use a macro
macro_rules! default_theme_name {
    () => {
        "default_theme.json5"
    };
}
const DEFAULT_THEME_NAME: &str = default_theme_name!();
const DEFAULT_THEME: &str = include_str!(concat!("./", default_theme_name!()));

impl ThemeDefinition {
    pub fn parse_default() -> Self {
        json5::from_str(DEFAULT_THEME).expect("default theme should be always valid")
    }

    pub fn load_default(read_only: bool) -> Result<Self, LoadError> {
        if read_only {
            return Ok(Self::parse_default());
        }

        let result = Self::prepare_default_theme();
        match result {
            Ok(theme) => Ok(theme),
            Err(e) => {
                log::error!("Failed to prepare default theme: {}", e);
                Err(e)
            }
        }
    }

    fn prepare_default_theme() -> Result<Self, LoadError> {
        let path = theme_file_path(DEFAULT_THEME_NAME);

        std::fs::create_dir_all(path.parent().unwrap())?;
        if !path.exists() {
            std::fs::write(&path, DEFAULT_THEME)?;
        }

        Self::load_by_path(path)
    }

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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct Style {
    pub bg: Option<Color>,
    pub fg: Option<Color>,
    #[serde(deserialize_with = "deserialize_attributes", default)]
    pub attr: Attributes,
    #[serde(default = "default_alpha")]
    pub alpha: u8,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            bg: Default::default(),
            fg: Default::default(),
            attr: Default::default(),
            alpha: default_alpha(),
        }
    }
}

fn default_alpha() -> u8 {
    u8::MAX
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

    pub fn mix(self, on: Self, scheme: &TerminalColorScheme, ignore_fg: bool) -> Self {
        use MixMode::*;

        Style {
            bg: Color::mix(
                Bg {
                    top: self.bg,
                    on: on.bg,
                },
                self.alpha,
                scheme,
            ),
            fg: Color::mix(
                match ignore_fg {
                    true => TransparentFg {
                        top_bg: self.bg,
                        on_fg: on.fg,
                    },
                    false => Fg {
                        fg: self.fg,
                        on: on.bg,
                    },
                },
                self.alpha,
                scheme,
            ),
            attr: self.attr,
            alpha: 255,
        }
    }

    pub fn to_named(mut self, scheme: &TerminalColorScheme) -> Self {
        if let Some(Color::RGB(r, g, b) | Color::Hex(r, g, b)) = self.fg {
            self.fg = scheme.closest_color((r, g, b), true).map(Color::Named);
        }

        if let Some(Color::RGB(r, g, b) | Color::Hex(r, g, b)) = self.bg {
            self.bg = scheme.closest_color((r, g, b), false).map(Color::Named);
        }

        self
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Color {
    // TODO: 256 (ansi)
    RGB(u8, u8, u8),
    Named(NamedColor),
    #[serde(deserialize_with = "deserialize_hex")]
    Hex(u8, u8, u8),
}

pub enum MixMode {
    Bg {
        top: Option<Color>,
        on: Option<Color>,
    },
    Fg {
        fg: Option<Color>,
        on: Option<Color>,
    },
    TransparentFg {
        top_bg: Option<Color>,
        on_fg: Option<Color>,
    },
}

impl Color {
    pub fn as_text(&self) -> String {
        match self {
            Color::Named(named) => named.to_debug(),
            Color::RGB(r, g, b) | Color::Hex(r, g, b) => format!("#{:02X}{:02X}{:02X}", r, g, b),
        }
    }

    pub fn mix(mode: MixMode, alpha: u8, scheme: &TerminalColorScheme) -> Option<Self> {
        use Color::*;
        use MixMode::*;

        let alpha = alpha as u16;

        if alpha == 0 {
            return match mode {
                Bg { on, .. } => on,
                Fg { on, .. } => Some(on.unwrap_or(RGB(
                    scheme.primary_bg.0,
                    scheme.primary_bg.1,
                    scheme.primary_bg.2,
                ))),
                TransparentFg { on_fg: on, .. } => on,
            };
        }

        if alpha == 255 {
            return match mode {
                Bg { top, .. } => top,
                Fg { fg, .. } => fg,
                TransparentFg { top_bg, .. } => Some(top_bg.unwrap_or(RGB(
                    scheme.primary_bg.0,
                    scheme.primary_bg.1,
                    scheme.primary_bg.2,
                ))),
            };
        }

        let classify = |color, default| match color {
            None => default,
            Some(Named(name)) => name.to_rgb(scheme),
            Some(RGB(r, g, b) | Hex(r, g, b)) => (r, g, b),
        };

        let [(r1, g1, b1), (r2, g2, b2)] = match mode {
            Bg { top, on } => [
                classify(top, scheme.primary_bg),
                classify(on, scheme.primary_bg),
            ],
            Fg { fg, on } => [
                classify(fg, scheme.primary_fg),
                classify(on, scheme.primary_bg),
            ],
            TransparentFg { top_bg, on_fg } => [
                classify(top_bg, scheme.primary_bg),
                classify(on_fg, scheme.primary_fg),
            ],
        };

        let r = (r1 as u16 * alpha + r2 as u16 * (255 - alpha)) / 255;
        let g = (g1 as u16 * alpha + g2 as u16 * (255 - alpha)) / 255;
        let b = (b1 as u16 * alpha + b2 as u16 * (255 - alpha)) / 255;
        Some(RGB(r as u8, g as u8, b as u8))
    }
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

fn deserialize_hex<'de, D>(deserializer: D) -> Result<(u8, u8, u8), D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if !(s.len() == 7 || s.len() == 4) {
        return Err(D::Error::custom(format!(
            "invalid hex color, expected format `#RGB` or `#RRGGBB`: {:?}",
            s
        )));
    }
    let s = s.trim_start_matches('#');
    if !(s.len() == 6 || s.len() == 3) {
        return Err(D::Error::custom(format!(
            "invalid hex color, expected format `#RGB` or `#RRGGBB`: {:?}",
            s
        )));
    }

    let (r, g, b) = if s.len() == 6 {
        (
            u8::from_str_radix(&s[0..2], 16).map_err(D::Error::custom)?,
            u8::from_str_radix(&s[2..4], 16).map_err(D::Error::custom)?,
            u8::from_str_radix(&s[4..6], 16).map_err(D::Error::custom)?,
        )
    } else {
        (
            u8::from_str_radix(&s[0..1], 16).map_err(D::Error::custom)? * 17,
            u8::from_str_radix(&s[1..2], 16).map_err(D::Error::custom)? * 17,
            u8::from_str_radix(&s[2..3], 16).map_err(D::Error::custom)? * 17,
        )
    };

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

impl NamedColor {
    pub fn to_rgb(&self, mapping: &TerminalColorScheme) -> (u8, u8, u8) {
        use NamedColor::*;
        match self {
            Black => mapping.black,
            DarkGrey => mapping.dark_grey,
            Red => mapping.red,
            DarkRed => mapping.dark_red,
            Green => mapping.green,
            DarkGreen => mapping.dark_green,
            Yellow => mapping.yellow,
            DarkYellow => mapping.dark_yellow,
            Blue => mapping.blue,
            DarkBlue => mapping.dark_blue,
            Magenta => mapping.magenta,
            DarkMagenta => mapping.dark_magenta,
            Cyan => mapping.cyan,
            DarkCyan => mapping.dark_cyan,
            White => mapping.white,
            Grey => mapping.grey,
        }
    }
}

pub type Rgb = (u8, u8, u8);

pub type SharedScheme = Rc<TerminalColorScheme>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalColorScheme {
    primary_fg: Rgb,
    primary_bg: Rgb,
    black: Rgb,     // grey
    dark_grey: Rgb, // dark grey
    red: Rgb,
    dark_red: Rgb,
    green: Rgb,
    dark_green: Rgb,
    yellow: Rgb,
    dark_yellow: Rgb,
    blue: Rgb,
    dark_blue: Rgb,
    magenta: Rgb,
    dark_magenta: Rgb,
    cyan: Rgb,
    dark_cyan: Rgb,
    white: Rgb,
    grey: Rgb,
}

impl TerminalColorScheme {
    fn hex(s: &str) -> Rgb {
        if s.len() == 6 {
            (
                u8::from_str_radix(&s[0..2], 16).unwrap(),
                u8::from_str_radix(&s[2..4], 16).unwrap(),
                u8::from_str_radix(&s[4..6], 16).unwrap(),
            )
        } else if s.len() == 3 {
            (
                u8::from_str_radix(&s[0..1], 16).unwrap() * 17,
                u8::from_str_radix(&s[1..2], 16).unwrap() * 17,
                u8::from_str_radix(&s[2..3], 16).unwrap() * 17,
            )
        } else {
            panic!("Invalid hex color: {}", s);
        }
    }

    pub fn named(name: &str) -> Self {
        // TODO: generate from existing terminal schemes, Alacritty mby?
        const H: fn(&str) -> Rgb = TerminalColorScheme::hex;

        match name {
            "catppuccin_mocha" => TerminalColorScheme {
                primary_fg: H("cdd6f4"),
                primary_bg: H("1e1e2e"),
                black: H("45475a"),
                dark_grey: H("585b70"),
                red: H("f38ba8"),
                dark_red: H("f38ba8"),
                green: H("a6e3a1"),
                dark_green: H("a6e3a1"),
                yellow: H("f9e2af"),
                dark_yellow: H("f9e2af"),
                blue: H("89b4fa"),
                dark_blue: H("89b4fa"),
                magenta: H("f5c2e7"),
                dark_magenta: H("f5c2e7"),
                cyan: H("94e2d5"),
                dark_cyan: H("94e2d5"),
                white: H("bac2de"),
                grey: H("a6adc8"),
            },
            _ => panic!("Unknown terminal color scheme: {}", name),
        }
    }

    pub fn closest_color(&self, color: (u8, u8, u8), fg: bool) -> Option<NamedColor> {
        let mut closest = None;
        let mut closest_dist = u32::MAX;

        let colors = [
            (Some(NamedColor::Black), self.black),
            (Some(NamedColor::DarkGrey), self.dark_grey),
            (Some(NamedColor::Red), self.red),
            (Some(NamedColor::DarkRed), self.dark_red),
            (Some(NamedColor::Green), self.green),
            (Some(NamedColor::DarkGreen), self.dark_green),
            (Some(NamedColor::Yellow), self.yellow),
            (Some(NamedColor::DarkYellow), self.dark_yellow),
            (Some(NamedColor::Blue), self.blue),
            (Some(NamedColor::DarkBlue), self.dark_blue),
            (Some(NamedColor::Magenta), self.magenta),
            (Some(NamedColor::DarkMagenta), self.dark_magenta),
            (Some(NamedColor::Cyan), self.cyan),
            (Some(NamedColor::DarkCyan), self.dark_cyan),
            (Some(NamedColor::White), self.white),
            (Some(NamedColor::Grey), self.grey),
            (None, if fg { self.primary_fg } else { self.primary_bg }),
        ];

        for (named, rgb) in colors.iter() {
            let dr = rgb.0 as i32 - color.0 as i32;
            let dg = rgb.1 as i32 - color.1 as i32;
            let db = rgb.2 as i32 - color.2 as i32;
            let dist = (dr * dr + dg * dg + db * db) as u32;
            if dist < closest_dist {
                closest_dist = dist;
                closest = *named;
            }
        }

        closest
    }
}

impl Default for TerminalColorScheme {
    fn default() -> Self {
        TerminalColorScheme {
            primary_fg: (255, 255, 255),
            primary_bg: (0, 0, 0),
            black: (0, 0, 0),
            dark_grey: (64, 64, 64),
            red: (255, 0, 0),
            dark_red: (128, 0, 0),
            green: (0, 255, 0),
            dark_green: (0, 128, 0),
            yellow: (255, 255, 0),
            dark_yellow: (128, 128, 0),
            blue: (0, 0, 255),
            dark_blue: (0, 0, 128),
            magenta: (255, 0, 255),
            dark_magenta: (128, 0, 128),
            cyan: (0, 255, 255),
            dark_cyan: (0, 128, 128),
            white: (255, 255, 255),
            grey: (192, 192, 192),
        }
    }
}

#[derive(Clone, Debug, Default)]
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

impl ThemeResolver {
    pub fn to_map(self) -> HashMap<String, String> {
        self.0
    }

    pub fn as_map(&self) -> &HashMap<String, String> {
        &self.0
    }

    pub fn to_logical_tree(&self) -> StyleNode<'_> {
        fn add<'a, 'b>(
            node: &'b mut StyleNode<'a>,
            segs: &[&'a str],
            root: bool,
        ) -> Option<&'b mut StyleNode<'a>> {
            if segs.is_empty() {
                return None;
            }
            let seg = segs[0];
            let child = if !root {
                let key = format!(".{}", seg);
                node.map
                    .entry(Cow::Owned(key))
                    .or_insert_with(StyleNode::new)
            } else {
                node.map
                    .entry(Cow::from(seg))
                    .or_insert_with(StyleNode::new)
            };

            if segs.len() == 1 {
                return Some(child);
            }

            add(child, &segs[1..], false)
        }

        let mut node = StyleNode::new();
        let theme_resolver = self.as_map();
        for style in theme_resolver.keys() {
            let segs = style.split('.').collect::<Vec<_>>();
            add(&mut node, &segs, true).unwrap().style = Some(style);
        }

        node
    }

    pub fn to_deps_tree(&self) -> StyleNode<'_> {
        fn add<'a>(node: &mut StyleNode<'a>, map: &'a ThemeResolver, key: &str) {
            for (key, _) in map.0.iter().filter(|(_, v)| *v == key) {
                let mut new_node = StyleNode::new();
                new_node.style = Some(key);
                add(&mut new_node, map, key);
                node.map.insert(Cow::Borrowed(key), new_node);
            }
        }

        let mut node = StyleNode::new();
        add(&mut node, self, "");
        node
    }
}

#[derive(Debug)]
pub struct StyleNode<'a> {
    pub map: BTreeMap<Cow<'a, str>, StyleNode<'a>>,
    pub style: Option<&'a str>,
}

#[allow(clippy::new_without_default)]
impl<'a> StyleNode<'a> {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            style: None,
        }
    }

    pub fn new_styled(style: &'a str) -> Self {
        Self {
            map: BTreeMap::new(),
            style: Some(style),
        }
    }

    pub fn print(&self, indent: usize, depth: usize, show_no: bool, no: &mut usize) {
        for (key, node) in &self.map {
            if show_no {
                println!("{no:<depth$}{key}", depth = depth);
            } else {
                println!("{:<depth$}{key}", "", depth = depth);
            }
            node.print(indent, depth + indent, show_no, no);
            *no += 1;
        }
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

        let default_style = Style::default();
        let default_style = Some(&default_style);
        let text_style = Style {
            bg: Some(Color::Named(NamedColor::Black)),
            fg: Some(Color::Named(NamedColor::White)),
            ..Default::default()
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
                ..Default::default()
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

    #[test]
    fn test_default_theme() {
        assert!(json5::from_str::<ThemeDefinition>(DEFAULT_THEME).is_ok());
    }

    #[test]
    fn parse_color() {
        assert_eq!(json5::from_str(r##""#FF0000""##), Ok(Color::Hex(255, 0, 0)));
        assert_eq!(json5::from_str(r##""#F00""##), Ok(Color::Hex(255, 0, 0)));
        assert_eq!(
            json5::from_str::<Color>(r##""#123""##),
            json5::from_str(r##""#112233""##)
        );
        assert!(json5::from_str::<Color>(r###""##23""###).is_err());
        assert!(json5::from_str::<Color>(r###""123""###).is_err());
        assert!(json5::from_str::<Color>(r###""#12""###).is_err());
        assert!(json5::from_str::<Color>(r###""#1234""###).is_err());
    }
}
