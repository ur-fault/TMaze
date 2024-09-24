use std::io;
pub use std::time::Duration;

use crate::{
    renderer::Frame,
    settings::theme::{Theme, ThemeResolver},
};

pub mod button;
pub mod draw_fn;
pub mod helpers;
pub mod menu;
pub mod popup;
pub mod progressbar;
pub mod rect;
pub mod usecase;

pub use button::*;
pub use draw_fn::*;
pub use helpers::*;
pub use menu::*;
pub use popup::*;
pub use progressbar::*;
pub use rect::*;

pub trait Screen {
    fn draw(&self, frame: &mut Frame, theme: &Theme) -> io::Result<()>;
}

pub fn theme_resolver() -> ThemeResolver {
    let mut resolver = ThemeResolver::new();

    resolver
        .link("text", "") // "" is same as "default"
        .link("border", "")
        .link("highlight", "")
        .link("background", "") // TODO: use
        .link("dim", "");

    resolver
        .extend(button::button_theme_resolver())
        .extend(menu::menu_theme_resolver())
        .extend(popup::popup_theme_resolver())
        .extend(progressbar::progressbar_theme_resolver())
        .extend(rect::rect_theme_resolver());

    resolver
}
