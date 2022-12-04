pub use std::time::Duration;

pub use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent},
    terminal::size,
};
pub use masof::{Color, ContentStyle, Renderer};
pub use substring::Substring;

use crate::helpers;
use crate::core::*;

pub mod draw;
pub mod menu;
pub mod popup;
pub mod progressbar;

pub use draw::*;
pub use menu::*;
pub use popup::*;
pub use progressbar::*;

#[derive(Debug)]
pub struct CrosstermError(pub crossterm::ErrorKind);

impl From<masof::renderer::Error> for CrosstermError {
    fn from(error: masof::renderer::Error) -> Self {
        match error {
            masof::renderer::Error::CrossTermError(e) => {
                Self(e)
            }
            _ => panic!("Unexpected error: {}", error),
        }
    }
}

impl From<crossterm::ErrorKind> for CrosstermError {
    fn from(error: crossterm::ErrorKind) -> Self {
        Self(error)
    }
}

pub fn box_center_screen(box_dims: Dims) -> Result<Dims, CrosstermError> {
    let size_u16 = size()?;
    Ok(helpers::box_center(
        Dims(0, 0),
        Dims(size_u16.0 as i32, size_u16.1 as i32),
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