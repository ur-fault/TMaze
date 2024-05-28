use tmaze::{app::app::App, ui::menu};

fn main() {
    let menu_config = menu::MenuConfig::new_from_strings(
        "Menu",
        vec![
            "Option 1".to_string(),
            "Option 2".to_string(),
            "Option 3".to_string(),
        ],
    )
    .counted()
    .default(1);

    let menu = menu::Menu::new(menu_config).into_activity();
    let mut app = App::new(menu);

    app.run();
}
