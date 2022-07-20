mod depth_first_search;
mod rnd_kruskals;

use super::Maze;
pub use crate::core::*;
use crossbeam::channel::{Receiver, Sender};
pub use depth_first_search::DepthFirstSearch;
pub use rnd_kruskals::RndKruskals;
use std::{
    any::Any,
    sync::{Arc, RwLock},
    thread::JoinHandle,
};

#[derive(Debug)]
pub enum GenerationErrorInstant {
    InvalidSize(Dims3D),
}

#[derive(Debug)]
pub enum GenerationErrorThreaded {
    GenerationError(GenerationErrorInstant),
    AbortGeneration,
    UnknownError(Box<dyn Send + Any + 'static>),
}

#[derive(Debug)]
pub struct StopGenerationError;

#[derive(Clone, Debug)]
pub struct StopGenerationFlag {
    stop: Arc<RwLock<bool>>,
}

impl StopGenerationFlag {
    pub fn new() -> Self {
        StopGenerationFlag {
            stop: Arc::new(RwLock::new(false)),
        }
    }

    pub fn stop(&self) -> bool {
        *self.stop.write().unwrap() = true;
        self.is_stopped()
    }

    pub fn is_stopped(&self) -> bool {
        *self.stop.read().unwrap()
    }
}

pub type MazeGeneratorComunication = (
    JoinHandle<Result<Maze, GenerationErrorThreaded>>,
    StopGenerationFlag,
    Receiver<(usize, usize)>,
);

pub trait MazeAlgorithm {
    fn generate(
        size: Dims3D,
        floored: bool,
    ) -> Result<MazeGeneratorComunication, GenerationErrorInstant>;
    fn generate_individual(
        size: Dims3D,
        stopper: StopGenerationFlag,
        progress: Sender<(usize, usize)>,
    ) -> Result<Maze, GenerationErrorThreaded>;
}
