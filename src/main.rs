#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

mod game;
use game::Game;
use crate::maze::MazeAlgorithm;

mod maze;

fn main() -> Result<(), game::game::Error> {
    Game::new().run()
}
