mod game;
mod helpers;
mod settings;
// mod core;
mod ui;

use game::{Game, game::GameError};
use clap::Parser;
use cmaze::{core, maze};

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {

}

fn main() -> Result<(), GameError> {
    let _args = Args::parse();

    Game::new().run()
}
