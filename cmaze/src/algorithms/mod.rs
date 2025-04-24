pub mod region_generator;
pub mod region_splitter;
pub mod types;

use hashbrown::{HashMap, HashSet};
use rand::{seq::SliceRandom as _, thread_rng, Rng as _, SeedableRng as _};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

use std::{
    iter,
    sync::{Arc, Mutex},
};

use crate::{
    array::Array3D,
    dims::*,
    gameboard::{maze::MazeBoard, Cell, CellWall, Maze},
    progress::ProgressHandle,
    registry::Registry,
};
use region_generator::RegionGenerator;
use region_splitter::RegionSplitter;
pub use types::*;

/// Random number generator used for anything, where determinism is required.
pub type Random = rand_xoshiro::Xoshiro256StarStar;

/// Registry of the region generators.
pub type GeneratorRegistry = Registry<dyn RegionGenerator>;

/// Registry of the region splitters.
pub type SplitterRegistry = Registry<dyn RegionSplitter>;

#[derive(Debug)]
pub enum GeneratorError {
    Unknown,
    Validation,
}

#[derive(Debug, Clone)]
enum LocalSplitterSpec {
    Predefined {
        regions: Array3D<Option<u8>>,
        region_specs: Vec<LocalRegionSpec>,
        active_region_heuristic: Option<RegionChooseHeuristic>,
    },
    ToGenerate {
        mask: CellMask,
        splitter: (Arc<dyn RegionSplitter>, Params),
        generator: (Arc<dyn RegionGenerator>, Params),
    },
}

#[derive(Debug, Clone)]
enum LocalRegionSpec {
    Predefined(MazeBoard),
    ToGenerate {
        generator: Arc<dyn RegionGenerator>,
        params: Params,
        seed: Option<u64>,
    },
}

#[derive(Debug, Clone, Copy)]
enum PosInMaze {
    Region(u8),
    Cell(Dims3D),
}

impl From<Position> for PosInMaze {
    fn from(pos: Position) -> Self {
        match pos {
            Position::Region(id) => PosInMaze::Region(id),
            Position::Pos(pos) => PosInMaze::Cell(pos),
        }
    }
}

impl From<Dims3D> for PosInMaze {
    fn from(pos: Dims3D) -> Self {
        PosInMaze::Cell(pos)
    }
}

/// Main struct of this module.
///
/// It generates the complete maze from specification and from the optional seed. For more info
/// check out [`LocalSplitterSpec`], [`LocalRegionSpec`] and [`MazeType`].
#[derive(Debug, Clone)]
pub struct Generator {
    seed: Option<u64>,
    splitter: LocalSplitterSpec,
    type_: MazeType,
    start: Option<PosInMaze>,
    end: Option<PosInMaze>,
}

impl Generator {
    pub fn from_maze_spec(
        spec: &MazeSpec,
        generators: &GeneratorRegistry,
        splitters: &SplitterRegistry,
    ) -> Self {
        fn get_from_registry<T: ?Sized>(
            registry: &Registry<T>,
            pair: Option<&(String, Params)>,
        ) -> (Arc<T>, Params) {
            pair.as_ref()
                .map(|(name, params)| (registry.get(name).unwrap(), params.clone()))
                .unwrap_or_else(|| (registry.get_default().unwrap(), Params::default()))
        }

        let MazeSpec {
            inner_spec,
            seed,
            maze_type,
        } = spec;

        match inner_spec {
            MazeSpecType::Regions {
                regions,
                start,
                end,
                active_region_heuristic,
            } => {
                let size = regions.first().unwrap().mask.size();
                let mut region_ids = Array3D::new_dims(None, size).unwrap();

                let region_specs = regions
                    .iter()
                    .enumerate()
                    .map(|(region_id, MazeRegionSpec { mask, region_type })| {
                        for pos in mask.iter_enabled() {
                            region_ids[pos] = Some(region_id as u8);
                        }
                        match region_type {
                            MazeRegionType::Predefined { board } => {
                                LocalRegionSpec::Predefined(board.clone())
                            }
                            MazeRegionType::Generated {
                                generator: (generator, params),
                                seed,
                            } => LocalRegionSpec::ToGenerate {
                                generator: generators.get(generator).expect("unknown generator"),
                                params: params.clone(),
                                seed: *seed,
                            },
                        }
                    })
                    .collect::<Vec<_>>();

                Self {
                    seed: *seed,
                    splitter: LocalSplitterSpec::Predefined {
                        regions: region_ids,
                        region_specs,
                        active_region_heuristic: *active_region_heuristic,
                    },
                    type_: maze_type.unwrap_or(MazeType::Normal),
                    start: start.map(Into::into),
                    end: end.map(Into::into),
                }
            }
            MazeSpecType::Simple {
                start,
                end,
                size,
                mask,
                splitter,
                generator,
            } => {
                let size = size.or_else(|| mask.as_ref().map(|m| m.size())).unwrap();
                Self {
                    seed: *seed,
                    splitter: LocalSplitterSpec::ToGenerate {
                        mask: mask
                            .clone()
                            .unwrap_or_else(|| CellMask::new_dims(size).unwrap()),
                        splitter: get_from_registry(splitters, splitter.as_ref()),
                        generator: get_from_registry(generators, generator.as_ref()),
                    },
                    type_: maze_type.unwrap_or(MazeType::default()),
                    start: start.map(Into::into),
                    end: end.map(Into::into),
                }
            }
        }
    }

