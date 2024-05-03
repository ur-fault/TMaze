use tmaze::{app::app::App, ui::menu};

fn main() {
    let menu_config = menu::MenuConfig::new(
        "Menu",
        vec![
            "Option 1".to_string(),
            "Option 2".to_string(),
            "Option 3".to_string(),
        ],
    )
    .counted()
    .default(1);

    // We cannot create an App with menu now,
    // since menu needs &App to correctly name itself.
    // Otherwise we could create menu activity manually.
    let mut app = App::empty();

    let menu = menu::Menu::new(menu_config).into_activity(&app);
    app.activities_mut().push(menu);

    app.run();
}
