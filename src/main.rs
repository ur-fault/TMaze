mod game;
use game::Game;
mod maze;
mod tmcore;
mod ui;
mod helpers;

fn main() -> Result<(), tmcore::Error> {
    Game::new().run()
}
