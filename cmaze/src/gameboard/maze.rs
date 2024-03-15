use self::Passage::*;
use crate::core::*;
use crate::gameboard::cell::{Cell, Passage};

#[derive(Debug, Clone)]
pub struct Maze {
    pub(crate) cells: Vec<Vec<Vec<Cell>>>,
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) depth: usize,
}

impl Maze {
    pub fn new(cells: Vec<Vec<Vec<Cell>>>) -> Self {
        let width = cells[0][0].len();
        let height = cells[0].len();
        let depth = cells.len();
        Maze {
            cells,
            width,
            height,
            depth,
        }
    }

    pub fn size(&self) -> Dims3D {
        Dims3D(self.width as i32, self.height as i32, self.depth as i32)
    }

    pub fn is_in_bounds(&self, pos: Dims3D) -> bool {
        0 <= pos.0
            && pos.0 < self.width as i32
            && 0 <= pos.1
            && pos.1 < self.height as i32
            && 0 <= pos.2
            && pos.2 < self.depth as i32
    }

    pub fn is_valid_neighbor(&self, cell: Dims3D, off: Dims3D) -> bool {
        off.abs_sum() == 1
            && self.is_in_bounds(cell)
            && self.is_in_bounds(Dims3D(cell.0 + off.0, cell.1 + off.1, cell.2 + off.2))
    }

    pub fn is_valid_passage(&self, cell: Dims3D, passage: Passage) -> bool {
        match passage {
            Portal(p) => self.is_in_bounds(p.other),
            _ => self.is_valid_neighbor(cell, passage.offset().unwrap()),
        }
    }

    /// Returns the wall between two cells, if it exists
    /// If the cells are not adjacent, returns None
    ///
    /// *Note*: This function doesn't use portal and
    /// and will *NOT* return a wall if the cells are connected by a portal
    pub fn which_wall_between(cell: Dims3D, cell2: Dims3D) -> Option<Passage> {
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

    // TODO: Maybe use `smallvec` for this function, who knows
    pub fn get_neighbors(&self, cell: Dims3D) -> Vec<&Cell> {
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
            .map(|off| {
                let pos = cell + off;
                &self.cells[pos.2 as usize][pos.1 as usize][pos.0 as usize]
            })
            .collect()
    }

    pub fn add_passage(&mut self, cell: Dims3D, passage: Passage) {
        if !self.is_valid_passage(cell, passage) {
            return;
        }

        self.cells[cell.2 as usize][cell.1 as usize][cell.0 as usize].make_passage(passage);
        // let neighbor_offset = passage.offset();
        // {
        //     let x2 = (cell.0 + neighbor_offset.0) as usize;
        //     let y2 = (cell.1 + neighbor_offset.1) as usize;
        //     let z2 = (cell.2 + neighbor_offset.2) as usize;
        //     self.cells[z2][y2][x2].make_passage(passage.reverse_passage());
        // }
        if let Some(neighbor_offset) = passage.offset() {
            let x2 = (cell.0 + neighbor_offset.0) as usize;
            let y2 = (cell.1 + neighbor_offset.1) as usize;
            let z2 = (cell.2 + neighbor_offset.2) as usize;
            self.cells[z2][y2][x2].make_passage(passage.reverse_passage().unwrap());
        } else {
            let pos = passage.portal_end().unwrap();
            self.get_cell_mut(pos)
                .unwrap()
                .make_passage(passage.reverse_passage().unwrap());
        }
    }

    pub fn get_cells(&self) -> &[Vec<Vec<Cell>>] {
        &self.cells
    }

    pub fn get_cell(&self, pos: Dims3D) -> Option<&Cell> {
        if self.is_in_bounds(pos) {
            Some(&self.cells[pos.2 as usize][pos.1 as usize][pos.0 as usize])
        } else {
            None
        }
    }

    pub fn get_cell_mut(&mut self, pos: Dims3D) -> Option<&mut Cell> {
        if self.is_in_bounds(pos) {
            Some(&mut self.cells[pos.2 as usize][pos.1 as usize][pos.0 as usize])
        } else {
            None
        }
    }
}
