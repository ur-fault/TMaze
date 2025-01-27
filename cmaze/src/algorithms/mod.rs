pub mod region_generator;
pub mod region_splitter;
pub mod types;

use serde::{Deserialize, Serialize};
use thiserror::Error;
pub use types::*;

use hashbrown::{HashMap, HashSet};
use rand::{seq::SliceRandom as _, thread_rng, Rng as _, SeedableRng as _};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use region_splitter::RegionSplitter;

use std::{iter, ops, sync::Arc};

use crate::{
    array::Array3D,
    dims::*,
    gameboard::{Cell, CellWall, Maze},
    progress::ProgressHandle,
    registry::Registry,
};
use region_generator::RegionGenerator;

/// Random number generator used for anything, where determinism is required.
pub type Random = rand_xoshiro::Xoshiro256StarStar;

/// Registry of the region generators.
pub type GeneratorRegistry = Registry<dyn RegionGenerator>;

/// Registry of the region splitters.
pub type SplitterRegistry = Registry<dyn RegionSplitter>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "CellMaskSerde", into = "CellMaskSerde")]
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

    pub fn as_array3d(&self) -> &Array3D<bool> {
        &self.0
    }

    pub fn fill(&mut self, value: bool) {
        self.0.fill(value);
    }

    pub fn iter_enabled(&self) -> impl Iterator<Item = Dims3D> + '_ {
        self.0.iter_pos().filter(move |&pos| self[pos])
    }
}

impl<T: Clone> Array3D<Option<T>> {
    pub fn to_mask(self) -> CellMask {
        CellMask(self.map(|v| v.is_some()))
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

impl From<Array3D<bool>> for CellMask {
    fn from(array: Array3D<bool>) -> Self {
        Self(array)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum CellMaskSerde {
    Bool(Array3D<bool>),
    Int(Array3D<u8>),
    Base64 { base64: String, size: Dims3D },
}

#[derive(Debug, Error)]
pub enum CellMaskSerdeError {
    #[error("Invalid base64 data: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("Invalid size: {0}")]
    InvalidSize(usize),
}

impl TryFrom<CellMaskSerde> for CellMask {
    type Error = CellMaskSerdeError;

    fn try_from(mask: CellMaskSerde) -> Result<Self, Self::Error> {
        match mask {
            CellMaskSerde::Bool(array) => Ok(CellMask(array)),
            CellMaskSerde::Int(array) => Ok(CellMask(array.map(|v| v != 0))),
            CellMaskSerde::Base64 { base64: bytes, size } => {
                use base64::prelude::*;
                let bits = base64::prelude::BASE64_STANDARD.decode(bytes)?;

                if (size.product() as usize).div_ceil(8) != bits.len() {
                    return Err(CellMaskSerdeError::InvalidSize(bits.len()));
                }

                // bit array -> byte array
                let mut bools = vec![false; size.product() as usize];
                for pos in Dims3D::iter_fill(Dims3D::ZERO, size) {
                    let index = pos.linear_index(size);
                    let byte = bits[index / 8];
                    let bit = (byte >> (index % 8)) & 1;
                    bools[index] = bit == 0; // for unknown fucking reason, the bits are inverted
                }

                Ok(CellMask(Array3D::from_buf(
                    bools,
                    size.0 as usize,
                    size.1 as usize,
                    size.2 as usize,
                )))
            }
        }
    }
}

impl From<CellMask> for CellMaskSerde {
    fn from(mask: CellMask) -> Self {
        CellMaskSerde::Bool(mask.0)
    }
}

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
    },
    ToGenerate {
        mask: CellMask,
        splitter: (Arc<dyn RegionSplitter>, Params),
        generator: (Arc<dyn RegionGenerator>, Params),
    },
}

#[derive(Debug, Clone)]
enum LocalRegionSpec {
    Predefined(Maze),
    ToGenerate {
        generator: Arc<dyn RegionGenerator>,
        params: Params,
    },
}

#[derive(Debug, Clone)]
pub struct Generator {
    seed: Option<u64>,
    splitter: LocalSplitterSpec,
    type_: MazeType,
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
            size,
            inner_spec,
            seed,
            maze_type,
        } = spec;

        match inner_spec {
            MazeSpecType::Regions {
                regions,
                start: _,
                end: _,
            } => {
                let mut region_ids = Array3D::new_dims(None, *size).unwrap();

                let region_specs = regions
                    .iter()
                    .enumerate()
                    .map(|(region_id, MazeRegionSpec { mask, region_type })| {
                        for pos in mask.iter_enabled() {
                            region_ids[pos] = Some(region_id as u8);
                        }
                        match region_type {
                            MazeRegionType::Predefined { maze } => {
                                LocalRegionSpec::Predefined(maze.clone())
                            }
                            MazeRegionType::Generated {
                                generator: (generator, params),
                            } => LocalRegionSpec::ToGenerate {
                                generator: generators.get(generator).expect("unknown generator"),
                                params: params.clone(),
                            },
                        }
                    })
                    .collect::<Vec<_>>();

                Self {
                    seed: *seed,
                    splitter: LocalSplitterSpec::Predefined {
                        regions: region_ids,
                        region_specs,
                    },
                    type_: maze_type.unwrap_or(MazeType::Normal),
                }
            }
            MazeSpecType::Simple {
                start: _,
                end: _,
                mask,
                splitter,
                generator,
            } => Self {
                seed: *seed,
                splitter: LocalSplitterSpec::ToGenerate {
                    mask: mask
                        .clone()
                        .unwrap_or_else(|| CellMask::new_dims(*size).unwrap()),
                    splitter: get_from_registry(&splitters, splitter.as_ref()),
                    generator: get_from_registry(&generators, generator.as_ref()),
                },
                type_: maze_type.unwrap_or(MazeType::default()),
            },
        }
    }

