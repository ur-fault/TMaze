mod game;
use game::Game;

mod maze;

fn main() -> Result<(), game::game::Error> {
    Game::new().run()
}
