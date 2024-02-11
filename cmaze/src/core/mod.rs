pub mod dims;

pub use dims::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameMode {
    pub size: Dims3D,
    pub is_tower: bool,
}