    // TODO: Custom error type
    pub fn generate(&self, progress: ProgressHandle) -> Result<Maze, GeneratorError> {
        let seed = self.seed.unwrap_or_else(|| thread_rng().gen());
        let mut rng = Random::seed_from_u64(seed);

        Ok(match &self.splitter {
            LocalSplitterSpec::Predefined {
                regions,
                region_specs,
            } => {
                let mask = regions.clone().to_mask();
                progress.lock().from = mask.enabled_count();

                // FIXME: this ain't parallelized at all
                let regions = regions.clone().map(|r| r.unwrap_or_default());
                let generated_regions: Vec<_> = region_specs
                    .iter()
                    .map(|spec| match spec {
                        LocalRegionSpec::Predefined(maze) => Some(maze.clone()),
                        LocalRegionSpec::ToGenerate {
                            generator,
                            params: _,
                        } => generator.generate(mask.clone(), &mut rng, progress.split()),
                    })
                    .collect::<Option<_>>()
                    .ok_or(GeneratorError::Unknown)?;

                Self::connect_regions(&regions, &mask, generated_regions, &mut rng)
            }
            LocalSplitterSpec::ToGenerate {
                mask,
                splitter: (splitter, _),
                generator: (generator, _),
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

                let rngs = (0..parts.len())
                    .map(|_| {
                        rng.long_jump();
                        rng.clone()
                    })
                    .collect::<Vec<_>>();

                let parts = parts
                    .into_par_iter()
                    .zip(rngs)
                    .map(|(mask, mut rng)| {
                        let group_count =
                            (mask.enabled_count() / SPLIT_COUNT).clamp(1, u8::MAX as usize) as u8;
                        let groups = splitter
                            .split(&mask, &mut rng, progress.split())
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
                                generator.generate(mask, &mut rng, progress)
                            })
                            .collect()
                        else {
                            return Err(GeneratorError::Unknown);
                        };

                        Ok(Self::connect_regions(&groups, &mask, regions, &mut rng))
                    })
                    .collect::<Result<_, GeneratorError>>()?;

                progress.lock().finish();

                match self.type_ {
                    MazeType::Normal => Self::connect_regions(&groups, mask, parts, &mut rng),
                    MazeType::Tower => {
                        let mut maze = Maze {
                            cells: Array3D::new_dims(Cell::new(), mask.size()).unwrap(),
                            type_: MazeType::Tower,
                        };
                        for pos in mask.iter_enabled() {
                            maze.cells[pos] = parts[pos.2 as usize].cells[Dims3D(pos.0, pos.1, 0)];
                        }

                        maze
                    }
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

    pub fn connect_regions(
        groups: &Array3D<u8>,
        mask: &CellMask,
        regions: Vec<Maze>,
        rng: &mut Random,
    ) -> Maze {
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
        let mut maze = Maze {
            cells: Array3D::new_dims(Cell::new(), groups.size()).unwrap(),
            type_: MazeType::Normal,
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
