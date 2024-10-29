mod depth_first_search;
mod rnd_kruskals;

use hashbrown::{HashMap, HashSet};
use rand::{seq::SliceRandom as _, thread_rng, Rng as _, SeedableRng as _};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use smallvec::SmallVec;

use std::{fmt, ops, sync::Arc};

use super::{Cell, CellWall, Maze};

use crate::{array::Array3D, dims::*, progress::ProgressHandle};
pub use depth_first_search::DepthFirstSearch;
pub use rnd_kruskals::RndKruskals;

/// Random number generator used for anything, where determinism is required.
pub type Random = rand_xoshiro::Xoshiro256StarStar;

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
        // If less then 10% of the cells are enabled, we can collect all of them and choose one,
        // otherwise we can just choose random cell and check that it's enabled.

        let enabled = self.enabled_count();
        if enabled < self.0.len() / 10 {
            let enabled = self
                .0
                .iter_pos()
                .filter(|&pos| self[pos])
                .collect::<Vec<_>>();
            enabled.choose(rng).copied()
        } else if enabled > 0 {
            loop {
                let size = self.size();
                let pos = Dims3D(
                    rng.gen_range(0..size.0),
                    rng.gen_range(0..size.1),
                    rng.gen_range(0..size.2),
                );

                if self[pos] {
                    return Some(pos);
                }
            }
        } else {
            None
        }
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
    default_generator: Arc<dyn RegionGenerator>,
    splitter: Arc<dyn RegionSplitter>,
}

impl Generator {
    pub fn new(generator: Box<dyn RegionGenerator>, splitter: Box<dyn RegionSplitter>) -> Self {
        Self {
            default_generator: generator.into(),
            splitter: splitter.into(),
        }
    }

    // TODO: Custom error type
    pub fn generate(
        &self,
        mask: CellMask,
        seed: Option<u64>,
        progress: ProgressHandle,
    ) -> Result<Maze, GeneratorError> {
        let mut rng = Random::seed_from_u64(seed.unwrap_or_else(|| thread_rng().gen()));

        let maze_size = self.default_generator.guess_progress_complexity(&mask);
        progress.lock().from = maze_size; // initial work estimate

        const SPLIT_COUNT: usize = 100;
        let group_count = (mask.enabled_count() / SPLIT_COUNT).clamp(1, u8::MAX as usize) as u8;
        let groups = self
            .splitter
            .split(&mask, &mut rng, progress.split())
            .ok_or(GeneratorError)?;
        let masks = Self::split_to_masks(group_count, &groups);

        if progress.is_stopped() {
            return Err(GeneratorError);
        }

        let progresses = masks
            .iter()
            .map(|mask| {
                let local = progress.split();
                local.lock().from = self.default_generator.guess_progress_complexity(mask);
                local
            })
            .collect::<Vec<_>>();
        progress.lock().from = 0;

        let rngs = masks
            .iter()
            .map(|_| {
                rng.jump();
                rng.clone()
            })
            .collect::<Vec<_>>();

        let Some(regions) = masks
            .into_par_iter()
            .zip(progresses)
            .zip(rngs)
            .map(|((mask, progress), mut rng)| {
                self.default_generator.generate(mask, &mut rng, progress)
            })
            .collect()
        else {
            return Err(GeneratorError);
        };

        let connect_regions = Self::connect_regions(groups, regions, &mut rng);
        progress.lock().finish();

        Ok(connect_regions)
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

pub trait RegionGenerator: fmt::Debug + Sync + Send {
    fn generate(&self, mask: CellMask, rng: &mut Random, progress: ProgressHandle) -> Option<Maze>;

    fn guess_progress_complexity(&self, mask: &CellMask) -> usize {
        mask.enabled_count()
    }
}

pub trait RegionSplitter: fmt::Debug + Sync + Send {
    fn split(
        &self,
        mask: &CellMask,
        rng: &mut Random,
        progress: ProgressHandle,
    ) -> Option<Array3D<u8>>;
}

#[derive(Debug, Clone, Copy)]
pub enum RegionCount {
    Every(usize),
    Exact(u8),
}

#[derive(Debug)]
pub struct DefaultRegionSplitter {
    pub count: RegionCount,
}

impl DefaultRegionSplitter {
    pub fn random_points(mask: &CellMask, count: u8, rng: &mut Random) -> Vec<Dims3D> {
        let count = count as usize;
        let mut points = Vec::with_capacity(count);

        while points.len() < count {
            // FIXME: this is absolutely horrible and slow implementation,
            // but since we don't sample a lot of points, it should be fine. I hope...
            let point = mask.random_cell(rng).unwrap();

            if !points.contains(&point) {
                points.push(point);
            }
        }

        points
    }
    pub fn split_groups(
        points: Vec<Dims3D>,
        mask: &CellMask,
        rng: &mut Random,
        progress: ProgressHandle,
    ) -> Option<Array3D<u8>> {
        assert!(points.len() <= u8::MAX as usize);
        assert!(!points.is_empty());
        assert!(points.iter().collect::<HashSet<_>>().len() == points.len());

        progress.lock().from = mask.enabled_count();

        let size = mask.size();
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

            let mut set_new = 0;
            for cell in Dims3D::iter_fill(Dims3D::ZERO, size) {
                if !mask[cell] || groups[cell].is_some() {
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
                    set_new += 1;
                }
            }

            cycle = cycle.wrapping_add(1);
            progress.lock().done += set_new;
            if progress.is_stopped() {
                return None;
            }
        }

        progress.lock().finish();

        Some(groups.map(|group| group.unwrap().0))
    }
}

impl RegionSplitter for DefaultRegionSplitter {
    fn split(
        &self,
        mask: &CellMask,
        rng: &mut Random,
        progress: ProgressHandle,
    ) -> Option<Array3D<u8>> {
        let region_count = match self.count {
            RegionCount::Every(every) => mask.enabled_count() / every,
            RegionCount::Exact(count) => count as usize,
        }
        .clamp(1, u8::MAX as usize) as u8;

        let points = Self::random_points(mask, region_count, rng);

        progress.lock().from = mask.enabled_count();

        let size = mask.size();
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

            let mut set_new = 0;
            for cell in Dims3D::iter_fill(Dims3D::ZERO, size) {
                if !mask[cell] || groups[cell].is_some() {
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
                    set_new += 1;
                }
            }

            cycle = cycle.wrapping_add(1);
            progress.lock().done += set_new;
            if progress.is_stopped() {
                return None;
            }
        }

        progress.lock().finish();

        Some(groups.map(|group| group.unwrap().0))
    }
}
