mod game;
use game::Game;
mod maze;
pub mod tmcore;

fn main() -> Result<(), tmcore::Error> {
    Game::new().run()
}
