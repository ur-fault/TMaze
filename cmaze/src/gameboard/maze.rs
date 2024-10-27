use self::CellWall::*;
use crate::{
    array::Array3D,
    dims::*,
    gameboard::cell::{Cell, CellWall},
};

#[derive(Clone)]
pub struct Maze {
    // pub(crate) cells: Vec<Vec<Vec<Cell>>>,
    pub(crate) cells: Array3D<Cell>,
    pub(crate) is_tower: bool,
}

impl Maze {
    pub fn size(&self) -> Dims3D {
        self.cells.size()
    }

    pub fn is_in_bounds(&self, pos: Dims3D) -> bool {
        0 <= pos.0
            && pos.0 < self.size().0
            && 0 <= pos.1
            && pos.1 < self.size().1
            && 0 <= pos.2
            && pos.2 < self.size().2
    }

    pub fn is_valid_neighbor(&self, cell: Dims3D, off: Dims3D) -> bool {
        (off.0 == -1 || off.0 == 1 || off.0 == 0)
            && (off.1 == -1 || off.1 == 1 || off.1 == 0)
            && (off.2 == -1 || off.2 == 1 || off.2 == 0)
            && ((off.0 == 1 || off.0 == -1) as u8
                + (off.1 == 1 || off.1 == -1) as u8
                + (off.2 == 1 || off.2 == -1) as u8)
                == 1
            && self.is_in_bounds(cell)
            && self.is_in_bounds(Dims3D(cell.0 + off.0, cell.1 + off.1, cell.2 + off.2))
    }

    pub fn is_valid_wall(&self, cell: Dims3D, wall: CellWall) -> bool {
        let neighbor_offset = wall.to_coord();
        self.is_valid_neighbor(cell, neighbor_offset)
    }

    pub fn which_wall_between(cell: Dims3D, cell2: Dims3D) -> Option<CellWall> {
        match (cell.0 - cell2.0, cell.1 - cell2.1, cell.2 - cell2.2) {
            (-1, 0, 0) => Some(Right),
            (1, 0, 0) => Some(Left),
            (0, -1, 0) => Some(Bottom),
            (0, 1, 0) => Some(Top),
            (0, 0, 1) => Some(Down),
            (0, 0, -1) => Some(Up),
            _ => None,
        }
    }

    pub fn get_wall(&self, from: Dims3D, wall: CellWall) -> Option<bool> {
        let to = from + wall.to_coord();
        if self.is_in_bounds(from) != self.is_in_bounds(to) {
            return Some(true);
        }

        Some(
            self.get_cell(from)
                .map(|c| c.get_wall(wall))
                .unwrap_or(false),
        )
    }

    pub fn get_neighbors_pos(&self, cell: Dims3D) -> Vec<Dims3D> {
        let offsets = [
            Dims3D(-1, 0, 0),
            Dims3D(1, 0, 0),
            Dims3D(0, -1, 0),
            Dims3D(0, 1, 0),
            Dims3D(0, 0, -1),
            Dims3D(0, 0, 1),
        ];
        offsets
            .into_iter()
            .filter(|off| self.is_valid_neighbor(cell, *off))
            .map(|off| Dims3D(cell.0 + off.0, cell.1 + off.1, cell.2 + off.2))
            .collect()
    }

    pub fn get_neighbors(&self, cell: Dims3D) -> Vec<&Cell> {
        self.get_neighbors_pos(cell)
            .into_iter()
            .filter_map(|pos| self.get_cell(pos))
            .collect()
    }

    pub fn remove_wall(&mut self, cell: Dims3D, wall: CellWall) {
        if !self.is_valid_wall(cell, wall) {
            return;
        }

        self.cells[cell].remove_wall(wall);
        self.cells[cell + wall.to_coord()].remove_wall(wall.reverse_wall());
    }

    pub fn get_cells(&self) -> &Array3D<Cell> {
        &self.cells
    }

    pub fn get_cell(&self, pos: Dims3D) -> Option<&Cell> {
        if self.is_in_bounds(pos) {
            Some(&self.cells[pos])
        } else {
            None
        }
    }

    pub fn get_cell_mut(&mut self, pos: Dims3D) -> Option<&mut Cell> {
        if self.is_in_bounds(pos) {
            Some(&mut self.cells[pos])
        } else {
            None
        }
    }

    pub fn is_tower(&self) -> bool {
        self.is_tower
    }
}
