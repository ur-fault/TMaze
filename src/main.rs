mod game;

use game::Game;
mod helpers;
mod maze;
mod settings;
mod core;
mod ui;

fn main() -> Result<(), core::Error> {
    Game::new().run()
}
