use std::time::Duration;

use cmaze::dims::Dims;
use crossterm::style::{Attribute, Color, ContentStyle};
use unicode_width::UnicodeWidthStr;

use crate::{
    helpers::{self, strings::multisize_string},
    renderer::{drawable::Drawable, helpers::term_size},
    settings::theme::Style,
};

pub fn center_box_in_screen(box_dims: Dims) -> Dims {
    let size_u16 = term_size();
    helpers::box_center(
        Dims(0, 0),
        Dims(size_u16.0 as i32, size_u16.1 as i32),
        box_dims,
    )
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

pub fn foreground_style(color: Color) -> ContentStyle {
    ContentStyle {
        foreground_color: Some(color),
        ..ContentStyle::default()
    }
}

pub fn background_style(color: Color) -> ContentStyle {
    ContentStyle {
        background_color: Some(color),
        ..ContentStyle::default()
    }
}

pub fn style_with_attribute(style: ContentStyle, attr: Attribute) -> ContentStyle {
    ContentStyle {
        attributes: style.attributes | attr,
        ..style
    }
}

pub struct CapsuleText(pub String);

impl Drawable<Style> for CapsuleText {
    fn draw(&self, pos: Dims, frame: &mut impl crate::renderer::Frame, style: Style) {
        frame.draw(pos + Dims(0, 0), '', style.invert());
        frame.draw(pos + Dims(1, 0), self.0.as_str(), style);
        frame.draw(
            pos + Dims(self.0.width() as i32 + 1, 0),
            '',
            style.invert(),
        );
    }
}
