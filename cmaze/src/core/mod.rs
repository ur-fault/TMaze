use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dims(pub i32, pub i32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dims3D(pub i32, pub i32, pub i32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DimsU(pub usize, pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub struct GameMode(pub i32, pub i32, pub i32, pub bool);
pub struct GameMode {
    pub size: Dims3D,
    pub is_tower: bool,
}
impl Add for Dims {
    type Output = Dims;

    fn add(self, other: Dims) -> Dims {
        Dims(self.0 + other.0, self.1 + other.1)
    }
}

impl Sub for Dims {
    type Output = Dims;

    fn sub(self, other: Dims) -> Dims {
        Dims(self.0 - other.0, self.1 - other.1)
    }
}

impl AddAssign for Dims {
    fn add_assign(&mut self, other: Dims) {
        self.0 += other.0;
        self.1 += other.1;
    }
}

impl SubAssign for Dims {
    fn sub_assign(&mut self, other: Dims) {
        self.0 -= other.0;
        self.1 -= other.1;
    }
}

impl From<(i32, i32)> for Dims {
    fn from(tuple: (i32, i32)) -> Self {
        Dims(tuple.0, tuple.1)
    }
}

impl Into<(i32, i32)> for Dims {
    fn into(self) -> (i32, i32) {
        (self.0, self.1)
    }
}

impl Add for Dims3D {
    type Output = Dims3D;

    fn add(self, other: Dims3D) -> Dims3D {
        Dims3D(self.0 + other.0, self.1 + other.1, self.2 + other.2)
    }
}

impl Sub for Dims3D {
    type Output = Dims3D;

    fn sub(self, other: Dims3D) -> Dims3D {
        Dims3D(self.0 - other.0, self.1 - other.1, self.2 - other.2)
    }
}

impl AddAssign for Dims3D {
    fn add_assign(&mut self, other: Dims3D) {
        self.0 += other.0;
        self.1 += other.1;
        self.2 += other.2;
    }
}

impl SubAssign for Dims3D {
    fn sub_assign(&mut self, other: Dims3D) {
        self.0 -= other.0;
        self.1 -= other.1;
        self.2 -= other.2;
    }
}

impl From<(i32, i32, i32)> for Dims3D {
    fn from(tuple: (i32, i32, i32)) -> Self {
        Dims3D(tuple.0, tuple.1, tuple.2)
    }
}

impl Into<(i32, i32, i32)> for Dims3D {
    fn into(self) -> (i32, i32, i32) {
        (self.0, self.1, self.2)
    }
}

impl Add for DimsU {
    type Output = DimsU;

    fn add(self, other: DimsU) -> DimsU {
        DimsU(self.0 + other.0, self.1 + other.1)
    }
}

impl Sub for DimsU {
    type Output = DimsU;

    fn sub(self, other: DimsU) -> DimsU {
        DimsU(self.0 - other.0, self.1 - other.1)
    }
}

impl AddAssign for DimsU {
    fn add_assign(&mut self, other: DimsU) {
        self.0 += other.0;
        self.1 += other.1;
    }
}

impl SubAssign for DimsU {
    fn sub_assign(&mut self, other: DimsU) {
        self.0 -= other.0;
        self.1 -= other.1;
    }
}

impl From<(usize, usize)> for DimsU {
    fn from(tuple: (usize, usize)) -> Self {
        DimsU(tuple.0, tuple.1)
    }
}

impl Into<(usize, usize)> for DimsU {
    fn into(self) -> (usize, usize) {
        (self.0, self.1)
    }
}