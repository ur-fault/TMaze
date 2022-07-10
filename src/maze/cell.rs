use crate::maze::cell::CellWall::*;
use crate::core::*;

#[derive(Clone)]
pub struct Cell {
    left: bool,
    top: bool,
    right: bool,
    bottom: bool,
    up: bool,
    down: bool,
    coord: Dims3D,
}

impl Cell {
    pub fn new(pos: Dims3D) -> Cell {
        Cell {
            left: true,
            right: true,
            top: true,
            bottom: true,
            up: true,
            down: true,
            coord: pos,
        }
    }

    pub fn remove_wall(&mut self, wall: CellWall) {
        match wall {
            Left => self.left = false,
            Top => self.top = false,
            Right => self.right = false,
            Bottom => self.bottom = false,
            Up => self.up = false,
            Down => self.down = false,
        }
    }

    pub fn get_wall(&self, wall: CellWall) -> bool {
        match wall {
            Left => self.left,
            Top => self.top,
            Right => self.right,
            Bottom => self.bottom,
            Up => self.up,
            Down => self.down,
        }
    }

    pub fn get_coord(&self) -> Dims3D {
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
    Up,
    Down,
}

impl CellWall {
    pub fn to_coord(&self) -> Dims3D {
        match self {
            Self::Left => (-1, 0, 0),
            Self::Right => (1, 0, 0),
            Self::Top => (0, -1, 0),
            Self::Bottom => (0, 1, 0),
            Self::Up => (0, 0, 1),
            Self::Down => (0, 0, -1),
        }
    }

    pub fn reverse_wall(&self) -> CellWall {
        match self {
            Left => Right,
            Right => Left,
            Top => Bottom,
            Bottom => Top,
            Up => Down,
            Down => Up,
        }
    }

    pub fn perpendicular_walls(&self) -> (CellWall, CellWall, CellWall, CellWall) {
        match self {
            Left | Right => (Top, Bottom, Up, Down),
            Top | Bottom => (Left, Right, Up, Down),
            Up | Down => (Top, Bottom, Left, Right),
        }
    }
}
