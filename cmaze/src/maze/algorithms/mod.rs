use super::Maze;
mod depth_first_search;
pub use depth_first_search::DepthFirstSearch;
mod rnd_kruskals;
pub use crate::core::*;
pub use rnd_kruskals::RndKruskals;
use std::fmt;

#[derive(Debug)]
pub enum GenerationError<R, A> {
    InvalidSize(Dims3D),
    ReportFunctionError(ReportCallbackError<R, A>),
}

impl<R, A> From<ReportCallbackError<R, A>> for GenerationError<R, A>
where
    R: fmt::Debug,
    A: fmt::Debug,
{
    fn from(e: ReportCallbackError<R, A>) -> Self {
        GenerationError::ReportFunctionError(e)
    }
}

#[derive(Debug)]
pub enum ReportCallbackError<R, A> {
    ReportFunctionError(R),
    AbortGeneration(A),
}

pub trait MazeAlgorithm<R, A> {
    fn generate<T: FnMut(usize, usize) -> Result<(), ReportCallbackError<R, A>>>(
        size: Dims3D,
        floored: bool,
        report_progress: Option<T>,
    ) -> Result<Maze, GenerationError<R, A>>;

    fn generate_individual<T: FnMut(usize, usize) -> Result<(), ReportCallbackError<R, A>>>(
        size: Dims3D,
        report_progress: Option<T>,
    ) -> Result<Maze, GenerationError<R, A>>;
}
