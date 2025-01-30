use std::str::FromStr;

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use crate::gameboard::Maze;

use super::{CellMask, Dims3D, GeneratorRegistry, SplitterRegistry};

/// Parameters for different algorithms. Region splitter, region generator, etc.
/// In the future, not only String will be allowed, but also other types.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Params {
    map: HashMap<String, String>,
}

impl Params {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(|s| s.as_str())
    }

    pub fn parsed<T: FromStr>(&self, key: &str) -> Option<Result<T, T::Err>> {
        self.get(key).map(|s| s.parse())
    }

    pub fn parsed_or<T: FromStr>(&self, key: &str, default: T) -> T {
        match self.parsed(key) {
            None | Some(Err(_)) => default,
            Some(Ok(v)) => v,
        }
    }

    pub fn parsed_or_warn<T: FromStr>(&self, key: &str, default: T) -> T {
        match self.parsed(key) {
            None => default,
            Some(Ok(v)) => v,
            Some(Err(_)) => {
                log::warn!("Invalid value for parameter '{}', using default value", key);
                default
            }
        }
    }
}

/// Specific algorithm specification.
pub type Algorithm = (String, Params);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MazeSpec {
    // /// Size of the maze.
    // pub size: Dims3D,
    /// Specification of the maze.
    #[serde(default, flatten)]
    pub inner_spec: MazeSpecType,

    /// Seed of the maze.
    ///
    /// Used for deterministic generation.
    pub seed: Option<u64>,

    /// Type of the maze.
    pub maze_type: Option<MazeType>,
}

impl MazeSpec {
    pub fn validate(&self, generators: &GeneratorRegistry, splitters: &SplitterRegistry) -> bool {
        fn check_position(pos: Position, regions: &[MazeRegionSpec]) -> bool {
            match pos {
                Position::Pos(pos) => {
                    // Is the position at least in one region?
                    regions.iter().any(|r| r.mask[pos])
                }
                Position::Region(region) => {
                    // Is the region valid?
                    (region as usize) < regions.len()
                }
            }
        }

        fn pos_to_dim(pos: Position, regions: &[MazeRegionSpec]) -> Dims3D {
            match pos {
                Position::Pos(pos) => pos,
                Position::Region(region) => {
                    regions[region as usize].mask.iter_enabled().next().unwrap()
                }
            }
        }

        match &self.inner_spec {
            MazeSpecType::Regions {
                regions,
                start,
                end,
            } => {
                if let (Some(Position::Pos(start)), Some(Position::Pos(end))) = (start, end) {
                    if start == end {
                        return false;
                    }
                }

                if regions.is_empty() {
                    return false;
                }

                let size = regions.first().unwrap().mask.size();
                if regions.iter().any(|r| !r.validate(size, generators)) {
                    return false;
                }

                let mut exclusion_check_mask = CellMask::new_dims_empty(size).unwrap();
                for region in regions {
                    for pos in region.mask.iter_enabled() {
                        if exclusion_check_mask[pos] {
                            return false;
                        }

                        exclusion_check_mask[pos] = true;
                    }
                }

                if let Some(start) = *start {
                    if !check_position(start, regions) {
                        return false;
                    }
                }

                if let Some(end) = *end {
                    if !check_position(end, regions) {
                        return false;
                    }
                }

                if let (Some(start), Some(end)) = (start, end) {
                    let union = regions
                        .iter()
                        .fold(CellMask::new_dims_empty(size).unwrap(), |a, b| {
                            a.or(&b.mask)
                        });
                    if !union.connactable(pos_to_dim(*start, regions), pos_to_dim(*end, regions)) {
                        return false;
                    }
                }
            }

            MazeSpecType::Simple {
                size,
                start,
                end,
                mask,
                generator,
                splitter,
            } => {
                if let (Some(start), Some(end)) = (start, end) {
                    if start == end {
                        return false;
                    }
                }

                let Some(size) = size.or_else(|| mask.as_ref().map(|m| m.size())) else {
                    return false;
                };

                if !size.all_positive() {
                    return false;
                }

                if let Some(mask) = mask {
                    if mask.size() != size {
                        return false;
                    }

                    if mask.is_empty() {
                        return false;
                    }

                    if let Some(start) = start {
                        if !mask[*start] {
                            return false;
                        }
                    }

                    if let Some(end) = end {
                        if !mask[*end] {
                            return false;
                        }
                    }

                    if let (Some(start), Some(end)) = (start, end) {
                        if !mask.connactable(*start, *end) {
                            return false;
                        }
                    }
                }

                if let Some(generator) = generator {
                    if !generators.is_registered(&generator.0) {
                        return false;
                    }
                }

                if let Some(splitter) = splitter {
                    if !splitters.is_registered(&splitter.0) {
                        return false;
                    }
                }
            }
        }

        true
    }

