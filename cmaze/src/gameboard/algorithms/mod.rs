mod depth_first_search;
mod rnd_kruskals;

use rand::{thread_rng, Rng};
use rayon::prelude::*;

use std::{
    ops,
    sync::{Arc, Mutex, RwLock},
    thread,
};

use super::{Cell, CellWall, Maze};

use crate::{array::Array3D, dims::*, game::ProgressComm};
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

impl Default for StopGenerationFlag {
    fn default() -> Self {
        Self::new()
    }
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

#[derive(Clone, Debug)]
pub struct Flag(Arc<RwLock<bool>>);

impl Flag {
    pub fn new() -> Self {
        Flag(Arc::new(RwLock::new(false)))
    }

    pub fn stop(&self) {
        *self.0.write().unwrap() = true;
    }

    pub fn is_stopped(&self) -> bool {
        *self.0.read().unwrap()
    }
}

impl Default for Flag {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Progress {
    pub done: usize,
    pub from: usize,
    is_done: bool,
}

#[derive(Debug, Clone)]
pub struct CellMask {
    // TODO: Use bitset
    buf: Vec<bool>,
    width: i32,
    height: i32,
    depth: i32,
}

impl CellMask {
    pub fn new(width: usize, height: usize, depth: usize) -> Self {
        Self {
            buf: vec![true; width * height * depth],
            width: width as i32,
            height: height as i32,
            depth: depth as i32,
        }
    }

    pub fn new_dims(size: Dims3D) -> Self {
        Self::new(size.0 as usize, size.1 as usize, size.2 as usize)
    }

    pub fn new_2d(width: usize, height: usize) -> Self {
        Self::new(width, height, 1)
    }

    pub fn size(&self) -> Dims3D {
        Dims3D(self.width, self.height, self.depth)
    }

    pub fn is_empty(&self) -> bool {
        self.buf.iter().all(|&b| !b)
    }

    pub fn is_full(&self) -> bool {
        self.buf.iter().all(|&b| b)
    }

    pub fn dim_to_idx(&self, pos: Dims3D) -> Option<usize> {
        let Dims3D(x, y, z) = pos;

        if (x < 0 || x >= self.width) || (y < 0 || y >= self.height) || (z < 0 || z >= self.depth) {
            None
        } else {
            Some((z * self.width * self.height + y * self.width + x) as usize)
        }
    }

    pub fn is_connected(&self) -> bool {
        let mut mask = self.clone();

        fn dfs(mask: &mut CellMask, pos: Dims3D) {
            if (pos.0 < 0 || pos.0 >= mask.width)
                || (pos.1 < 0 || pos.1 >= mask.height)
                || (pos.2 < 0 || pos.2 >= mask.depth)
            {
                return;
            }

            if mask[pos] {
                mask[pos] = false;

                for dir in CellWall::get_in_order() {
                    dfs(mask, pos + dir.to_coord());
                }
            }
        }

        if mask.is_empty() {
            return false;
        }

        for pos in Dims3D::ZERO.iter_fill(self.size()) {
            if mask[pos] {
                dfs(&mut mask, pos);
                break;
            }
        }

        mask.is_empty()
    }
}

impl ops::Index<Dims3D> for CellMask {
    type Output = bool;

    /// Returns the value at the given index, or `false` if the index is out of bounds.
    fn index(&self, index: Dims3D) -> &Self::Output {
        self.dim_to_idx(index)
            .and_then(|i| self.buf.get(i))
            .unwrap_or(&false)
    }
}

impl ops::IndexMut<Dims3D> for CellMask {
    fn index_mut(&mut self, index: Dims3D) -> &mut Self::Output {
        self.dim_to_idx(index)
            .and_then(|i| self.buf.get_mut(i))
            .expect("Index out of bounds")
    }
}

pub struct Generator {
    pub generator: Box<dyn GroupGenerator>,
}

impl Generator {
    pub fn new(generator: Box<dyn GroupGenerator>) -> Self {
        Self { generator }
    }

    // TODO: Custom error type
    pub fn generate(&self, size: Dims3D) -> Result<Maze, ()> {
        if size.0 <= 0 || size.1 <= 0 || size.2 <= 0 {
            return Err(());
        }

        Ok(self.generator.generate(CellMask::new_dims(size)))
    }
}

pub trait GroupGenerator {
    fn generate(&self, mask: CellMask) -> Maze;
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
            is_done: false,
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
                        let (x, y) = (thread_rng().gen_range(0..w), thread_rng().gen_range(0..h));
                        cells[Dims3D(x, y, floor as i32)].remove_wall(CellWall::Up);
                        cells[Dims3D(x, y, floor as i32 + 1)].remove_wall(CellWall::Down);
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
    ) -> Result<Array3D<Cell>, GenErrorThreaded> {
        let Dims3D(w, h, d) = size;
        let (.., du) = (w as usize, h as usize, d as usize);
        let generate_floor = |progress| {
            let stop_flag = stop_flag.clone();

            Self::generate_individual(Dims3D(w, h, 1), stop_flag, progress)
        };

        let stop_flag = stop_flag.clone();

        thread::scope(|s| {
            let mut local_progresses = (0..du)
                .map(|_| Progress {
                    done: 0,
                    from: 1,
                    is_done: false,
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

                let all_done = local_progresses.iter().all(|p| p.is_done);
                let mut progress = progress.lock().unwrap();
                progress.is_done = all_done;
                progress.done = local_progresses.iter().map(|p| p.done).sum();
                progress.from = local_progresses.iter().map(|p| p.from).sum();

                if all_done || stop_flag.is_stopped() {
                    break;
                }
            });

            let mut cells = Array3D::new(Cell::new(), w as usize, h as usize, du);

            let x: Vec<_> = (0..du)
                .into_par_iter()
                .map(|i| generate_floor(Arc::clone(&shared_progresses[i])))
                .collect::<Result<_, _>>()?;

            for (i, floor) in x.into_iter().enumerate() {
                for Dims3D(x, y, _) in floor.cells.iter_pos() {
                    cells[Dims3D(x, y, i as i32)] = floor.cells[Dims3D(x, y, 0)].clone();
                }
            }

            Ok(cells)
        })
    }

    fn generate_individual(
        size: Dims3D,
        stopper: StopGenerationFlag,
        progress: Arc<Mutex<Progress>>,
    ) -> Result<Maze, GenErrorThreaded>;
}
