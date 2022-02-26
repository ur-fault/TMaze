use super::Maze;
use crate::game::Error;
mod depth_first_search;
pub use depth_first_search::DepthFirstSearch;
mod rnd_kruskals;
pub use rnd_kruskals::RndKruskals;

pub trait MazeAlgorithm {
    fn new<T: FnMut(usize, usize) -> Result<(), Error>>(
        w: usize,
        h: usize,
        start_: Option<(usize, usize)>,
        report_progress: Option<T>,
    ) -> Result<Maze, Error>;
}
