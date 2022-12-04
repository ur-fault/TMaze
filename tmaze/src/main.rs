mod game;
mod helpers;
mod settings;
mod ui;

use clap::Parser;
use cmaze::{core, maze};
use game::{game::GameError, App};

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {}

fn main() -> Result<(), GameError> {
    let _args = Args::parse();

    App::new().run()
}
