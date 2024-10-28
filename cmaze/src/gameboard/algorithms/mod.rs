mod depth_first_search;
mod rnd_kruskals;

use hashbrown::{HashMap, HashSet};
use rand::{seq::SliceRandom as _, thread_rng, Rng as _, SeedableRng as _};
use rayon::prelude::*;
use smallvec::SmallVec;

use std::{
    fmt, ops,
    sync::{Arc, Mutex, MutexGuard, RwLock},
    thread,
};

use super::{Cell, CellWall, Maze};

use crate::{array::Array3D, dims::*, game::ProgressComm};
pub use depth_first_search::DepthFirstSearch;
pub use rnd_kruskals::RndKruskals;

/// Random number generator used for anything, where determinism is required.
pub type Random = rand_xoshiro::Xoshiro256StarStar;

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

#[derive(Clone)]
pub struct ProgressHandle {
    progress: Arc<Mutex<Progress>>,
    handler: ProgressHandler,
}

impl ProgressHandle {
    pub fn new(handler: ProgressHandler) -> Self {
        Self {
            progress: Arc::new(Mutex::new(Progress::new_empty())),
            handler,
        }
    }

    pub fn split(&self) -> Self {
        self.handler.add()
    }

    pub fn lock(&self) -> MutexGuard<Progress> {
        self.progress.lock().unwrap()
    }
}

#[derive(Clone)]
pub struct ProgressHandler {
    jobs: Arc<Mutex<Vec<ProgressHandle>>>,
}

impl ProgressHandler {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add(&self) -> ProgressHandle {
        let progress = ProgressHandle::new(self.clone());
        self.jobs.lock().unwrap().push(progress.clone());
        progress
    }

