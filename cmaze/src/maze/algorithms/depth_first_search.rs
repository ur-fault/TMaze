use super::super::cell::Cell;
use super::{Maze, MazeAlgorithm, ReportCallbackError, GenerationError};
use crate::maze::CellWall;
use crate::core::*;
use rand::{seq::SliceRandom, thread_rng, Rng};
use rayon::prelude::*;
use std::fmt;

pub struct DepthFirstSearch {}

impl<R, A> MazeAlgorithm<R, A> for DepthFirstSearch where R: fmt::Debug, A: fmt::Debug {
    fn generate<T: FnMut(usize, usize) -> Result<(), ReportCallbackError<R, A>>>(
        size: Dims3D,
        floored: bool,
        mut report_progress: Option<T>,
    ) -> Result<Maze, GenerationError<R, A>> {
        if size.0 == 0 || size.1 == 0 || size.2 == 0 {
            return Err(GenerationError::InvalidSize(size));
        }

        let (w, h, d) = size;
        let (wu, hu, du) = (w as usize, h as usize, d as usize);

        Ok(Maze {
            cells: if size.2 > 1 && floored {
                let mut cells: Vec<_> = (0..d)
                    .map(|_| {
                        Ok(Self::generate_individual((w, h, 1), report_progress.as_mut())?
                            .cells
                            .remove(0))
                    })
                    .collect::<Result<_, GenerationError<R, A>>>()?;

                for floor in 0..du - 1 {
                    let (x, y) = (thread_rng().gen_range(0..wu), thread_rng().gen_range(0..hu));
                    cells[floor][y][x].remove_wall(CellWall::Up);
                    cells[floor + 1][y][x].remove_wall(CellWall::Down);
                }

                cells
            } else {
                Self::generate_individual((w, h, d), report_progress.as_mut())
                    .unwrap()
                    .cells
            },
            width: wu,
            height: hu,
            depth: du,
        })
    }

    fn generate_individual<T: FnMut(usize, usize) -> Result<(), ReportCallbackError<R, A>>>(
        size: Dims3D,
        mut report_progress: Option<T>,
    ) -> Result<Maze, GenerationError<R, A>> {
        if size.0 == 0 || size.1 == 0 || size.2 == 0 {
            return Err(GenerationError::InvalidSize(size));
        }
        let (w, h, d) = size;
        let (wu, hu, du) = (w as usize, h as usize, d as usize);
        let cell_count = wu * hu * du;

        let mut visited: Vec<Dims3D> = Vec::with_capacity(cell_count);
        let mut stack: Vec<Dims3D> = Vec::with_capacity(cell_count);

        let (sx, sy, sz) = (0, 0, 0);

        let mut cells: Vec<Vec<Vec<Cell>>> = vec![vec![Vec::with_capacity(wu); hu]; du];
        for z in 0..d {
            for y in 0..h {
                for x in 0..w {
                    cells[z as usize][y as usize].push(Cell::new((x, y, z)));
                }
            }
        }

        let mut maze = Maze {
            cells,
            width: wu,
            height: hu,
            depth: du,
        };

        let mut current = (sx, sy, sz);
        visited.push(current);
        stack.push(current);
        while !stack.is_empty() {
            current = stack.pop().unwrap();
            let unvisited_neighbors = maze
                .get_neighbors(current)
                .into_par_iter()
                .map(|cell| cell.get_coord())
                .filter(|cell| !visited.contains(cell))
                .collect::<Vec<_>>();

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
