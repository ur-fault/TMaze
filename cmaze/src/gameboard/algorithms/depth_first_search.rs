use std::sync::{Arc, Mutex};

use rand::seq::SliceRandom;
use smallvec::SmallVec;

use super::{
    super::cell::Cell, CellMask, Flag, GenErrorInstant, GenErrorThreaded, GroupGenerator, Maze,
    MazeAlgorithm, Progress, Random,
};

use crate::{array::Array3D, dims::*};

use hashbrown::HashSet;

#[derive(Debug)]
pub struct DepthFirstSearch;

impl GroupGenerator for DepthFirstSearch {
    fn generate(&self, mask: CellMask, rng: &mut Random) -> Maze {
        let size = mask.size();

        let cells = Array3D::new_dims(Cell::new(), size).unwrap();
        let mut maze = Maze {
            cells,
            is_tower: false,
        };

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
        }

        maze
    }
}

impl MazeAlgorithm for DepthFirstSearch {
    fn generate_individual(
        size: Dims3D,
        stopper: Flag,
        progress: Arc<Mutex<Progress>>,
    ) -> Result<Maze, GenErrorThreaded> {
        if !size.all_positive() {
            return Err(GenErrorThreaded::GenerationError(
                GenErrorInstant::InvalidSize(size),
            ));
        }

        let Dims3D(w, h, d) = size;
        let (wu, hu, du) = (w as usize, h as usize, d as usize);
        let cell_count = wu * hu * du;
        progress.lock().unwrap().from = cell_count;

        let mut visited = HashSet::with_capacity(cell_count);
        let mut stack = Vec::with_capacity(cell_count);

        let cells = Array3D::new(Cell::new(), wu, hu, du);
        let mut maze = Maze {
            cells,
            is_tower: false,
        };

        let mut current = Dims3D::ZERO;
        visited.insert(current);
        stack.push(current);
        while !stack.is_empty() {
            current = stack.pop().unwrap();
            let unvisited_neighbors = maze
                .get_neighbors_pos(current)
                .into_iter()
                .filter(|cell| !visited.contains(cell))
                .collect::<Vec<_>>();

            if !unvisited_neighbors.is_empty() {
                stack.push(current);
                let chosen = *unvisited_neighbors.choose(&mut rand::thread_rng()).unwrap();
                let chosen_wall = Maze::which_wall_between(current, chosen).unwrap();
                maze.remove_wall(current, chosen_wall);
                visited.insert(chosen);
                stack.push(chosen);
            }

            progress.lock().unwrap().done = visited.len();

            if stopper.is_stopped() {
                return Err(GenErrorThreaded::AbortGeneration);
            }
        }

        progress.lock().unwrap().is_done = true;

        Ok(maze)
    }
}
