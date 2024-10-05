use tmaze::settings::theme::{Color, ThemeResolver};

fn main() {
    use std::env::args;

    use tmaze::settings::theme::ThemeDefinition;

    let path = args().nth(1).expect("No path given");

    let theme = ThemeDefinition::load_by_path(path.into()).expect("Failed to load theme");
    let mut resolver = ThemeResolver::new();

    let mut popup_res = ThemeResolver::new();
    popup_res
        .link("popup.ui", "ui")
        .link("popup.text", "text")
        .link("popup.title", "popup.text")
        .link("popup.content", "popup.text");

    let mut button_res = ThemeResolver::new();
    button_res
        .link("button.ui", "ui")
        .link("button.text", "text")
        .link("button.hover.ui", "button.ui")
        .link("button.hover.text", "button.text");

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