    pub fn progress(&self) -> Progress {
        self.jobs.lock().unwrap().iter().fold(
            Progress {
                done: 0,
                from: 0,
                is_done: true,
            },
            |prog, job| prog.combine(&job.progress.lock().unwrap()),
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Progress {
    pub done: usize,
    pub from: usize,
    pub is_done: bool,
}

impl Progress {
    pub fn new(done: usize, from: usize) -> Self {
        Self {
            done,
            from,
            is_done: false,
        }
    }

    pub fn new_empty() -> Self {
        Self::new(0, 1)
    }

    pub fn percent(&self) -> f32 {
        self.done as f32 / self.from as f32
    }

    pub fn finish(&mut self) {
        self.done = self.from;
        self.is_done = true;
    }

    pub fn combine(&self, other: &Self) -> Self {
        Self {
            done: self.done + other.done,
            from: self.from + other.from,
            is_done: self.is_done && other.is_done,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CellMask(Array3D<bool>);

impl CellMask {
    pub fn new(width: usize, height: usize, depth: usize) -> Self {
        Self(Array3D::new(true, width, height, depth))
    }

    pub fn new_dims(size: Dims3D) -> Option<Self> {
        Some(Self(Array3D::new_dims(true, size)?))
    }

    pub fn new_dims_empty(size: Dims3D) -> Option<Self> {
        Some(Self(Array3D::new_dims(false, size)?))
    }

    pub fn new_2d(width: usize, height: usize) -> Self {
        Self::new(width, height, 1)
    }

    pub fn size(&self) -> Dims3D {
        self.0.size()
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|&b| !b)
    }

    pub fn is_full(&self) -> bool {
        self.0.iter().all(|&b| b)
    }

    pub fn enabled_count(&self) -> usize {
        self.0.iter().filter(|&&b| b).count()
    }

    pub fn random_cell(&self, rng: &mut Random) -> Option<Dims3D> {
        let enabled = self
            .0
            .iter_pos()
            .filter(|&pos| self[pos])
            .collect::<Vec<_>>();
        enabled.choose(rng).copied()
    }

    pub fn is_connected(&self) -> bool {
        let mut mask = self.clone();

        fn dfs(mask: &mut CellMask, pos: Dims3D) {
            let Dims3D(width, height, depth) = mask.size();

            if (pos.0 < 0 || pos.0 >= width)
                || (pos.1 < 0 || pos.1 >= height)
                || (pos.2 < 0 || pos.2 >= depth)
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

        for pos in Dims3D::iter_fill(Dims3D::ZERO, self.size()) {
            if mask[pos] {
                dfs(&mut mask, pos);
                break;
            }
        }

        mask.is_empty()
    }

    pub fn to_array3d(self) -> Array3D<bool> {
        self.0
    }

    pub fn fill(&mut self, value: bool) {
        self.0.fill(value);
    }
}

impl ops::Index<Dims3D> for CellMask {
    type Output = bool;

    /// Returns the value at the given index, or `false` if the index is out of bounds.
    fn index(&self, index: Dims3D) -> &Self::Output {
        self.0.get(index).unwrap_or(&false)
    }
}

impl ops::IndexMut<Dims3D> for CellMask {
    fn index_mut(&mut self, index: Dims3D) -> &mut Self::Output {
        self.0
            .get_mut(index)
            .unwrap_or_else(|| panic!("Index out of bounds: {:?}", index))
    }
}

#[derive(Debug)]
pub struct GeneratorError;

#[derive(Debug, Clone)]
pub struct Generator {
    generator: Arc<dyn GroupGenerator>,
}

impl Generator {
    pub fn new(generator: Box<dyn GroupGenerator>) -> Self {
        Self {
            generator: generator.into(),
        }
    }

    // TODO: Custom error type
    pub fn generate(
        &self,
        size: Dims3D,
        seed: Option<u64>,
        progress: ProgressHandle,
    ) -> Result<Maze, GeneratorError> {
        if size.0 <= 0 || size.1 <= 0 || size.2 <= 0 {
            return Err(GeneratorError);
        }

        let mut rng = Random::seed_from_u64(seed.unwrap_or_else(|| thread_rng().gen()));

        progress.lock().done = 0;

        const SPLIT_COUNT: i32 = 100;
        let group_count = (size.product() / SPLIT_COUNT).clamp(1, u8::MAX as i32) as u8;
        let points = Self::random_points(size, group_count, &mut rng);
        let groups = Self::split_groups(points, size, &mut rng);
        let masks = Self::split_to_masks(group_count, &groups);

        let regions: Vec<_> = masks
            .into_iter()
            .map(|mask| self.generator.generate(mask, &mut rng, progress.split()))
            .collect();

        let connect_regions = Self::connect_regions(groups, regions, &mut rng);
        progress.lock().finish();

        Ok(connect_regions)
    }

    pub fn random_points(size: Dims3D, count: u8, rng: &mut Random) -> Vec<Dims3D> {
        assert!(size.all_positive());
        assert!(count as i32 <= size.product() && count > 0);

        let count = count as usize;
        let mut points = Vec::with_capacity(count);

        rng.gen_range(0..size.0);

        while points.len() < count {
            let point = Dims3D(
                rng.gen_range(0..size.0),
                rng.gen_range(0..size.1),
                rng.gen_range(0..size.2),
            );

            if !points.contains(&point) {
                points.push(point);
            }
        }

        points
    }

    // Split an maze into sensible sized groups,
    pub fn split_groups(points: Vec<Dims3D>, size: Dims3D, rng: &mut Random) -> Array3D<u8> {
        assert!(points.len() <= u8::MAX as usize);
        assert!(!points.is_empty());
        assert!(points.clone().into_iter().collect::<HashSet<_>>().len() == points.len());

        let mut groups = Array3D::new_dims(None, size).unwrap();

        // assign initial groups
        for (i, point) in points.into_iter().enumerate() {
            groups[point] = Some((i as u8, usize::MAX));
        }

        // This algorithm uses simple flood with diamond shaped search and randomized group order
        // on each cycle.
        // If it's found that it generates boring results, it can be replaced with a more complex
        // one.

        let mut cycle = 0usize;

        loop {
            if groups.all(|group| group.is_some()) {
                break;
            }

            for cell in Dims3D::iter_fill(Dims3D::ZERO, size) {
                if groups[cell].is_some() {
                    continue;
                }

                let neighbors = CellWall::get_in_order()
                    .into_iter()
                    .map(|dir| cell + dir.to_coord())
                    .filter_map(|pos| {
                        groups.get(pos).and_then(|g| {
                            g.and_then(|(g, cyc)| if cyc == cycle { None } else { Some(g) })
                        })
                    })
                    .collect::<SmallVec<[_; 6]>>();

                if let Some(new_group) = neighbors.choose(rng) {
                    groups[cell] = Some((*new_group, cycle));
                }
            }

            cycle = cycle.wrapping_add(1);
        }

        groups.map(|group| group.unwrap().0)
    }

    // Split groups into masks, ready for maze generation
    pub fn split_to_masks(group_count: u8, groups: &Array3D<u8>) -> Vec<CellMask> {
        let mut masks =
            vec![CellMask::new_dims_empty(groups.size()).unwrap(); group_count as usize];

        for (cell, &group) in groups.iter_pos().zip(groups.iter()) {
            masks[group as usize][cell] = true;
        }

        masks
    }

    pub fn connect_regions(groups: Array3D<u8>, regions: Vec<Maze>, rng: &mut Random) -> Maze {
        // Disclaimer: this implementation can be slow af, since there is a maximum of a 256 groups
        // We use a simple Kruskal's algorithm to connect the regions

        let mut walls = HashMap::new();
        for ((from_g, to_g), (from, dir)) in Self::build_region_graph(&groups) {
            assert!(from_g < to_g);
            walls
                .entry((from_g, to_g))
                .or_insert_with(Vec::new)
                .push((from, dir));
        }

        // Choose only one wall from all of the options
        let mut walls: Vec<_> = walls
            .into_iter()
            .map(|(k, v)| (k, *v.choose(rng).unwrap()))
            .collect();
        walls.shuffle(rng);

        let mut sets: Vec<HashSet<u8>> = (0..regions.len() as u8)
            .map(|i| Some(i).into_iter().collect())
            .collect();

        // Combine the regions, so we can start connecting them
        let mut maze = Maze {
            cells: Array3D::new_dims(Cell::new(), groups.size()).unwrap(),
            is_tower: false,
        };
        for cell in groups.iter_pos() {
            let group = groups[cell];
            let region = &regions[group as usize];
            maze.cells[cell] = region.cells[cell];
        }

        #[allow(unused_variables)]
        while let Some(((from_g, to_g), (from, dir))) = walls.pop() {
            let from_set = sets
                .iter()
                .enumerate()
                .find(|(_, set)| set.contains(&from_g))
                .unwrap();
            if from_set.1.contains(&to_g) {
                continue;
            }
            maze.remove_wall(from, dir);

            let from_set = sets.swap_remove(from_set.0);
            let to_set = sets.iter_mut().find(|set| set.contains(&to_g)).unwrap();
            to_set.extend(from_set);
        }

        maze
    }

    pub fn build_region_graph(groups: &Array3D<u8>) -> Vec<((u8, u8), (Dims3D, CellWall))> {
        let mut borders = vec![];

        for cell in groups.iter_pos() {
            let group = groups[cell];

            use CellWall::*;
            for dir in [Right, Bottom, Down] {
                let neighbor = cell + dir.to_coord();

                if let Some(&neighbor_group) = groups.get(neighbor) {
                    if neighbor_group != group {
                        if group < neighbor_group {
                            borders.push(((group, neighbor_group), (cell, dir)));
                        } else {
                            borders.push(((neighbor_group, group), (cell, dir)));
                        }
                    }
                }
            }
        }

        borders
    }
}

pub trait GroupGenerator: fmt::Debug + Sync + Send {
    fn generate(&self, mask: CellMask, rng: &mut Random, progress: ProgressHandle) -> Maze;

    fn guess_progress_complexity(&self, mask: &CellMask) -> usize {
        mask.enabled_count()
    }
}

pub trait MazeAlgorithm {
    fn generate(
        size: Dims3D,
        floored: bool,
    ) -> Result<ProgressComm<Result<Maze, GenErrorThreaded>>, GenErrorInstant> {
        todo!()
        // if size.0 <= 0 || size.1 <= 0 || size.2 <= 0 {
        //     return Err(GenErrorInstant::InvalidSize(size));
        // }
        //
        // let stop_flag = Flag::new();
        // let progress = Arc::new(Mutex::new(Progress {
        //     done: 0,
        //     from: 1,
        //     is_done: false,
        // }));
        // let recv = Arc::clone(&progress);
        //
        // let stop_flag_clone = stop_flag.clone();
        //
        // Ok(ProgressComm {
        //     handle: thread::spawn(move || {
        //         let Dims3D(w, h, d) = size;
        //         let du = d as usize;
        //
        //         let cells = if floored && d > 1 {
        //             let mut cells = Self::generate_floors(size, progress, stop_flag)?;
        //
        //             for floor in 0..du - 1 {
        //                 let (x, y) = (thread_rng().gen_range(0..w), thread_rng().gen_range(0..h));
        //                 cells[Dims3D(x, y, floor as i32)].remove_wall(CellWall::Up);
        //                 cells[Dims3D(x, y, floor as i32 + 1)].remove_wall(CellWall::Down);
        //             }
        //
        //             cells
        //         } else {
        //             Self::generate_individual(Dims3D(w, h, d), stop_flag, progress)?.cells
        //         };
        //
        //         Ok(Maze {
        //             cells,
        //             is_tower: floored,
        //         })
        //     }),
        //     stop_flag: stop_flag_clone,
        //     recv,
        // })
    }

    fn generate_floors(
        size: Dims3D,
        progress: Arc<Mutex<Progress>>,
        stop_flag: Flag,
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
                    cells[Dims3D(x, y, i as i32)] = floor.cells[Dims3D(x, y, 0)];
                }
            }

            Ok(cells)
        })
    }

    fn generate_individual(
        size: Dims3D,
        stopper: Flag,
        progress: Arc<Mutex<Progress>>,
    ) -> Result<Maze, GenErrorThreaded>;
}
