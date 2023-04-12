mod game;
mod helpers;
mod renderer;
mod settings;
mod ui;

use clap::Parser;
use cmaze::{core, maze};
use game::{App, GameError};

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    #[clap(short, long, action, help = "Reset config to default and quit")]
    reset_config: bool,
}

fn main() -> Result<(), GameError> {
    let _args = Args::parse();

    if _args.reset_config {
        settings::Settings::reset_config(settings::Settings::default_path());
        return Ok(());
    }

    App::new().run()
}
