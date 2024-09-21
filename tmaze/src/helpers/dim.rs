use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Offset {
    Abs(i32),
    Rel(f32),
}

impl Offset {
    pub fn to_abs(self, size: i32) -> i32 {
        match self {
            Offset::Rel(ratio) => (size as f32 * ratio).round() as i32,
            Offset::Abs(chars) => chars,
        }
    }
}

impl Default for Offset {
    fn default() -> Self {
        Offset::Rel(0.25)
    }
}
