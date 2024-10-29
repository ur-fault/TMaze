use rand::seq::SliceRandom;
use smallvec::SmallVec;

use super::{super::cell::Cell, CellMask, GroupGenerator, Maze, ProgressHandle, Random};

use crate::array::Array3D;

use hashbrown::HashSet;

#[derive(Debug)]
pub struct DepthFirstSearch;

impl GroupGenerator for DepthFirstSearch {
    fn generate(&self, mask: CellMask, rng: &mut Random, progress: ProgressHandle) -> Maze {
        let size = mask.size();

        let cells = Array3D::new_dims(Cell::new(), size).unwrap();
        let mut maze = Maze {
            cells,
            is_tower: false,
        };

        progress.lock().from = mask.enabled_count();

        let mut visited = HashSet::with_capacity(mask.enabled_count());
        let mut stack = Vec::new();

        let mut current = mask.random_cell(rng).unwrap();

        visited.insert(current);
        stack.push(current);
        while !stack.is_empty() {
            current = stack.pop().unwrap();
            let unvisited_neighbors = maze
                .get_neighbors_pos(current)
                .into_iter()
                .filter(|cell| mask[*cell])
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
        }

        progress.lock().finish();

        maze
    }
}
