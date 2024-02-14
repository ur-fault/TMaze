use crate::core::*;

#[derive(Debug, Clone, Copy)]
pub struct Portal {
    pub id: usize,
    pub other: Dims3D,
}

#[derive(Debug, Clone)]
pub struct Cell {
    left: bool,
    top: bool,
    right: bool,
    bottom: bool,
    up: bool,
    down: bool,
    portal: Option<Portal>,
    pub(crate) coord: Dims3D,
}

impl Cell {
    pub fn new(pos: Dims3D) -> Cell {
        Cell {
            left: false,
            right: false,
            top: false,
            bottom: false,
            up: false,
            down: false,
            portal: None,
            coord: pos,
        }
    }

    pub fn make_passage(&mut self, passage: Passage) {
        match passage {
            Passage::Left => self.left = true,
            Passage::Top => self.top = true,
            Passage::Right => self.right = true,
            Passage::Bottom => self.bottom = true,
            Passage::Up => self.up = true,
            Passage::Down => self.down = true,
            Passage::Portal(p) => self.portal = Some(p),
        }
    }

    /// Returns state of the passage in the given direction (way)
    ///
    /// Returns:
    /// - None if the passage is blocked
    /// - Some(None) if the passage is open
    /// - Some(Some(Portal)) if the passage is a portal
    pub fn get_way(&self, way: Way) -> Option<Option<Portal>> {
        match way {
            Way::Left => self.left.then(|| None),
            Way::Top => self.top.then(|| None),
            Way::Right => self.right.then(|| None),
            Way::Bottom => self.bottom.then(|| None),
            Way::Up => self.up.then(|| None),
            Way::Down => self.down.then(|| None),
            Way::Portal => self.portal.map(|p| Some(p)),
        }
    }

    pub fn get_portal(&self) -> Option<Portal> {
        self.portal
    }

    pub fn end_of_way(&self, way: Way) -> Option<Dims3D> {
        self.is_open(way).then(|| match way {
            Way::Portal => self.portal.unwrap().other,
            _ => self.coord + way.offset().unwrap(),
        })
    }

    pub fn is_open(&self, way: Way) -> bool {
        self.get_way(way).is_some()
    }

    pub fn is_closed(&self, way: Way) -> bool {
        !self.is_open(way)
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
pub enum Passage {
    Left,
    Right,
    Top,
    Bottom,
    Up,
    Down,
    Portal(Portal),
}

impl Passage {
    pub fn end(&self, start: Dims3D) -> Dims3D {
        match self {
            Self::Portal(p) => p.other,
            _ => start + self.to_way().offset().unwrap(),
        }
    }

    pub fn portal_end(&self) -> Option<Dims3D> {
        match self {
            Self::Portal(p) => Some(p.other),
            _ => None,
        }
    }

    pub fn reverse_passage(&self) -> Option<Self> {
        use Passage::*;

        match self {
            Left => Some(Right),
            Right => Some(Left),
            Top => Some(Bottom),
            Bottom => Some(Top),
            Up => Some(Down),
            Down => Some(Up),
            Portal { .. } => None,
        }
    }

    pub fn reverse(&self, pos: Dims3D) -> Self {
        match self {
            Self::Portal(p) => Self::Portal(Portal {
                id: p.id,
                other: pos,
            }),
            _ => self.reverse_passage().unwrap(),
        }
    }

    pub fn offset(&self) -> Option<Dims3D> {
        self.to_way().offset()
    }

    pub fn to_way(self) -> Way {
        self.into()
    }

    pub fn is_portal(&self) -> bool {
        self.to_way().is_portal()
    }

    pub fn as_portal(&mut self) -> Option<&mut Portal> {
        match self {
            Passage::Portal(p) => Some(p),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Way {
    Left,
    Right,
    Top,
    Bottom,
    Up,
    Down,
    Portal,
}

impl Way {
    pub fn offset(&self) -> Option<Dims3D> {
        match self {
            Self::Left => Some(Dims3D(-1, 0, 0)),
            Self::Right => Some(Dims3D(1, 0, 0)),
            Self::Top => Some(Dims3D(0, -1, 0)),
            Self::Bottom => Some(Dims3D(0, 1, 0)),
            Self::Up => Some(Dims3D(0, 0, 1)),
            Self::Down => Some(Dims3D(0, 0, -1)),
            Self::Portal => None,
        }
    }

    pub fn is_portal(&self) -> bool {
        matches!(self, Self::Portal)
    }

    pub fn reverse(&self) -> Option<Self> {
        match self {
            Self::Left => Some(Self::Right),
            Self::Right => Some(Self::Left),
            Self::Top => Some(Self::Bottom),
            Self::Bottom => Some(Self::Top),
            Self::Up => Some(Self::Down),
            Self::Down => Some(Self::Up),
            Self::Portal => None,
        }
    }

    /// Returns the walls that are perpendicular to the current wall
    ///
    /// *Note*: Portal is perpendicular to everything else, but nothing is perpendicular to Portal
    pub fn perpendicular_ways(&self) -> Option<[Way; 5]> {
        use Way::*;

        match self {
            Left | Right => Some([Top, Bottom, Up, Down, Portal]),
            Top | Bottom => Some([Left, Right, Up, Down, Portal]),
            Up | Down => Some([Top, Bottom, Left, Right, Portal]),
            Portal => None,
        }
    }
}

impl From<Passage> for Way {
    fn from(p: Passage) -> Self {
        match p {
            Passage::Left => Way::Left,
            Passage::Right => Way::Right,
            Passage::Top => Way::Top,
            Passage::Bottom => Way::Bottom,
            Passage::Up => Way::Up,
            Passage::Down => Way::Down,
            Passage::Portal { .. } => Way::Portal,
        }
    }
}
