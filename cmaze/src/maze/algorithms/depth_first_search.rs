use std::thread;

use super::super::cell::Cell;
use super::{
    GenerationErrorInstant, GenerationErrorThreaded, Maze, MazeAlgorithm,
    MazeGeneratorComunication, StopGenerationFlag,
};
use crate::core::*;
use crate::maze::CellWall;
use crossbeam::channel::{unbounded, Sender};
use crossbeam::scope;
use rand::{seq::SliceRandom, thread_rng, Rng};
use rayon::prelude::*;

pub struct DepthFirstSearch {}

impl MazeAlgorithm for DepthFirstSearch {
    fn generate(
        size: Dims3D,
        floored: bool,
    ) -> Result<MazeGeneratorComunication, GenerationErrorInstant> {
        if size.0 == 0 || size.1 == 0 || size.2 == 0 {
            return Err(GenerationErrorInstant::InvalidSize(size));
        }

        let stop_flag = StopGenerationFlag::new();
        let (s_progress, r_progress) = unbounded::<(usize, usize)>();

        let Dims3D(w, h, d) = size;
        let (wu, hu, du) = (w as usize, h as usize, d as usize);

        let stop_flag_clone = stop_flag.clone();

        Ok((
            thread::spawn(move || {
                Ok(Maze {
                    cells: if size.2 > 1 && floored {
                        let mut cells: Vec<Vec<Vec<Cell>>> = (0..du)
                            .map(|maze_i| {
                                let (s, r) = unbounded::<(usize, usize)>();

                                let s_progress = s_progress.clone();
                                let stop_flag = stop_flag.clone();
                                match scope(|scope| {
                                    scope.spawn(move |_| {
                                        for (done, from) in r.iter() {
                                            s_progress
                                                .send((done + maze_i * from, from * du))
                                                .unwrap();
                                        }
                                    });

                                    if stop_flag.is_stopped() {
                                        return Err(GenerationErrorThreaded::AbortGeneration);
                                    }

                                    Self::generate_individual(Dims3D(w, h, 1), stop_flag, s)
                                })
                                .map(
                                    |res| -> Result<Vec<Vec<Cell>>, GenerationErrorThreaded> {
                                        Ok(res?.cells.remove(0))
                                    },
                                ) {
                                    Ok(Ok(maze)) => Ok(maze),
                                    Err(e) => Err(GenerationErrorThreaded::UnknownError(e)),
                                    Ok(Err(e)) => Err(e),
                                }
                            })
                            .collect::<Result<Vec<Vec<Vec<Cell>>>, GenerationErrorThreaded>>()?;

                        for floor in 0..du - 1 {
                            let (x, y) =
                                (thread_rng().gen_range(0..wu), thread_rng().gen_range(0..hu));
                            cells[floor][y][x].remove_wall(CellWall::Up);
                            cells[floor + 1][y][x].remove_wall(CellWall::Down);
                        }

                        cells
                    } else {
                        Self::generate_individual(
                            Dims3D(w, h, d),
                            stop_flag.clone(),
                            s_progress.clone(),
                        )?
                        .cells
                    },
                    width: wu,
                    height: hu,
                    depth: du,
                })
            }),
            stop_flag_clone,
            r_progress,
        ))
    }

    fn generate_individual(
        size: Dims3D,
        stopper: StopGenerationFlag,
        progress: Sender<(usize, usize)>,
    ) -> Result<Maze, GenerationErrorThreaded> {
        if size.0 == 0 || size.1 == 0 || size.2 == 0 {
            return Err(GenerationErrorThreaded::GenerationError(
                GenerationErrorInstant::InvalidSize(size),
            ));
        }
        let Dims3D(w, h, d) = size;
        let (wu, hu, du) = (w as usize, h as usize, d as usize);
        let cell_count = wu * hu * du;

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
        };

        let mut current = Dims3D(sx, sy, sz);
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

            progress.send((visited.len(), cell_count)).unwrap();

            if stopper.is_stopped() {
                return Err(GenerationErrorThreaded::AbortGeneration);
            }
        }

        Ok(maze)
    }
}
