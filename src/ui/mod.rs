pub use std::time::Duration;

pub use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent},
    terminal::size,
};
pub use masof::{Color, ContentStyle, Renderer};
pub use substring::Substring;

use crate::helpers;
use crate::core::*;

mod draw;
mod menu;
mod popup;
mod progressbar;

pub use draw::*;
pub use menu::*;
pub use popup::*;
pub use progressbar::*;

pub fn box_center_screen(box_dims: Dims) -> Result<Dims, Error> {
    let size_u16 = size()?;
    Ok(helpers::box_center(
        (0, 0),
        (size_u16.0 as i32, size_u16.1 as i32),
        box_dims,
    ))
}

pub fn format_duration(dur: Duration) -> String {
    format!(
        "{}m{:.1}s",
        dur.as_secs() / 60,
        (dur.as_secs() % 60) as f32 + dur.subsec_millis() as f32 / 1000f32,
    )
}
