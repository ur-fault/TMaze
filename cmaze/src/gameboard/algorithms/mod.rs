mod depth_first_search;
mod rnd_kruskals;

use rand::{thread_rng, Rng};
use rayon::prelude::*;

use std::{
    sync::{Arc, Mutex, RwLock},
    thread,
};

use super::{Cell, CellWall, Maze};

use crate::{dims::*, game::ProgressComm};
pub use depth_first_search::DepthFirstSearch;
pub use rnd_kruskals::RndKruskals;

#[derive(Debug)]
pub enum GenErrorInstant {
    InvalidSize(Dims3D),
}

#[derive(Debug)]
pub enum GenErrorThreaded {
    GenerationError(GenErrorInstant),
    AbortGeneration,
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

#[derive(Debug, Clone, Copy)]
pub struct Progress {
    pub done: usize,
    pub from: usize,
    is_finished: bool,
}

pub trait MazeAlgorithm {
    fn generate(
        size: Dims3D,
        floored: bool,
    ) -> Result<ProgressComm<Result<Maze, GenErrorThreaded>>, GenErrorInstant> {
        if size.0 <= 0 || size.1 <= 0 || size.2 <= 0 {
            return Err(GenErrorInstant::InvalidSize(size));
        }

        let stop_flag = StopGenerationFlag::new();
        let progress = Arc::new(Mutex::new(Progress {
            done: 0,
            from: 1,
            is_finished: false,
        }));
        let recv = Arc::clone(&progress);

        let stop_flag_clone = stop_flag.clone();

        Ok(ProgressComm {
            handle: thread::spawn(move || {
                let Dims3D(w, h, d) = size;
                let (wu, hu, du) = (w as usize, h as usize, d as usize);

                let cells = if floored && d > 1 {
                    let mut cells = Self::generate_floors(size, progress, stop_flag)?;

                    for floor in 0..du - 1 {
                        let (x, y) = (thread_rng().gen_range(0..wu), thread_rng().gen_range(0..hu));
                        cells[floor][y][x].remove_wall(CellWall::Up);
                        cells[floor + 1][y][x].remove_wall(CellWall::Down);
                    }

                    cells
                } else {
                    Self::generate_individual(Dims3D(w, h, d), stop_flag, progress)?.cells
                };

                Ok(Maze {
                    cells,
                    width: wu,
                    height: hu,
                    depth: du,
                    is_tower: floored,
                })
            }),
            stop_flag: stop_flag_clone,
            recv,
        })
    }

    fn generate_floors(
        size: Dims3D,
        progress: Arc<Mutex<Progress>>,
        stop_flag: StopGenerationFlag,
    ) -> Result<Vec<Vec<Vec<Cell>>>, GenErrorThreaded> {
        let Dims3D(w, h, d) = size;
        let (.., du) = (w as usize, h as usize, d as usize);
        let generate_floor = |progress| {
            let stop_flag = stop_flag.clone();

            let generation_result = Self::generate_individual(Dims3D(w, h, 1), stop_flag, progress);

            generation_result.map(|mut res| res.cells.remove(0))
        };

        let stop_flag = stop_flag.clone();

        thread::scope(|s| {
            let mut local_progresses = (0..du)
                .map(|_| Progress {
                    done: 0,
                    from: 1,
                    is_finished: false,
                })
                .collect::<Vec<_>>();
            let shared_progresses = local_progresses
                .iter()
                .map(|p| Arc::new(Mutex::new(*p)))
                .collect::<Vec<_>>();

            let shared2 = shared_progresses.clone();

            s.spawn(move || loop {
                for (i, progress) in shared2.iter().enumerate() {
                    let p = *progress.lock().unwrap();
                    local_progresses[i] = p;
                }

                let all_done = local_progresses.iter().all(|p| p.is_finished);
                let mut progress = progress.lock().unwrap();
                progress.is_finished = all_done;
                progress.done = local_progresses.iter().map(|p| p.done).sum();
                progress.from = local_progresses.iter().map(|p| p.from).sum();

                if all_done || stop_flag.is_stopped() {
                    break;
                }
            });

            (0..du)
                .into_par_iter()
                .map(|i| shared_progresses[i].clone())
                .map(generate_floor)
                .collect::<Result<Vec<_>, GenErrorThreaded>>()
        })
    }

    fn generate_individual(
        size: Dims3D,
        stopper: StopGenerationFlag,
        progress: Arc<Mutex<Progress>>,
    ) -> Result<Maze, GenErrorThreaded>;
}