    pub fn size(&self) -> Option<Dims3D> {
        Some(match &self.inner_spec {
            MazeSpecType::Regions { regions, .. } => regions.first()?.mask.size(),
            MazeSpecType::Simple { size, mask, .. } => {
                size.or_else(|| mask.as_ref().map(|m| m.size()))?
            }
        })
    }
}

/// Maze specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MazeSpecType {
    Regions {
        /// Specified regions.
        regions: Vec<MazeRegionSpec>,

        /// Player start position.
        start: Option<Position>,

        /// Player end position.
        end: Option<Position>,
    },
    /// Simple maze specification.
    ///
    /// Used for basic mazes, mostly specified inside the user config. User can specify only the
    /// size and the mask of the maze and used algorithms for splitting and generating the maze.
    Simple {
        /// Maze size.
        ///
        /// Can be ommited when the mask is specified.
        size: Option<Dims3D>,

        /// Player start position.
        ///
        /// We don't use [`Position`] here, because it's not possible to specify region to start in,
        /// since the regions are not generated yet.
        start: Option<Dims3D>,

        /// Player end position.
        ///
        /// We don't use [`Position`] here, because it's not possible to specify region to start in,
        /// since the regions are not generated yet.
        end: Option<Dims3D>,

        /// Mask of the maze.
        mask: Option<CellMask>,

        /// Region splitter.
        splitter: Option<Algorithm>,

        /// Region generator.
        generator: Option<Algorithm>,
    },
    // TODO: Combined, where we can specify specific regions and mask of the rest, and it's handled
    // automatically by the generator.
}

// impl Default for MazeSpecType {
//     fn default() -> Self {
//         MazeSpecType::Simple {
//             start: None,
//             end: None,
//             mask: None,
//             splitter: None,
//             generator: None,
//         }
//     }
// }

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Position {
    Pos(Dims3D),
    Region(u8),
}

/// Maze type.
///
/// Is not exhaustive, but generators can report that they don't support the given maze type.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MazeType {
    #[default]
    Normal,
    Tower,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MazeRegionSpec {
    pub mask: CellMask,
    pub region_type: MazeRegionType,
}

impl MazeRegionSpec {
    pub fn validate(&self, maze_size: Dims3D, generators: &GeneratorRegistry) -> bool {
        if self.mask.size() != maze_size {
            return false;
        }

        if !self.mask.is_connected() {
            return false;
        }

        match &self.region_type {
            MazeRegionType::Predefined { maze } => {
                if maze.size() != self.mask.size() || maze.size() != maze_size {
                    return false;
                }
            }
            MazeRegionType::Generated {
                generator: (gen, _),
                seed: _,
            } => {
                if !generators.is_registered(gen) {
                    return false;
                }
            }
        }

        true
    }
}

/// Specification of a maze region.
///
/// Used in the preset files to define the regions of the maze. Mods will also be able to use this,
/// once they are implemented.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MazeRegionType {
    /// Predefined maze region.
    ///
    /// Basically already generated maze region. It is used if the user wants specific part of the
    /// maze or if it's pregenerated externally.
    Predefined {
        /// Maze of this region.
        maze: Maze,
    },

    /// Generated maze region.
    ///
    /// This defines how a region of the maze should be generated.
    Generated {
        /// Name of the generator used to generate the maze.
        /// Must be registered in the generator registry.
        generator: Algorithm,

        /// Optional seed for the region.
        /// If not specified, it will be calculated from the maze seed.
        seed: Option<u64>,
    },
}
