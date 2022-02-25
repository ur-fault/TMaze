use crate::game::game::Error;
use crate::maze::cell::{Cell, CellWall};
use rand::seq::SliceRandom;

pub struct Maze {
    cells: Vec<Vec<Cell>>,
    width: usize,
    height: usize,
}

impl Maze {
    pub fn new_dfs<T: FnMut(usize) -> Result<(), Error>>(
        w: usize,
        h: usize,
        start_: Option<(usize, usize)>,
        mut report_progress: Option<T>,
    ) -> Result<Maze, Error> {
        let mut visited: Vec<(usize, usize)> = Vec::with_capacity(w * h);
        let mut stack: Vec<(usize, usize)> = Vec::with_capacity(w * h);

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
                let chosen_wall = Self::which_wall(current, chosen);
                maze.remove_wall(current, chosen_wall);
                visited.push(chosen);
                stack.push(chosen);
            }

            if let Some(_) = report_progress {
                report_progress.as_mut().unwrap()(visited.len())?;
            }
        }

        Ok(maze)
    }

    pub fn size(&self) -> (usize, usize) {
        (self.cells[0].len(), self.cells.len())
    }

    pub fn is_in_bounds(&self, x: isize, y: isize) -> bool {
        0 <= x && x < self.width as isize && 0 <= y && y < self.height as isize
    }

    pub fn is_valid_neighbor(&self, cell: (usize, usize), off_x: isize, off_y: isize) -> bool {
        (off_x == -1 || off_x == 1 || off_x == 0)
            && (off_y == -1 || off_y == 1 || off_y == 0)
            && ((off_x == 0) ^ (off_y == 0))
            && self.is_in_bounds(cell.0 as isize, cell.1 as isize)
            && self.is_in_bounds(cell.0 as isize + off_x, cell.1 as isize + off_y)
    }

    pub fn is_valid_wall(&self, cell: (usize, usize), wall: CellWall) -> bool {
        let neighbor_offset = wall.to_coord();
        self.is_valid_neighbor(cell, neighbor_offset.0, neighbor_offset.1)
    }

    pub fn which_wall(cell: (usize, usize), cell2: (usize, usize)) -> CellWall {
        match (
            cell.0 as isize - cell2.0 as isize,
            cell.1 as isize - cell2.1 as isize,
        ) {
            (-1, 0) => CellWall::Right,
            (1, 0) => CellWall::Left,
            (0, -1) => CellWall::Bottom,
            (0, 1) => CellWall::Top,
            _ => panic!(),
        }
    }

    pub fn get_neighbors(&self, cell: (usize, usize)) -> Vec<&Cell> {
        let offsets: [(isize, isize); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        offsets
            .into_iter()
            .filter(|(x, y)| self.is_valid_neighbor(cell, *x, *y))
            .map(|(x, y)| {
                &self.cells[(cell.1 as isize + y) as usize][(cell.0 as isize + x) as usize]
            })
            .collect()
    }

    pub fn remove_wall(&mut self, cell: (usize, usize), wall: CellWall) {
        if !self.is_valid_wall(cell, wall) {
            return;
        }

        self.cells[cell.1][cell.0].remove_wall(wall);
        let neighbor_offset = wall.to_coord();
        {
            let x2 = (cell.0 as isize + neighbor_offset.0) as usize;
            let y2 = (cell.1 as isize + neighbor_offset.1) as usize;
            self.cells[y2][x2].remove_wall(wall.reverse_wall());
        }
    }

    pub fn get_cells(&self) -> &[Vec<Cell>] {
        &self.cells
    }
}
