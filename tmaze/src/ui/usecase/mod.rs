use dpad::dpad_theme_resolver;

use crate::settings::theme::ThemeResolver;

pub mod dpad;

pub fn usedcase_ui_theme_resolver() -> ThemeResolver {
    let mut resolver = ThemeResolver::new();
    resolver.extend(dpad_theme_resolver());

    resolver
}
