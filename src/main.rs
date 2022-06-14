mod game;

use game::Game;
mod helpers;
mod maze;
mod settings;
mod core;
mod ui;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version)]
struct Args {

}

fn main() -> Result<(), core::Error> {
    let _args = Args::parse();

    Game::new().run()
}
