use tmaze::{
    app::{game::MainMenu, Activity, App, GameError},
    settings::Settings,
};

#[cfg(feature = "updates")]
use tmaze::updates;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version, author, about, name = "tmaze")]
struct Args {
    #[clap(short, long, action, help = "Reset config to default and quit")]
    reset_config: bool,
    #[clap(short, long, action, help = "Show config path and quit")]
    show_config_path: bool,
    #[clap(long, help = "Show config in debug format and quit")]
    debug_config: bool,
    #[clap(short, long, action, help = "Delete all saved data and quit")]
    delete_data: bool,
}

fn main() -> Result<(), GameError> {
    let _args = Args::parse();

    if _args.reset_config {
        Settings::reset_config(Settings::default_path());
        return Ok(());
    }

    if _args.show_config_path {
        let settings_path = Settings::default_path();
        if let Some(s) = settings_path.to_str() {
            println!("{}", s);
        } else {
            println!("{:?}", settings_path);
        }
        return Ok(());
    }

    if _args.debug_config {
        println!("{:#?}", Settings::load(Settings::default_path()));
        return Ok(());
    }

    if _args.delete_data {
        let _ = std::fs::remove_file(tmaze::data::SaveData::default_path());
        return Ok(());
    }

    better_panic::install();

    let mut app = App::empty();
    let menu = MainMenu::new(&app.data().settings);
    app.activities_mut()
        .push(Activity::new_base_boxed("main menu", menu));

    #[cfg(feature = "updates")]
    updates::check(&mut app.data_mut());

    app.run();

    Ok(())
}
