use std::time::Duration;

use cmaze::core::Dims;
use crossterm::style::{Attribute, Color, ContentStyle};
use unicode_width::UnicodeWidthStr;

use crate::{helpers, renderer::helpers::term_size};

pub fn center_box_in_screen(box_dims: Dims) -> Dims {
    let size_u16 = term_size();
    helpers::box_center(
        Dims(0, 0),
        Dims(size_u16.0 as i32, size_u16.1 as i32),
        box_dims,
    )
}

pub fn format_duration(dur: Duration) -> String {
    format!(
        "{}m{:.1}s",
        dur.as_secs() / 60,
        (dur.as_secs() % 60) as f32 + dur.subsec_millis() as f32 / 1000f32,
    )
}

pub fn smart_format_duration(dur: Duration, fract: bool) -> String {
    if dur.as_secs() < 60 {
        if fract {
            format!(
                "{:.1}s",
                dur.as_secs() as f32 + dur.subsec_millis() as f32 / 1000f32
            )
        } else {
            format!("{}s", dur.as_secs())
        }
    } else if fract {
        format_duration(dur)
    } else {
        format!(
            "{}m{}s",
            dur.as_secs() / 60,
            dur.as_secs() % 60 + dur.subsec_millis() as u64 / 1000
        )
    }
}

pub fn multisize_duration_format(dur: Duration, max_size: usize) -> String {
    multisize_string(
        [
            smart_format_duration(dur, true),
            smart_format_duration(dur, false),
        ],
        max_size,
    )
}

pub fn multisize_string(strings: impl IntoIterator<Item = String>, max_size: usize) -> String {
    let strings = &mut strings.into_iter();
    let mut current = strings.next().unwrap();
    while current.width() > max_size {
        current = strings.next().unwrap();
    }

    current
}

pub fn style_with_attribute(style: ContentStyle, attr: Attribute) -> ContentStyle {
    ContentStyle {
        attributes: style.attributes | attr,
        ..style
    }
}

pub fn swap_fg_bg(style: ContentStyle) -> ContentStyle {
    ContentStyle {
        background_color: Some(style.foreground_color.unwrap_or(Color::White)),
        foreground_color: Some(style.background_color.unwrap_or(Color::Black)),
        ..style
    }
}
