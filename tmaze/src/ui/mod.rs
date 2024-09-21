use std::io;
pub use std::time::Duration;

use crate::{core::*, renderer::Frame, settings::ColorScheme};

pub mod button;
pub mod draw_fn;
pub mod helpers;
pub mod menu;
pub mod popup;
pub mod progressbar;
pub mod usecase;
pub mod rect;

pub use button::*;
pub use draw_fn::*;
pub use helpers::*;
pub use menu::*;
pub use popup::*;
pub use progressbar::*;
pub use rect::*;

pub trait Screen {
    fn draw(&self, frame: &mut Frame, color_scheme: &ColorScheme) -> io::Result<()>;
}
