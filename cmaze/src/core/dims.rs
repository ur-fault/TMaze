use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use num_traits::{One, Zero};
use paste::paste;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dims(pub i32, pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dims3D(pub i32, pub i32, pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DimsU(pub usize, pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dims3DU(pub usize, pub usize, pub usize);

pub trait DimsTrait<const SIZE: usize>:
    Sized
    + Copy
    + Into<[Self::Item; SIZE]>
    + From<[Self::Item; SIZE]>
    + Add
    + Add<Self::Item>
    + AddAssign
    + AddAssign<Self::Item>
    + Sub
    + Sub<Self::Item>
    + SubAssign
    + SubAssign<Self::Item>
    + Mul
    + Mul<Self::Item>
    + MulAssign
    + MulAssign<Self::Item>
    + Div
    + Div<Self::Item>
    + DivAssign
    + DivAssign<Self::Item>
{
    type Item: std::fmt::Debug;
    const COUNT: usize = SIZE;

    fn sum(&self) -> Self::Item
    where
        Self::Item: Add<Output = Self::Item> + Zero + Copy,
    {
        self.to_arr()
            .iter()
            .fold(Self::Item::zero(), |acc, &x| acc + x)
    }

    fn product(&self) -> Self::Item
    where
        Self::Item: Mul<Output = Self::Item> + One + Copy,
    {
        self.to_arr()
            .iter()
            .fold(Self::Item::one(), |acc, &x| acc * x)
    }

    fn abs(self) -> Self
    where
        Self::Item: Neg<Output = Self::Item> + PartialOrd + Zero + Copy,
    {
        self.op_unary(|a| if *a < Self::Item::zero() { -*a } else { *a })
    }

    fn abs_sum(&self) -> Self::Item
    where
        Self::Item: Neg<Output = Self::Item> + PartialOrd + Zero + Add<Output = Self::Item> + Copy,
    {
        self.abs().sum()
    }

    fn to_arr(self) -> [Self::Item; SIZE] {
        self.into()
    }

    fn from_arr(arr: [Self::Item; SIZE]) -> Self {
        arr.into()
    }

    fn op_unary(&self, op: impl Fn(&Self::Item) -> Self::Item) -> Self {
        Self::from_arr(
            self.to_arr()
                .iter()
                .map(|x| op(x))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        )
    }

    fn op_binary(&self, other: &Self, op: impl Fn(&Self::Item, &Self::Item) -> Self::Item) -> Self {
        Self::from_arr(
            self.to_arr()
                .iter()
                .zip(other.to_arr().iter())
                .map(|(a, b)| op(a, b))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        )
    }
}

// Dims
impl From<[i32; 2]> for Dims {
    fn from(arr: [i32; 2]) -> Self {
        Dims(arr[0], arr[1])
    }
}

impl From<Dims> for [i32; 2] {
    fn from(dims: Dims) -> Self {
        [dims.0, dims.1]
    }
}

impl DimsTrait<2> for Dims {
    type Item = i32;
}

impl From<Dims3D> for Dims {
    fn from(dims: Dims3D) -> Self {
        Dims(dims.0, dims.1)
    }
}

impl From<(i32, i32)> for Dims {
    fn from((x, y): (i32, i32)) -> Self {
        Dims(x, y)
    }
}

impl From<Dims> for (i32, i32) {
    fn from(dims: Dims) -> Self {
        (dims.0, dims.1)
    }
}

// Dims3D
impl From<[i32; 3]> for Dims3D {
    fn from(arr: [i32; 3]) -> Self {
        Dims3D(arr[0], arr[1], arr[2])
    }
}

impl From<Dims3D> for [i32; 3] {
    fn from(dims: Dims3D) -> Self {
        [dims.0, dims.1, dims.2]
    }
}

impl DimsTrait<3> for Dims3D {
    type Item = i32;
}

// DimsU
impl From<[usize; 2]> for DimsU {
    fn from(arr: [usize; 2]) -> Self {
        DimsU(arr[0], arr[1])
    }
}

impl From<DimsU> for [usize; 2] {
    fn from(dims: DimsU) -> Self {
        [dims.0, dims.1]
    }
}

impl DimsTrait<2> for DimsU {
    type Item = usize;
}

// Dims3DU
impl From<[usize; 3]> for Dims3DU {
    fn from(arr: [usize; 3]) -> Self {
        Dims3DU(arr[0], arr[1], arr[2])
    }
}

impl From<Dims3DU> for [usize; 3] {
    fn from(dims: Dims3DU) -> Self {
        [dims.0, dims.1, dims.2]
    }
}

impl DimsTrait<3> for Dims3DU {
    type Item = usize;
}

macro_rules! impl_op {
    ($trait:ident + Assign, $op:ident, $dims:ident $(, $item:ident)?) => {
        impl_op!($trait, $op, $dims, $($item)?);

        paste! {
            impl [<$trait Assign>] for $dims {
                #[inline(always)]
                fn [<$op _assign>](&mut self, other: $dims) {
                    *self = self.$op(other);
                }
            }

            $(
                impl [<$trait Assign>]<$item> for $dims {
                    #[inline(always)]
                    fn [<$op _assign>](&mut self, other: $item) {
                        *self = self.$op(other);
                    }
                }
            )?
        }
    };
    ($trait:ident, $op:ident, $dims:ident $(, $item:ident)?) => {
        impl $trait for $dims {
            type Output = $dims;

            #[inline(always)]
            fn $op(self, other: $dims) -> $dims {
                self.op_binary(&other, |a, b| a.$op(b))
            }
        }

        $(
            impl $trait<$item> for $dims {
                type Output = $dims;

                #[inline(always)]
                fn $op(self, other: $item) -> $dims {
                    self.op_unary(|a| a.$op(other))
                }
            }
        )?
    };
}

macro_rules! impl_ops {
    ( $(($dims:ident, $item:ident)),* ) => {
        $(
            impl_op!(Add + Assign, add, $dims, $item);
            impl_op!(Sub + Assign, sub, $dims, $item);
            impl_op!(Mul + Assign, mul, $dims, $item);
            impl_op!(Div + Assign, div, $dims, $item);
        )*
    };
}

impl_ops![(Dims, i32), (Dims3D, i32), (DimsU, usize), (Dims3DU, usize)];
