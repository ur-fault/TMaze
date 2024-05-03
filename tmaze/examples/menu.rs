use std::io;

use tmaze::{
    app::{activity::Activity, app::App},
    renderer::Renderer,
    ui::{menu, Menu},
};

fn main() -> io::Result<()> {
    let menu_config = menu::MenuConfig::new(
        "Menu",
        vec![
            "Option 1".to_string(),
            "Option 2".to_string(),
            "Option 3".to_string(),
        ],
    );

    let menu = Menu::new(menu_config);

    let mut app = App::new(Activity::new("example", "menu", Box::new(menu)));

    app.run();

    Ok(())
}
