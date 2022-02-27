use super::super::cell::{Cell, CellWall};
use super::{Maze, MazeAlgorithm};
use crate::game::{Dims, Error};
use rand::{seq::SliceRandom, thread_rng};
use std::collections::HashSet;

pub struct RndKruskals {}

impl MazeAlgorithm for RndKruskals {
    fn new<T: FnMut(usize, usize) -> Result<(), Error>>(
        w: usize,
        h: usize,
        start_: Option<(usize, usize)>,
        mut report_progress: Option<T>,
    ) -> Result<Maze, Error> {
        let cell_count = w * h;
        let mut cells: Vec<Vec<Cell>> = vec![Vec::with_capacity(w); h];
        for y in 0..h {
            for x in 0..w {
                cells[y].push(Cell::new(x, y));
            }
        }

        let wall_count = h * (w - 1) + w * (h - 1);
        let mut walls: Vec<(Dims, CellWall)> = Vec::with_capacity(wall_count);

        for (iy, row) in cells.iter().enumerate() {
            for ix in 0..row.len() {
                if iy == h - 1 && ix == w - 1 {
                    continue;
                } else if iy == h - 1 {
                    walls.push(((ix as i32, iy as i32), CellWall::Right));
                } else if ix == w - 1 {
                    walls.push(((ix as i32, iy as i32), CellWall::Bottom));
                } else {
                    walls.push(((ix as i32, iy as i32), CellWall::Right));
                    walls.push(((ix as i32, iy as i32), CellWall::Bottom));
                }
            }
        }

        let mut sets = Vec::<HashSet<Dims>>::with_capacity(cell_count);
        for iy in 0..cells.len() {
            for ix in 0..cells[0].len() {
                sets.push(vec![(ix as i32, iy as i32)].into_iter().collect());
            }
        }

        walls.shuffle(&mut thread_rng());
        while let Some(((ix0, iy0), wall)) = walls.pop() {
            let (ix1, iy1) = (
                (wall.to_coord().0 + ix0 as isize) as i32,
                (wall.to_coord().1 + iy0 as isize) as i32,
            );

            let set0_i = sets
                .iter()
                .position(|set| set.contains(&(ix0, iy0)))
                .unwrap();
            let set1_i = sets
                .iter()
                .position(|set| set.contains(&(ix1, iy1)))
                .unwrap();

            if set0_i == set1_i {
                continue;
            }

            cells[iy0 as usize][ix0 as usize].remove_wall(wall);
            cells[iy1 as usize][ix1 as usize].remove_wall(wall.reverse_wall());
            let set0 = sets.swap_remove(set0_i);

            let set1_i = if set1_i == sets.len() - 1 {
                sets.len() - 1
            } else {
                sets.iter()
                    .position(|set| set.contains(&(ix1, iy1)))
                    .unwrap()
            };
            sets[set1_i].extend(set0);

            if let Some(_) = report_progress {
                report_progress.as_mut().unwrap()(wall_count - walls.len(), wall_count)?;
            }
        }

        Ok(Maze {
            cells,
            width: w,
            height: h,
        })
    }
}
