use std::{
    borrow::{Borrow, Cow},
    fmt,
    ops::Deref,
};

use cmaze::dims::Dims;

use substring::Substring;
use unicode_width::UnicodeWidthStr as _;

use crate::{
    renderer::{drawable::Drawable, Frame},
    settings::theme::Style,
    ui::draw_str,
};

pub fn trim_center(text: &str, width: usize) -> &str {
    let str_width = text.width();
    if str_width <= width {
        return text;
    }

    let offset = (str_width - width) / 2;
    text.substring(offset, offset + width)
}

/// Returns the first string that fits within `max_size` width.
///
/// Returns the last string if none fits. So it's *NOT* guaranteed that the returned string fits.
/// It's up to the caller to handle this case. Perhaps by trimming the string ([`trim_center`]).
///
/// # Panics
///
/// Panics if the iterator is empty.
pub fn multisize_string<S>(strings: impl IntoIterator<Item = S>, max_size: usize) -> S
where
    S: Borrow<str>,
{
    let strings = &mut strings.into_iter();

    // NOTE: we cannot use `Iterator::find` because we need at least the last element,
    // if none fits
    let mut current = strings.next().unwrap();
    while current.borrow().width() > max_size {
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

impl Drawable<Style> for MbyStaticStr {
    fn draw(&self, Dims(x, y): Dims, frame: &mut dyn Frame, styles: Style) {
        draw_str(frame, x, y, self, styles);
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
