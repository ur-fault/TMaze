use rand::{seq::SliceRandom, thread_rng};

use std::sync::{Arc, Mutex};

use super::{
    super::cell::{Cell, CellWall},
    CellMask, GenErrorInstant, GenErrorThreaded, GroupGenerator, Maze, MazeAlgorithm, Progress,
    StopGenerationFlag,
};
use crate::{array::Array3D, dims::*};

use CellWall::*;

use hashbrown::HashSet;

pub struct RndKruskals {}
impl MazeAlgorithm for RndKruskals {
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

        let cells = Array3D::new(Cell::new(), wu, hu, du);
        let mut sets = Vec::<HashSet<Dims3D>>::with_capacity(cell_count);

        let wall_count = (hu * (wu - 1) + wu * (hu - 1)) * du + wu * hu * (du - 1);
        let mut walls: Vec<(Dims3D, CellWall)> = Vec::with_capacity(wall_count);
        progress.lock().unwrap().from = wall_count;

        for pos @ Dims3D(x, y, z) in cells.iter_pos() {
            if x as usize != wu - 1 {
                walls.push((pos, Right));
            }

            if y as usize != hu - 1 {
                walls.push((pos, Bottom));
            }

            if z as usize != du - 1 {
                walls.push((pos, Up));
            }

            sets.push(vec![pos].into_iter().collect());
        }

        let mut maze = Maze {
            cells,
            width: w as usize,
            height: h as usize,
            depth: d as usize,
            is_tower: false,
        };

        walls.shuffle(&mut thread_rng());
        while let Some((from, wall)) = walls.pop() {
            let to = from + wall.to_coord();

            let from_set = sets
                .iter()
                .position(|set| set.contains(&from))
                .expect("cant find set0");

            if sets[from_set].contains(&to) {
                continue;
            }

            maze.get_cell_mut(from).unwrap().remove_wall(wall);
            maze.get_cell_mut(to)
                .unwrap()
                .remove_wall(wall.reverse_wall());
            let from_set = sets.swap_remove(from_set);

            let to_set = sets.iter().position(|set| set.contains(&to)).unwrap();
            sets[to_set].extend(from_set);

            progress.lock().unwrap().done = wall_count - walls.len();

            if stopper.is_stopped() {
                return Err(GenErrorThreaded::AbortGeneration);
            }
        }

        progress.lock().unwrap().is_done = true;

        Ok(maze)
    }
}

impl GroupGenerator for RndKruskals {
    fn generate(&self, mask: CellMask) -> Maze {
        let Dims3D(w, h, d) = mask.size();
        let (wu, hu, du) = (w as usize, h as usize, d as usize);

        let mut walls: Vec<(Dims3D, CellWall)> = Vec::new();
        let mut sets = Vec::<HashSet<Dims3D>>::new();
        for z in 0..mask.depth {
            for y in 0..mask.height {
                for x in 0..mask.width {
                    let pos = (x, y, z).into();
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
                    }

                    sets.push(vec![pos].into_iter().collect());
                }
            }
        }

        let cells: Array3D<Cell> = Array3D::new(Cell::new(), wu, hu, du);

        let mut maze = Maze {
            cells,
            width: w as usize,
            height: h as usize,
            depth: d as usize,
            is_tower: false,
        };

        walls.shuffle(&mut thread_rng());
        while let Some((from, wall)) = walls.pop() {
            let to = from + wall.to_coord();

            let from_set = sets.iter().position(|set| set.contains(&from)).unwrap();

            if sets[from_set].contains(&to) {
                continue;
            }

            maze.get_cell_mut(from).unwrap().remove_wall(wall);
            maze.get_cell_mut(to)
                .unwrap()
                .remove_wall(wall.reverse_wall());
            let from_set = sets.swap_remove(from_set);

            let to_set = sets.iter().position(|set| set.contains(&to)).unwrap();
            sets[to_set].extend(from_set);
        }

        todo!()
    }
}
