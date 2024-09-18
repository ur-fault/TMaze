use std::{borrow::Cow, fmt, ops::Deref};

use cmaze::core::Dims;
use substring::Substring;
use unicode_width::UnicodeWidthStr as _;

use crate::{renderer::drawable::Drawable, ui::draw_str};

pub fn trim_center(text: &str, width: usize) -> &str {
    let str_width = text.width();
    if str_width <= width {
        return text;
    }

    let offset = (str_width - width) / 2;
    text.substring(offset, offset + width)
}


pub fn multisize_string_fast<'a>(
    strings: impl IntoIterator<Item = &'a str>,
    max_size: usize,
) -> &'a str {
    let strings = &mut strings.into_iter();
    let mut current = strings.next().unwrap();
    while current.width() > max_size {
        current = strings.next().unwrap();
    }

    current
}

pub fn multisize_string(strings: impl IntoIterator<Item = String>, max_size: usize) -> String {
    let strings = &mut strings.into_iter();
    let mut current = strings.next().unwrap();
    while current.width() > max_size {
        current = strings.next().unwrap();
    }

    current
}

pub enum MbyStaticStr {
    Static(&'static str),
    Owned(String),
}

impl MbyStaticStr {
    pub fn as_ref_cow(&self) -> Cow<str> {
        match self {
            Self::Static(s) => Cow::Borrowed(s),
            Self::Owned(s) => Cow::Borrowed(s.as_str()),
        }
    }
}

impl fmt::Display for MbyStaticStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Static(s) => write!(f, "{}", s),
            Self::Owned(s) => write!(f, "{}", s),
        }
    }
}

impl fmt::Debug for MbyStaticStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Static(s) => write!(f, "{:?}", s),
            Self::Owned(s) => write!(f, "{:?}", s),
        }
    }
}

impl Drawable for MbyStaticStr {
    fn draw(&self, pos: Dims, frame: &mut crate::renderer::Frame) {
        self.draw_with_style(pos, frame, crossterm::style::ContentStyle::default());
    }

    fn draw_with_style(
        &self,
        Dims(x, y): Dims,
        frame: &mut crate::renderer::Frame,
        style: crossterm::style::ContentStyle,
    ) {
        draw_str(frame, x, y, self, style);
    }
}

impl From<&'static str> for MbyStaticStr {
    fn from(s: &'static str) -> Self {
        Self::Static(s)
    }
}

impl From<String> for MbyStaticStr {
    fn from(s: String) -> Self {
        Self::Owned(s)
    }
}

impl From<MbyStaticStr> for Cow<'static, str> {
    fn from(value: MbyStaticStr) -> Self {
        match value {
            MbyStaticStr::Static(s) => Cow::Borrowed(s),
            MbyStaticStr::Owned(s) => Cow::Owned(s),
        }
    }
}

impl Deref for MbyStaticStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Static(s) => s,
            Self::Owned(s) => s,
        }
    }
}
