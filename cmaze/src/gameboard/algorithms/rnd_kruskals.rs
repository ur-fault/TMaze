use rand::seq::SliceRandom;

use super::{
    super::cell::{Cell, CellWall},
    CellMask, RegionGenerator, Maze, ProgressHandle, Random,
};
use crate::{array::Array3D, dims::*};

use CellWall::*;

use hashbrown::HashSet;

#[derive(Debug)]
pub struct RndKruskals;

impl RegionGenerator for RndKruskals {
    fn generate(&self, mask: CellMask, rng: &mut Random, progress: ProgressHandle) -> Option<Maze> {
        let Dims3D(w, h, d) = mask.size();
        let (wu, hu, du) = (w as usize, h as usize, d as usize);

        let mut walls: Vec<(Dims3D, CellWall)> = Vec::new();
        let mut sets = Vec::<HashSet<Dims3D>>::new();
        for pos in Dims3D::iter_fill(Dims3D::ZERO, mask.size()) {
            if mask[pos] {
                if mask[pos + Dims3D(1, 0, 0)] {
                    walls.push((pos, Right));
                }

                if mask[pos + Dims3D(0, 1, 0)] {
                    walls.push((pos, Bottom));
                }

                if mask[pos + Dims3D(0, 0, 1)] {
                    walls.push((pos, Up));
                }

                sets.push(Some(pos).into_iter().collect());
            }
        }

        let starter_wall_count = walls.len();
        progress.lock().from = walls.len();

        let cells: Array3D<Cell> = Array3D::new(Cell::new(), wu, hu, du);

        let mut maze = Maze {
            cells,
            is_tower: false,
        };

        walls.shuffle(rng);
        while let Some((from, wall)) = walls.pop() {
            let to = from + wall.to_coord();

            let from_set = sets.iter().position(|set| set.contains(&from)).unwrap();

            if sets[from_set].contains(&to) {
                continue;
            }

            maze.remove_wall(from, wall);
            let from_set = sets.swap_remove(from_set);

            let to_set = sets.iter().position(|set| set.contains(&to)).unwrap();
            sets[to_set].extend(from_set);

            progress.lock().done = starter_wall_count - walls.len();
            if progress.is_stopped() {
                return None;
            }
        }

        progress.lock().finish();

        Some(maze)
    }
}
