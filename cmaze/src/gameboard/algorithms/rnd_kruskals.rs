use self::Passage::{Bottom, Down, Left, Right, Up};
use super::super::cell::{Cell, Passage};
use super::{
    GenerationErrorInstant, GenerationErrorThreaded, Maze, MazeAlgorithm, Progress,
    StopGenerationFlag,
};
use crate::core::*;
use crate::gameboard::cell::Portal;
use crossbeam::channel::Sender;
use rand::Rng;
use rand::{seq::SliceRandom, thread_rng};

#[cfg(feature = "hashbrown")]
use hashbrown::HashSet;
#[cfg(not(feature = "hashbrown"))]
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

        let cells: Vec<Vec<Vec<Cell>>> = vec![vec![Vec::with_capacity(wu); hu]; du];

        let mut maze = Maze {
            cells,
            width: wu,
            height: hu,
            depth: du,
        };

        for z in 0..d {
            for y in 0..h {
                for x in 0..w {
                    maze.cells[z as usize][y as usize].push(Cell::new(Dims3D(x, y, z)));
                }
            }
        }

        let mut passages: Vec<(Dims3D, Passage)> = Vec::new();

        // Generate all walls
        for (iz, floor) in maze.cells.iter().enumerate() {
            for (iy, row) in floor.iter().enumerate() {
                for ix in 0..row.len() {
                    if ix != wu - 1 {
                        passages.push((Dims3D(ix as i32, iy as i32, iz as i32), Right));
                    }

                    if iy != hu - 1 {
                        passages.push((Dims3D(ix as i32, iy as i32, iz as i32), Bottom));
                    }

                    if iz != du - 1 {
                        passages.push((Dims3D(ix as i32, iy as i32, iz as i32), Up));
                    }
                }
            }
        }

        // Generate portals, which are two cells that are connected
        {
            // Generate grid of cells-like, tracking if they are part of a portal
            let mut portal_grid = vec![vec![vec![false; wu]; hu]; du];

            let mut portal_count = 0;

            loop {
                portal_count += 1;

                if portal_count >= cell_count / 2 {
                    break;
                }

                let (x0, y0, z0) = (
                    thread_rng().gen_range(0..wu),
                    thread_rng().gen_range(0..hu),
                    thread_rng().gen_range(0..du),
                );

                if portal_grid[z0][y0][x0] {
                    continue;
                }

                let (x1, y1, z1) = (
                    thread_rng().gen_range(0..wu),
                    thread_rng().gen_range(0..hu),
                    thread_rng().gen_range(0..du),
                );

                if portal_grid[z1][y1][x1] {
                    continue;
                }

                if (x0, y0, z0) == (x1, y1, z1) {
                    continue;
                }

                let cell0 = Dims3D(x0 as i32, y0 as i32, z0 as i32);
                let cell1 = Dims3D(x1 as i32, y1 as i32, z1 as i32);

                let rel_cell = cell1 - cell0;

                // Check that cells are *NOT* adjacent
                if rel_cell.0.abs() + rel_cell.1.abs() + rel_cell.2.abs() == 1 {
                    continue;
                }

                let passage = Passage::Portal(Portal {
                    other: cell1,
                    id: portal_count,
                });

                passages.push((cell0, passage));

                // Mark the cells as part of a portal
                portal_grid[z0][y0][x0] = true;
                portal_grid[z1][y1][x1] = true;
            }
        }

        let passage_count = passages.len();

        // In Randomized Kruskal's, we check if the two cells are in the same set
        // If they are, we skip the passage. So, we generate a set for each cell
        // This set is needed because we don't allow loops in the maze
        let mut sets = Vec::<HashSet<Dims3D>>::with_capacity(cell_count);
        for iz in 0..maze.cells.len() {
            for iy in 0..maze.cells[0].len() {
                for ix in 0..maze.cells[0][0].len() {
                    sets.push(
                        vec![Dims3D(ix as i32, iy as i32, iz as i32)]
                            .into_iter()
                            .collect(),
                    );
                }
            }
        }

        passages.shuffle(&mut thread_rng());
        while let Some((passage_start, passage)) = passages.pop() {
            let passage_end = passage.end(passage_start);

            let set0_i = sets
                .iter()
                .position(|set| set.contains(&passage_start))
                .unwrap();

            if sets[set0_i].contains(&passage_end) {
                continue;
            }

            let set1_i = sets
                .iter()
                .position(|set| set.contains(&passage_end))
                .unwrap();

            maze.get_cell_mut(passage_start)
                .unwrap()
                .make_passage(passage);

            maze.get_cell_mut(passage_end)
                .unwrap()
                .make_passage(passage.reverse(passage_start));

            let set0 = sets.swap_remove(set0_i);

            let set1_i = if set1_i == sets.len() - 1 {
                sets.len() - 1
            } else {
                sets.iter()
                    .position(|set| set.contains(&passage_end))
                    .unwrap()
            };

            sets[set1_i].extend(set0);

            progress
                .send(Progress {
                    done: passage_count - passages.len(),
                    from: passage_count,
                })
                .unwrap();

            if stopper.is_stopped() {
                return Err(GenerationErrorThreaded::AbortGeneration);
            }
        }

        Ok(maze)
    }
}
