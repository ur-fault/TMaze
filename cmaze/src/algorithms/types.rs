use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use crate::gameboard::Maze;

use super::{CellMask, Dims3D};

/// Parameters for different algorithms. Region splitter, region generator, etc.
/// In the future, not only String will be allowed, but also other types.
pub type Params = HashMap<String, String>;

/// Specific algorithm specification.
pub type Algorithm = (String, Params);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MazeSpec {
    /// Size of the maze.
    pub size: Dims3D,

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
    pub fn validate(&self) -> bool {
        fn check_position(pos: Position, regions: &[MazeRegionSpec]) -> bool {
            match pos {
                Position::Pos(pos) => {
                    // Is the position at least in one region?
                    regions.iter().any(|r| r.mask[pos])
                }
                Position::Region(region) => {
                    // Is the region valid?
                    regions.len() > region as usize
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

                if regions.iter().any(|r| !r.validate(self.size)) {
                    return false;
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
            }

            MazeSpecType::Simple {
                start, end, mask, ..
            } => {
                if let (Some(start), Some(end)) = (start, end) {
                    if start == end {
                        return false;
                    }
                }

                if let Some(mask) = mask {
                    if mask.size() != self.size {
                        return false;
                    }
                }
            }
        }

        true
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

impl Default for MazeSpecType {
    fn default() -> Self {
        MazeSpecType::Simple {
            start: None,
            end: None,
            mask: None,
            splitter: None,
            generator: None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    pub fn validate(&self, maze_size: Dims3D) -> bool {
        match &self.region_type {
            MazeRegionType::Predefined { maze } => {
                if maze.size() != self.mask.size() || maze.size() != maze_size {
                    return false;
                }
            }
            MazeRegionType::Generated { .. } => {}
        }

        true
    }
}

/// Specification of a maze region.
///
/// Used in the preset files to define the regions of the maze. Mods will also be able to use this,
/// once they are implemented.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    },
}
