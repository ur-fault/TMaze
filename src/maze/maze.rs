use crate::maze::cell::{Cell, CellWall};

pub struct Maze {
    pub(crate) cells: Vec<Vec<Cell>>,
    pub(crate) width: usize,
    pub(crate) height: usize,
}

impl Maze {
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
