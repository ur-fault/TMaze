use std::fmt;

use hashbrown::HashSet;
use rand::seq::SliceRandom as _;
use smallvec::SmallVec;

use crate::{
    array::Array3D,
    gameboard::{Cell, CellWall, Maze},
    progress::ProgressHandle,
};

use super::{CellMask, Dims3D, MazeType, Random};

pub trait RegionGenerator: fmt::Debug + Sync + Send {
    fn generate(&self, mask: CellMask, rng: &mut Random, progress: ProgressHandle) -> Option<Maze>;

    fn guess_progress_complexity(&self, mask: &CellMask) -> usize {
        mask.enabled_count()
    }
}

#[derive(Debug)]
pub struct DepthFirstSearch;

impl RegionGenerator for DepthFirstSearch {
    fn generate(&self, mask: CellMask, rng: &mut Random, progress: ProgressHandle) -> Option<Maze> {
        let size = mask.size();

        let cells = Array3D::new_dims(Cell::new(), size).unwrap();
        let mut maze = Maze {
            cells,
            mask,
            type_: MazeType::default(),
            start: Dims3D::ZERO,
            end: Dims3D::ZERO,
        };

        progress.lock().from = maze.mask.enabled_count();

        let mut visited = HashSet::with_capacity(maze.mask.enabled_count());
        let mut stack = Vec::new();

        let mut current = maze.mask.random_cell(rng).unwrap();

        visited.insert(current);
        stack.push(current);
        while !stack.is_empty() {
            current = stack.pop().unwrap();
            let unvisited_neighbors = maze
                .get_neighbors_pos(current)
                .into_iter()
                .filter(|cell| maze.mask[*cell])
                .filter(|cell| !visited.contains(cell))
                .collect::<SmallVec<[_; 6]>>();

            if !unvisited_neighbors.is_empty() {
                stack.push(current);
                let next = *unvisited_neighbors.choose(rng).unwrap();
                let chosen_wall = Maze::which_wall_between(current, next).unwrap();
                maze.remove_wall(current, chosen_wall);
                visited.insert(next);
                stack.push(next);
            }

            progress.lock().done = visited.len();
            if progress.is_stopped() {
                return None;
            }
        }

        progress.lock().finish();

        Some(maze)
    }
}

#[derive(Debug)]
pub struct RndKruskals;

impl RegionGenerator for RndKruskals {
    fn generate(&self, mask: CellMask, rng: &mut Random, progress: ProgressHandle) -> Option<Maze> {
        let Dims3D(w, h, d) = mask.size();
        let (wu, hu, du) = (w as usize, h as usize, d as usize);

        let mut walls: Vec<(Dims3D, CellWall)> = Vec::new();
        let mut sets = Vec::<HashSet<Dims3D>>::new();
        for pos in Dims3D::iter_fill(Dims3D::ZERO, mask.size()) {
            use CellWall::*;

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
            mask,
            type_: MazeType::default(),
            start: Dims3D::ZERO,
            end: Dims3D::ZERO,
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
