use super::super::cell::Cell;
use super::{Maze, MazeAlgorithm};
use crate::game::Error;
use rand::seq::SliceRandom;

pub struct DepthFirstSearch {}

impl MazeAlgorithm for DepthFirstSearch {
    fn new<T: FnMut(usize, usize) -> Result<(), Error>>(
        w: usize,
        h: usize,
        start_: Option<(usize, usize)>,
        mut report_progress: Option<T>,
    ) -> Result<Maze, Error> {
        let mut visited: Vec<(usize, usize)> = Vec::with_capacity(w * h);
        let mut stack: Vec<(usize, usize)> = Vec::with_capacity(w * h);

        let cell_count = w * h;

        let (sx, sy) = start_.unwrap_or((0, 0));

        let mut cells: Vec<Vec<Cell>> = vec![Vec::with_capacity(w); h];
        for y in 0..h {
            for x in 0..w {
                cells[y].push(Cell::new(x, y));
            }
        }

        let mut maze = Maze {
            cells,
            width: w,
            height: h,
        };

        let mut current = (sx, sy);
        visited.push(current);
        stack.push(current);
        while !stack.is_empty() {
            current = stack.pop().unwrap();
            let unvisited_neighbors = maze
                .get_neighbors(current)
                .into_iter()
                .map(|cell| cell.get_coord())
                .filter(|cell| !visited.contains(cell))
                .collect::<Vec<(usize, usize)>>();

            if !unvisited_neighbors.is_empty() {
                stack.push(current);
                let chosen = *unvisited_neighbors.choose(&mut rand::thread_rng()).unwrap();
                let chosen_wall = Maze::which_wall(current, chosen);
                maze.remove_wall(current, chosen_wall);
                visited.push(chosen);
                stack.push(chosen);
            }

            if let Some(_) = report_progress {
                report_progress.as_mut().unwrap()(visited.len(), cell_count)?;
            }
        }

        Ok(maze)
    }
}
