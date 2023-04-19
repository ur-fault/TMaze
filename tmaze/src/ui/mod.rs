pub use std::time::Duration;

use crossterm::Result as CResult;
pub use substring::Substring;

use crate::core::*;
use crate::helpers;
use crate::helpers::is_release;
use crate::renderer::helpers::term_size;

pub mod draw;
pub mod input;
pub mod menu;
pub mod popup;
pub mod progressbar;

pub use draw::*;
pub use input::*;
pub use menu::*;
pub use popup::*;
pub use progressbar::*;

#[derive(Debug)]
pub struct CrosstermError(pub crossterm::ErrorKind);

impl From<crossterm::ErrorKind> for CrosstermError {
    fn from(error: crossterm::ErrorKind) -> Self {
        Self(error)
    }
}

#[derive(Debug)]
pub enum GenericUIError {
    Back,
    Quit,
    Crossterm(CrosstermError),
}

impl From<CrosstermError> for GenericUIError {
    fn from(error: CrosstermError) -> Self {
        Self::Crossterm(error)
    }
}

impl From<MenuError> for GenericUIError {
    fn from(error: MenuError) -> Self {
        match error {
            MenuError::CrosstermError(e) => Self::Crossterm(e),
            MenuError::Exit => Self::Back,
            MenuError::FullQuit => Self::Quit,
            MenuError::EmptyMenu => panic!("Empty menu"),
        }
    }
}

impl From<crossterm::ErrorKind> for GenericUIError {
    fn from(error: crossterm::ErrorKind) -> Self {
        Self::Crossterm(error.into())
    }
}

pub fn box_center_screen(box_dims: Dims) -> Result<Dims, CrosstermError> {
    let size_u16 = term_size();
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

pub fn wait_for_key() -> CResult<KeyCode> {
    let mut e = crossterm::event::read();
    loop {
        match e {
            Ok(event) => match event {
                Event::Key(KeyEvent { code, kind, .. }) if !is_release(kind) => return Ok(code),
                _ => e = crossterm::event::read(),
            },
            Err(e) => return Err(e),
        }
    }
}
