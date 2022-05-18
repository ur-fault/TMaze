use super::Maze;
mod depth_first_search;
pub use depth_first_search::DepthFirstSearch;
mod rnd_kruskals;
pub use crate::tmcore::*;
pub use rnd_kruskals::RndKruskals;

pub trait MazeAlgorithm {
    fn generate<T: FnMut(usize, usize) -> Result<(), Error>>(
        size: Dims3D,
        floored: bool,
        report_progress: Option<T>,
    ) -> Result<Maze, Error>;

    fn generate_individual<T: FnMut(usize, usize) -> Result<(), Error>>(
        size: Dims3D,
        report_progress: Option<T>,
    ) -> Result<Maze, Error>;
}
