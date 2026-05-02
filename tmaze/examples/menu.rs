use tmaze::{
    app::app::{App, AppOptions},
    settings::ConfigSource,
    ui::menu,
};

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

    let menu = menu::Menu::try_new(menu_config).into_activity();
    let mut app = App::new(AppOptions {
        read_only: true,
        main_activity: Some(menu),
        config_source: ConfigSource::Default,
    });

    app.run();
}
