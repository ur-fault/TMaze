use tmaze::settings::theme::{Color, ThemeResolver};

fn main() {
    use std::env::args;

    use tmaze::settings::theme::ThemeDefinition;

    let path = args().nth(1).expect("No path given");

    let theme = ThemeDefinition::load_by_path(path.into()).expect("Failed to load theme");
    let mut resolver = ThemeResolver::new();

    let mut popup_res = ThemeResolver::new();
    popup_res
        .link("popup_ui", "ui")
        .link("popup_text", "text")
        .link("popup_title", "popup_text")
        .link("popup_content", "popup_text");

    let mut button_res = ThemeResolver::new();
    button_res
        .link("button_ui", "ui")
        .link("button_text", "text")
        .link("button_hover_ui", "button_ui")
        .link("button_hover_text", "button_text");

    resolver
        .link("text", "")
        .link("ui", "")
        .link("ui", "")
        .extend(popup_res)
        .extend(button_res);

    let theme = resolver.resolve(&theme);
    println!("{:#?}", theme);

    println!("{}", json5::to_string(&Color::RGB(0, 128, 255)).unwrap());
}
