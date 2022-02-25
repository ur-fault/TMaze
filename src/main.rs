#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

mod game;
use game::Game;
mod maze;
// use maze::{Maze, CellWall, Cell};

fn main() -> Result<(), game::game::Error> {
    Game::new().run()
}
