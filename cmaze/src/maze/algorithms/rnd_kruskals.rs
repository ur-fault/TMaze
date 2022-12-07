use self::CellWall::*;
use super::super::cell::{Cell, CellWall};
use super::{
    GenerationErrorInstant, GenerationErrorThreaded, Maze, MazeAlgorithm, Progress,
    StopGenerationFlag,
};
use crate::core::*;
use crossbeam::channel::Sender;
use rand::{seq::SliceRandom, thread_rng};
use std::collections::HashSet;

pub struct RndKruskals {}
impl MazeAlgorithm for RndKruskals {
    fn generate_individual(
        size: Dims3D,
        stopper: StopGenerationFlag,
        progress: Sender<Progress>,
    ) -> Result<Maze, GenerationErrorThreaded> {
        if size.0 == 0 || size.1 == 0 || size.2 == 0 {
            return Err(GenerationErrorThreaded::GenerationError(
                GenerationErrorInstant::InvalidSize(size),
            ));
        }

        let Dims3D(w, h, d) = size;
        let (wu, hu, du) = (w as usize, h as usize, d as usize);
        let cell_count = wu * hu * du;

        let mut cells: Vec<Vec<Vec<Cell>>> = vec![vec![Vec::with_capacity(wu); hu]; du];

        for z in 0..d {
            for y in 0..h {
                for x in 0..w {
                    cells[z as usize][y as usize].push(Cell::new(Dims3D(x, y, z)));
                }
            }
        }

        let wall_count = (hu * (wu - 1) + wu * (hu - 1)) * du + wu * hu * (du - 1);
        let mut walls: Vec<(Dims3D, CellWall)> = Vec::with_capacity(wall_count);

        for (iz, floor) in cells.iter().enumerate() {
            for (iy, row) in floor.iter().enumerate() {
                for ix in 0..row.len() {
                    if ix != wu - 1 {
                        walls.push((Dims3D(ix as i32, iy as i32, iz as i32), Right));
                    }

                    if iy != hu - 1 {
                        walls.push((Dims3D(ix as i32, iy as i32, iz as i32), Bottom));
                    }

                    if iz != du - 1 {
                        walls.push((Dims3D(ix as i32, iy as i32, iz as i32), Up));
                    }
                }
            }
        }

        let mut sets = Vec::<HashSet<Dims3D>>::with_capacity(cell_count);
        for iz in 0..cells.len() {
            for iy in 0..cells[0].len() {
                for ix in 0..cells[0][0].len() {
                    sets.push(
                        vec![Dims3D(ix as i32, iy as i32, iz as i32)]
                            .into_iter()
                            .collect(),
                    );
                }
            }
        }

        let mut maze = Maze {
            cells,
            width: w as usize,
            height: h as usize,
            depth: d as usize,
        };

        walls.shuffle(&mut thread_rng());
        while let Some((pos0, wall)) = walls.pop() {
            let pos1 = pos0 + wall.to_coord();

            let set0_i = sets.iter().position(|set| set.contains(&pos0)).unwrap();

            if sets[set0_i].contains(&pos1) {
                continue;
            }

            let set1_i = sets.iter().position(|set| set.contains(&pos1)).unwrap();

            maze.get_cell_mut(pos0).unwrap().remove_wall(wall);
            maze.get_cell_mut(pos1)
                .unwrap()
                .remove_wall(wall.reverse_wall());
            let set0 = sets.swap_remove(set0_i);

            let set1_i = if set1_i == sets.len() - 1 {
                sets.len() - 1
            } else {
                sets.iter().position(|set| set.contains(&pos1)).unwrap()
            };
            sets[set1_i].extend(set0);

            progress
                .send(Progress {
                    done: wall_count - walls.len(),
                    from: wall_count,
                })
                .unwrap();

            if stopper.is_stopped() {
                return Err(GenerationErrorThreaded::AbortGeneration);
            }
        }

        Ok(maze)
    }
}
