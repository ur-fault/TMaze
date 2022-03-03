use super::{Maze};
use crate::game::Error;
mod depth_first_search;
pub use depth_first_search::DepthFirstSearch;
mod rnd_kruskals;
pub use rnd_kruskals::RndKruskals;
pub use crate::game::Dims3D;

pub trait MazeAlgorithm {
    fn new<T: FnMut(usize, usize) -> Result<(), Error>>(
        size: Dims3D,
        report_progress: Option<T>,
    ) -> Result<Maze, Error>;
}