    pub fn generate(&self, progress: ProgressHandle) -> Result<Maze, GeneratorError> {
        let seed = self.seed.unwrap_or_else(|| thread_rng().gen());
        let mut rng = Random::seed_from_u64(seed);

        Ok(match &self.splitter {
            LocalSplitterSpec::Predefined {
                regions,
                region_specs,
                active_region_heuristic,
            } => {
                let mask = regions.clone().to_mask();
                progress.lock().from = mask.enabled_count();

                let regions = regions.clone();
                let region_count = region_specs.len();
                let generated_regions: Vec<_> = region_specs
                    .clone()
                    .into_par_iter()
                    .zip(split_rng(&mut rng, region_count))
                    .enumerate()
                    .map(|(i, (spec, mut rng))| match spec {
                        LocalRegionSpec::Predefined(maze) => Some(maze),
                        LocalRegionSpec::ToGenerate {
                            generator,
                            params,
                            seed,
                        } => {
                            let region_mask =
                                CellMask::from(regions.clone().map(|r| r == Some(i as u8)));
                            if let Some(seed) = seed {
                                rng = Random::seed_from_u64(seed);
                            }
                            generator.generate(region_mask, &mut rng, progress.split(), &params)
                        }
                    })
                    .collect::<Option<_>>()
                    .ok_or(GeneratorError::Unknown)?;

                let regions = regions.map(|r| r.unwrap_or_default());
                let board = Self::connect_regions(&regions, &mask, generated_regions, &mut rng);

                let mask = match active_region_heuristic.unwrap_or_default() {
                    RegionChooseHeuristic::Biggest => mask
                        .disjoint_parts()
                        .into_iter()
                        .max_by_key(|m| m.enabled_count())
                        .unwrap(),
                    RegionChooseHeuristic::Random => {
                        mask.disjoint_parts().choose(&mut rng).cloned().unwrap()
                    }
                    RegionChooseHeuristic::First => mask.connected(mask.first().unwrap()),
                    RegionChooseHeuristic::Last => mask.connected(mask.last().unwrap()),
                };

                let (start, end) = match (self.start, self.end) {
                    (Some(PosInMaze::Cell(start)), Some(PosInMaze::Cell(end))) => (start, end),
                    (Some(PosInMaze::Cell(start)), None) => {
                        if start != mask.last().unwrap() {
                            (start, mask.last().unwrap())
                        } else {
                            (start, mask.first().unwrap())
                        }
                    }
                    (None, Some(PosInMaze::Cell(end))) => {
                        if end != mask.first().unwrap() {
                            (mask.first().unwrap(), end)
                        } else {
                            (mask.last().unwrap(), end)
                        }
                    }
                    (Some(PosInMaze::Region(id1)), Some(PosInMaze::Region(id2))) => {
                        if id1 != id2 {
                            (
                                Self::random_in_region(&mut rng, &mask, &regions, id1).unwrap(),
                                Self::random_in_region(&mut rng, &mask, &regions, id2).unwrap(),
                            )
                        } else {
                            let mut tmp_mask = Self::mask_of_region(id1, &regions, &mask);
                            let start = tmp_mask.random_cell(&mut rng).unwrap();
                            tmp_mask[start] = false;
                            (
                                start,
                                tmp_mask
                                    .random_cell(&mut rng)
                                    .ok_or(GeneratorError::Validation)?,
                            )
                        }
                    }
                    (Some(PosInMaze::Region(id)), end @ (None | Some(PosInMaze::Cell(_)))) => {
                        let mut region_mask = Self::mask_of_region(id, &regions, &mask);
                        let end = match end {
                            Some(PosInMaze::Cell(end)) => end,
                            None => mask.last().unwrap(),
                            _ => unreachable!("end should be a cell or none"),
                        };
                        region_mask[end] = false;
                        (
                            region_mask
                                .random_cell(&mut rng)
                                .ok_or(GeneratorError::Validation)?,
                            end,
                        )
                    }
                    (start @ (None | Some(PosInMaze::Cell(_))), Some(PosInMaze::Region(id))) => {
                        let mut region_mask = Self::mask_of_region(id, &regions, &mask);
                        let start = match start {
                            Some(PosInMaze::Cell(start)) => start,
                            None => mask.first().unwrap(),
                            _ => unreachable!("start should be a cell or none"),
                        };
                        region_mask[start] = false;
                        (
                            start,
                            region_mask
                                .random_cell(&mut rng)
                                .ok_or(GeneratorError::Validation)?,
                        )
                    }
                    (None, None) => (mask.first().unwrap(), mask.last().unwrap()),
                };

                Maze {
                    board,
                    type_: self.type_,
                    start,
                    end,
                }
            }
            LocalSplitterSpec::ToGenerate {
                mask,
                splitter: (splitter, split_args),
                generator: (generator, gen_args),
            } => {
                progress.lock().from = generator.guess_progress_complexity(mask);

                const SPLIT_COUNT: usize = 100;

                let parts = match self.type_ {
                    MazeType::Normal => vec![mask.clone()],
                    MazeType::Tower => mask
                        .as_array3d()
                        .layers()
                        .map(|l| l.to_array().into())
                        .collect(),
                };

                let mut groups = Array3D::new_dims(0, mask.size()).unwrap();
                for (i, part) in parts.iter().enumerate() {
                    for pos in part.iter_enabled() {
                        groups[pos] = i.try_into().expect("too many parts (floors)");
                    }
                }

                let parts = parts
                    .into_par_iter()
                    .zip(split_rng(&mut rng, SPLIT_COUNT))
                    .map(|(mask, mut rng)| {
                        let group_count =
                            (mask.enabled_count() / SPLIT_COUNT).clamp(1, u8::MAX as usize) as u8;
                        let groups = splitter
                            .split(&mask, &mut rng, progress.split(), split_args)
                            .ok_or(GeneratorError::Unknown)?;
                        let masks = Self::split_to_masks(group_count, &groups, &mask);

                        if progress.is_stopped() {
                            return Err(GeneratorError::Unknown);
                        }

                        let progresses = masks
                            .iter()
                            .map(|mask| {
                                let local = progress.split();
                                local.lock().from = generator.guess_progress_complexity(mask);
                                local
                            })
                            .collect::<Vec<_>>();
                        progress.lock().from = 0;

                        let Some(regions) = masks
                            .into_par_iter()
                            .zip(progresses)
                            .zip(split_rng(&mut rng, group_count as usize))
                            .map(|((mask, progress), mut rng)| {
                                generator.generate(mask, &mut rng, progress, gen_args)
                            })
                            .collect()
                        else {
                            return Err(GeneratorError::Unknown);
                        };

                        Ok(Self::connect_regions(&groups, &mask, regions, &mut rng))
                    })
                    .collect::<Result<_, GeneratorError>>()?;

                progress.lock().finish();

                let board = match self.type_ {
                    MazeType::Normal => Self::connect_regions(&groups, mask, parts, &mut rng),
                    MazeType::Tower => {
                        let mut board = MazeBoard {
                            cells: Array3D::new_dims(Cell::new(), mask.size()).unwrap(),
                            mask: mask.clone(),
                        };
                        for pos in mask.iter_enabled() {
                            board.cells[pos] = parts[pos.2 as usize].cells[Dims3D(pos.0, pos.1, 0)];
                        }

                        // connect the floors
                        // TODO: this implementation has a bug, when each floor is not fully connected
                        for (floor, window) in parts.windows(2).enumerate() {
                            let [from, to] = window else {
                                unreachable!("windows should be long 2");
                            };
                            let mask = from.mask.or(&to.mask);

                            let rnd_cell = mask.random_cell(&mut rng).unwrap();
                            board.cells[Dims3D(rnd_cell.0, rnd_cell.1, floor as i32)]
                                .remove_wall(CellWall::Up);
                        }

                        board
                    }
                };

                let (start, end) = match (self.start, self.end) {
                    (Some(PosInMaze::Cell(start)), Some(PosInMaze::Cell(end))) => (start, end),
                    (Some(PosInMaze::Cell(start)), None) => {
                        if start != mask.last().unwrap() {
                            (start, mask.last().unwrap())
                        } else {
                            (start, mask.first().unwrap())
                        }
                    }
                    (None, Some(PosInMaze::Cell(end))) => {
                        if end != mask.first().unwrap() {
                            (mask.first().unwrap(), end)
                        } else {
                            (mask.last().unwrap(), end)
                        }
                    }
                    (None, None) => (mask.first().unwrap(), mask.last().unwrap()),
                    _ => {
                        return Err(GeneratorError::Validation);
                    }
                };

                Maze {
                    board,
                    type_: self.type_,
                    start,
                    end,
                }
            }
        })
    }

