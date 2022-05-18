mod game;

use game::Game;
mod helpers;
mod maze;
mod settings;
mod tmcore;
mod ui;

fn main() -> Result<(), tmcore::Error> {
    Game::new().run()
}
