use std::fmt;

use thiserror::Error;
use serde::{Deserialize, Serialize};

use crate::gameboard::maze::Maze;

#[derive(Debug, Error)]
pub enum SerializeError {
    IoError(std::io::Error),
}

impl fmt::Display for SerializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SerializeError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SerializableMaze {
    pub maze: Maze,
    #[serde(default)]
    pub title: String,
}