    // Split groups into masks, ready for maze generation
    pub fn split_to_masks(group_count: u8, groups: &Array3D<u8>, mask: &CellMask) -> Vec<CellMask> {
        let mut masks =
            vec![CellMask::new_dims_empty(groups.size()).unwrap(); group_count as usize];

        for (cell, &group) in groups.iter_pos().zip(groups.iter()) {
            if mask[cell] {
                masks[group as usize][cell] = true;
            }
        }

        masks
    }

    // Get mask of the single region
    pub fn mask_of_region(region: u8, groups: &Array3D<u8>, mask: &CellMask) -> CellMask {
        let mut reg_mask = CellMask::new_dims_empty(groups.size()).unwrap();

        for (cell, &group) in groups.iter_pos().zip(groups.iter()) {
            if group == region && mask[cell] {
                reg_mask[cell] = true;
            }
        }

        reg_mask
    }

    fn random_in_region(
        rng: &mut rand_xoshiro::Xoshiro256StarStar,
        mask: &CellMask,
        regions: &Array3D<u8>,
        region_id: u8,
    ) -> Option<Dims3D> {
        Self::mask_of_region(region_id, regions, mask).random_cell(rng)
    }

    pub fn connect_regions(
        groups: &Array3D<u8>,
        mask: &CellMask,
        regions: Vec<MazeBoard>,
        rng: &mut Random,
    ) -> MazeBoard {
        // Disclaimer: this implementation can be slow af, since there is a maximum of a 256 groups
        // We use a simple Kruskal's algorithm to connect the regions

        let mut walls = HashMap::new();
        for (pair, way) in Self::build_region_graph(groups, mask) {
            assert!(pair.0 < pair.1);
            walls.entry(pair).or_insert_with(Vec::new).push(way);
        }

        // Choose only one wall from all of the pairs
        let mut walls: Vec<_> = walls
            .into_iter()
            .map(|(k, v)| (k, *v.choose(rng).unwrap()))
            .collect();
        walls.shuffle(rng);

        let mut sets: Vec<HashSet<u8>> = (0..regions.len() as u8)
            .map(|i| iter::once(i).collect())
            .collect();

        // Combine the regions, so we can start connecting them
        let mut board = MazeBoard {
            cells: Array3D::new_dims(Cell::new(), groups.size()).unwrap(),
            mask: mask.clone(),
        };
        for cell in groups.iter_pos() {
            let group = groups[cell];
            let region = &regions[group as usize];
            board.cells[cell] = region.cells[cell];
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
            board.remove_wall(from, dir);

            let from_set = sets.swap_remove(from_set.0);
            let to_set = sets.iter_mut().find(|set| set.contains(&to_g)).unwrap();
            to_set.extend(from_set);
        }

        board
    }

    pub fn build_region_graph(
        groups: &Array3D<u8>,
        mask: &CellMask,
    ) -> Vec<((u8, u8), (Dims3D, CellWall))> {
        let mut borders = vec![];

        for cell in mask.iter_enabled() {
            let group = groups[cell];

            use CellWall::*;
            for dir in [Right, Bottom, Down] {
                let neighbor = cell + dir.to_coord();
                if !mask[neighbor] {
                    continue;
                }

                if let Some(&neighbor_group) = groups.get(neighbor) {
                    if neighbor_group != group {
                        borders.push((
                            (group.min(neighbor_group), group.max(neighbor_group)),
                            (cell, dir),
                        ));
                    }
                }
            }
        }

        borders
    }
}

fn split_rng(
    rng: &mut Random,
    count: usize,
) -> impl IndexedParallelIterator<Item = Random> + use<'_> {
    let rng = Mutex::new(rng);
    (0..count).into_par_iter().map(move |_| {
        let mut rng = rng.lock().unwrap();
        rng.long_jump();
        rng.clone()
    })
}
