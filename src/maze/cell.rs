use std::hash::{Hash, Hasher};

#[derive(Clone)]
pub struct Cell {
    left: bool,
    top: bool,
    right: bool,
    bottom: bool,
    coord: (usize, usize),
}

impl Cell {
    pub fn new(x: usize, y: usize) -> Cell {
        Cell {
            left: true,
            right: true,
            top: true,
            bottom: true,
            coord: (x, y),
        }
    }

    pub fn remove_wall(&mut self, wall: CellWall) {
        match wall {
            CellWall::Left => self.left = false,
            CellWall::Top => self.top = false,
            CellWall::Right => self.right = false,
            CellWall::Bottom => self.bottom = false,
        }
    }

    pub fn get_wall(&self, wall: CellWall) -> bool {
        match wall {
            CellWall::Left => self.left,
            CellWall::Top => self.top,
            CellWall::Right => self.right,
            CellWall::Bottom => self.bottom,
        }
    }

    pub fn get_coord(&self) -> (usize, usize) {
        self.coord
    }
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        self.coord == other.coord
    }
}

impl Eq for Cell {}

#[derive(Copy, Clone)]
pub enum CellWall {
    Left,
    Right,
    Top,
    Bottom,
}

impl CellWall {
    pub fn to_coord(&self) -> (isize, isize) {
        match self {
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
            Self::Top => (0, -1),
            Self::Bottom => (0, 1),
        }
    }

    pub fn reverse_wall(&self) -> CellWall {
        match self {
            CellWall::Left => CellWall::Right,
            CellWall::Right => CellWall::Left,
            CellWall::Top => CellWall::Bottom,
            CellWall::Bottom => CellWall::Top,
        }
    }

    pub fn perpendicular_walls(&self) -> (CellWall, CellWall) {
        match *self {
            Self::Left | Self::Right => (Self::Top, Self::Bottom),
            Self::Top | Self::Bottom => (Self::Left, Self::Right),
        }
    }
}
