use std::io;

use tmaze::{app::app::App, ui::menu};

fn main() -> io::Result<()> {
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

    let mut app = App::empty();

    let menu = menu::Menu::new(menu_config).into_activity(&app);
    app.activities_mut().push(menu);

    app.run();

    Ok(())
}
