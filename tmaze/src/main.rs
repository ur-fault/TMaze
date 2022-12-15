mod game;
mod helpers;
mod settings;
mod ui;

use clap::Parser;
use cmaze::{core, maze};
use game::{App, GameError};

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {}

fn main() -> Result<(), GameError> {
    let _args = Args::parse();

    App::new().run()
}
