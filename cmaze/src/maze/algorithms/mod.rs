mod depth_first_search;
mod rnd_kruskals;

use super::{Cell, CellWall, Maze};
pub use crate::core::*;
use crossbeam::{
    channel::{unbounded, Receiver, Sender},
    scope,
};
pub use depth_first_search::DepthFirstSearch;
use rand::{thread_rng, Rng};
use rayon;
pub use rnd_kruskals::RndKruskals;
use std::{
    any::Any,
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
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

impl Default for StopGenerationFlag {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Progress {
    pub done: usize,
    pub from: usize,
}

pub type MazeGeneratorComunication = (
    JoinHandle<Result<Maze, GenerationErrorThreaded>>,
    StopGenerationFlag,
    Receiver<Progress>,
);

pub trait MazeAlgorithm {
    fn generate(
        size: Dims3D,
        floored: bool,
        multithreaded_floored: bool,
    ) -> Result<MazeGeneratorComunication, GenerationErrorInstant> {
        if size.0 <= 0 || size.1 <= 0 || size.2 <= 0 {
            return Err(GenerationErrorInstant::InvalidSize(size));
        }

        let stop_flag = StopGenerationFlag::new();
        let (s_progress, r_progress) = unbounded::<Progress>();

        let stop_flag_clone = stop_flag.clone();

        Ok((
            thread::spawn(move || {
                let Dims3D(w, h, d) = size;
                let (wu, hu, du) = (w as usize, h as usize, d as usize);

                let cells = if floored && d > 1 {
                    let mut cells = Self::generate_floors(size, s_progress, stop_flag)?;

                    for floor in 0..du - 1 {
                        let (x, y) = (thread_rng().gen_range(0..wu), thread_rng().gen_range(0..hu));
                        cells[floor][y][x].remove_wall(CellWall::Up);
                        cells[floor + 1][y][x].remove_wall(CellWall::Down);
                    }

                    cells
                } else {
                    Self::generate_individual(Dims3D(w, h, d), stop_flag, s_progress.clone())?.cells
                };

                Ok(Maze {
                    cells,
                    width: wu,
                    height: hu,
                    depth: du,
                })
            }),
            stop_flag_clone,
            r_progress,
        ))
    }

    fn generate_floors(
        size: Dims3D,
        progres_sender: Sender<Progress>,
        stop_flag: StopGenerationFlag,
    ) -> Result<Vec<Vec<Vec<Cell>>>, GenerationErrorThreaded> {
        let Dims3D(w, h, d) = size;
        let (.., du) = (w as usize, h as usize, d as usize);
        let s_progress = progres_sender;

        let cells: Vec<Vec<Vec<Cell>>> = (0..du)
            .map(|maze_i| {
                let (s, r) = unbounded();

                let s_progress = s_progress.clone();
                let stop_flag = stop_flag.clone();
                match scope(|scope| {
                    scope.spawn(move |_| {
                        for Progress { done, from } in r.iter() {
                            s_progress
                                .send(Progress {
                                    done: done + maze_i * from,
                                    from: from * du,
                                })
                                .unwrap();
                        }
                    });

                    if stop_flag.is_stopped() {
                        return Err(GenerationErrorThreaded::AbortGeneration);
                    }

                    Self::generate_individual(Dims3D(w, h, 1), stop_flag, s)
                })
                .map(|res| Ok(res?.cells.remove(0)))
                {
                    Ok(Ok(maze)) => Ok(maze),
                    Err(e) => Err(GenerationErrorThreaded::UnknownError(e)),
                    Ok(Err(e)) => Err(e),
                }
            })
            .collect::<Result<Vec<_>, GenerationErrorThreaded>>()?;

        Ok(cells)
    }

    fn generate_individual(
        size: Dims3D,
        stopper: StopGenerationFlag,
        progress: Sender<Progress>,
    ) -> Result<Maze, GenerationErrorThreaded>;
}
