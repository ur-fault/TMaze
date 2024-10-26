use std::sync::{Arc, Mutex};

use rand::seq::SliceRandom;

use super::{
    super::cell::Cell, GenErrorInstant, GenErrorThreaded, Maze, MazeAlgorithm, Progress,
    StopGenerationFlag,
};

use crate::{array::Array3D, dims::*};

use hashbrown::HashSet;

pub struct DepthFirstSearch {}

impl MazeAlgorithm for DepthFirstSearch {
    fn generate_individual(
        size: Dims3D,
        stopper: StopGenerationFlag,
        progress: Arc<Mutex<Progress>>,
    ) -> Result<Maze, GenErrorThreaded> {
        if size.0 == 0 || size.1 == 0 || size.2 == 0 {
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
            width: wu,
            height: hu,
            depth: du,
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
