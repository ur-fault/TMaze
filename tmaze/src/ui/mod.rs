use std::io;
pub use std::time::Duration;

use thiserror::Error;

use crate::{core::*, renderer::Renderer};

pub mod draw;
pub mod helpers;
pub mod menu;
pub mod popup;
pub mod progressbar;

pub use draw::*;
pub use helpers::*;
pub use menu::*;
pub use popup::*;
pub use progressbar::*;

pub trait Screen {
    fn draw(&self, renderer: &mut Renderer) -> Result<(), io::Error>;
}
