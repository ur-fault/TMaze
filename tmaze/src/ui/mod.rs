use std::io;
pub use std::time::Duration;

use crate::{core::*, renderer::Frame, settings::ColorScheme};

pub mod button;
pub mod draw;
pub mod helpers;
pub mod menu;
pub mod popup;
pub mod progressbar;

pub use button::*;
pub use draw::*;
pub use helpers::*;
pub use menu::*;
pub use popup::*;
pub use progressbar::*;

pub trait Screen {
    fn draw(&self, frame: &mut Frame, color_scheme: &ColorScheme) -> io::Result<()>;
}
