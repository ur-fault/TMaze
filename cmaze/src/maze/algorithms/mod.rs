mod depth_first_search;
mod rnd_kruskals;

use super::{Maze, Cell, CellWall};
pub use crate::core::*;
use crossbeam::{channel::{Receiver, Sender, unbounded}, scope};
pub use depth_first_search::DepthFirstSearch;
use rand::{thread_rng, Rng};
pub use rnd_kruskals::RndKruskals;
use std::{
    any::Any,
    sync::{Arc, RwLock},
    thread::{JoinHandle, self},
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
        use_rayon: bool,
    ) -> Result<MazeGeneratorComunication, GenerationErrorInstant> {
        if size.0 <= 0 || size.1 <= 0 || size.2 <= 0 {
            return Err(GenerationErrorInstant::InvalidSize(size));
        }

        let stop_flag = StopGenerationFlag::new();
        let (s_progress, r_progress) = unbounded::<(usize, usize)>();

        let stop_flag_clone = stop_flag.clone();

        Ok((
            thread::spawn(move || {
                let Dims3D(w, h, d) = size;
                let (wu, hu, du) = (w as usize, h as usize, d as usize);

                let cells = if floored && d > 1 {
                    let mut cells: Vec<Vec<Vec<Cell>>> = (0..du)
                        .map(|maze_i| {
                            let (s, r) = unbounded::<(usize, usize)>();

                            let s_progress = s_progress.clone();
                            let stop_flag = stop_flag.clone();
                            match scope(|scope| {
                                scope.spawn(move |_| {
                                    for (done, from) in r.iter() {
                                        s_progress.send((done + maze_i * from, from * du)).unwrap();
                                    }
                                });

                                if stop_flag.is_stopped() {
                                    return Err(GenerationErrorThreaded::AbortGeneration);
                                }

                                Self::generate_individual(Dims3D(w, h, 1), stop_flag, s, use_rayon)
                            })
                            .map(
                                |res| -> Result<Vec<Vec<Cell>>, GenerationErrorThreaded> {
                                    Ok(res?.cells.remove(0))
                                },
                            ) {
                                Ok(Ok(maze)) => Ok(maze),
                                Err(e) => Err(GenerationErrorThreaded::UnknownError(e)),
                                Ok(Err(e)) => Err(e),
                            }
                        })
                        .collect::<Result<Vec<_>, GenerationErrorThreaded>>()?;

                    for floor in 0..du - 1 {
                        let (x, y) = (thread_rng().gen_range(0..wu), thread_rng().gen_range(0..hu));
                        cells[floor][y][x].remove_wall(CellWall::Up);
                        cells[floor + 1][y][x].remove_wall(CellWall::Down);
                    }

                    cells
                } else {
                    Self::generate_individual(Dims3D(w, h, d), stop_flag, s_progress.clone(), use_rayon)?.cells
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

    fn generate_individual(
        size: Dims3D,
        stopper: StopGenerationFlag,
        progress: Sender<(usize, usize)>,
        use_rayon: bool,
    ) -> Result<Maze, GenerationErrorThreaded>;
}
