pub mod game;
pub mod helpers;
pub mod renderer;
pub mod settings;
pub mod ui;
#[cfg(feature = "updates")]
pub mod updates;
pub mod constants;
pub mod data;

use cmaze::{core, gameboard};
