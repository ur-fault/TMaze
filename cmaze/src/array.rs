use std::ops;

use crate::dims::{Dims, Dims3D};

#[derive(Debug, Clone)]
pub struct Array3D<T> {
    buf: Vec<T>,
    width: usize,
    height: usize,
    depth: usize,
}

impl<T> Array3D<T> {
    pub fn size(&self) -> Dims3D {
        Dims3D(self.width as i32, self.height as i32, self.depth as i32)
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn dim_to_idx(&self, pos: Dims3D) -> Option<usize> {
        let Dims3D(x, y, z) = pos;
        let (x, y, z) = (x as usize, y as usize, z as usize);

        if x >= self.width || y >= self.height || z >= self.depth {
            return None;
        }

        Some(z * self.width * self.height + y * self.width + x)
    }

    pub fn idx_to_dim(&self, idx: usize) -> Option<Dims3D> {
        if idx >= self.buf.len() {
            return None;
        }

        let x = idx % self.width;
        let y = (idx / self.width) % self.height;
        let z = idx / (self.width * self.height);

        Some(Dims3D(x as i32, y as i32, z as i32))
    }

    pub fn get(&self, pos: Dims3D) -> Option<&T> {
        self.dim_to_idx(pos).and_then(|i| self.buf.get(i))
    }
}

impl<T> Array3D<T> {
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.buf.iter()
    }

    pub fn iter_pos(&self) -> impl Iterator<Item = Dims3D> + use<'_, T> {
        (0..self.buf.len()).filter_map(move |i| self.idx_to_dim(i))
    }

    pub fn all(&self, f: impl Fn(&T) -> bool) -> bool {
        self.buf.iter().all(f)
    }

    pub fn map<U>(self, f: impl Fn(T) -> U) -> Array3D<U> {
        Array3D {
            buf: self.buf.into_iter().map(f).collect(),
            width: self.width,
            height: self.height,
            depth: self.depth,
        }
    }

    pub fn layer(&self, z: usize) -> Option<Array2DView<T>> {
        if z >= self.depth {
            return None;
        }

        let z = z as i32;
        let start = self.dim_to_idx(Dims3D(0, 0, z))?;
        let end = start + self.width * self.height;

        Some(Array2DView {
            buf: &self.buf[start..end],
            width: self.width,
            height: self.height,
        })
    }

    pub fn mask(self, mask: &Array3D<bool>) -> Option<Array3D<Option<T>>> {
        if self.size() != mask.size() {
            return None;
        }

        Some(Array3D {
            buf: self
                .buf
                .into_iter()
                .zip(mask.buf.iter())
                .map(|(item, mask)| if *mask { Some(item) } else { None })
                .collect(),
            width: self.width,
            height: self.height,
            depth: self.depth,
        })
    }
}

impl<T: Clone> Array3D<T> {
    pub fn new(item: T, width: usize, height: usize, depth: usize) -> Self {
        Self {
            buf: vec![item.clone(); width * height * depth],
            width,
            height,
            depth,
        }
    }

    pub fn new_dims(item: T, size: Dims3D) -> Option<Self> {
        if !size.all_non_negative() {
            return None;
        }
        Some(Self::new(
            item,
            size.0 as usize,
            size.1 as usize,
            size.2 as usize,
        ))
    }
}

impl<T> ops::Index<Dims3D> for Array3D<T> {
    type Output = T;

    fn index(&self, index: Dims3D) -> &Self::Output {
        self.dim_to_idx(index)
            .and_then(|i| self.buf.get(i))
            .expect("Index out of bounds")
    }
}

impl<T> ops::IndexMut<Dims3D> for Array3D<T> {
    fn index_mut(&mut self, index: Dims3D) -> &mut Self::Output {
        self.dim_to_idx(index)
            .and_then(|i| self.buf.get_mut(i))
            .expect("Index out of bounds")
    }
}

pub struct Array2DView<'a, T> {
    buf: &'a [T],
    width: usize,
    height: usize,
}

impl<'a, T> Array2DView<'a, T> {
    pub fn size(&self) -> Dims {
        Dims(self.width as i32, self.height as i32)
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn dim_to_idx(&self, pos: Dims) -> Option<usize> {
        let Dims(x, y) = pos;
        let (x, y) = (x as usize, y as usize);

        if x >= self.width || y >= self.height {
            return None;
        }

        Some(y * self.width + x)
    }

    pub fn idx_to_dim(&self, idx: usize) -> Option<Dims> {
        if idx >= self.buf.len() {
            return None;
        }

        let x = idx % self.width;
        let y = idx / self.width;

        Some(Dims(x as i32, y as i32))
    }

    pub fn get(&self, pos: Dims) -> Option<&T> {
        self.dim_to_idx(pos).and_then(|i| self.buf.get(i))
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.buf.iter()
    }

    pub fn iter_pos(&self) -> impl Iterator<Item = Dims> + '_ {
        (0..self.buf.len()).filter_map(move |i| self.idx_to_dim(i))
    }
}

impl<T> ops::Index<Dims> for Array2DView<'_, T> {
    type Output = T;

    fn index(&self, index: Dims) -> &Self::Output {
        self.dim_to_idx(index)
            .and_then(|i| self.buf.get(i))
            .expect("Index out of bounds")
    }
}
