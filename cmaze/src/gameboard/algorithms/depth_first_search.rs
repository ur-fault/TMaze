use std::sync::{Arc, Mutex};

use rand::seq::SliceRandom;

use super::{
    super::cell::Cell, GenErrorInstant, GenErrorThreaded, Maze, MazeAlgorithm, Progress,
    StopGenerationFlag,
};
use crate::core::*;

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

        let mut visited: Vec<Dims3D> = Vec::with_capacity(cell_count);
        let mut stack: Vec<Dims3D> = Vec::with_capacity(cell_count);

        let (sx, sy, sz) = (0, 0, 0);

        let mut cells: Vec<Vec<Vec<Cell>>> = vec![vec![Vec::with_capacity(wu); hu]; du];
        for z in 0..d {
            for y in 0..h {
                for x in 0..w {
                    cells[z as usize][y as usize].push(Cell::new(Dims3D(x, y, z)));
                }
            }
        }

        let mut maze = Maze {
            cells,
            width: wu,
            height: hu,
            depth: du,
            is_tower: false,
        };

        let mut current = Dims3D(sx, sy, sz);
        visited.push(current);
        stack.push(current);
        while !stack.is_empty() {
            current = stack.pop().unwrap();
            let unvisited_neighbors = maze
                .get_neighbors(current)
                .into_iter()
                .map(|cell| cell.get_coord())
                .filter(|cell| !visited.contains(cell))
                .collect::<Vec<_>>();

            if !unvisited_neighbors.is_empty() {
                stack.push(current);
                let chosen = *unvisited_neighbors.choose(&mut rand::thread_rng()).unwrap();
                let chosen_wall = Maze::which_wall_between(current, chosen).unwrap();
                maze.remove_wall(current, chosen_wall);
                visited.push(chosen);
                stack.push(chosen);
            }

            progress.lock().unwrap().done = visited.len();

            if stopper.is_stopped() {
                return Err(GenErrorThreaded::AbortGeneration);
            }
        }

        progress.lock().unwrap().is_finished = true;

        Ok(maze)
    }
}
