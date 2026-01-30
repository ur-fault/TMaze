use dpad::dpad_theme_resolver;

use crate::settings::theme::ThemeResolver;

pub mod dpad;
mod screens;

pub use screens::*;

pub fn usecase_ui_theme_resolver() -> ThemeResolver {
    let mut resolver = ThemeResolver::new();
    resolver
        .extend(dpad_theme_resolver())
        .extend(style_browser::style_browser_theme_resolver());

    resolver
}
