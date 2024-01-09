pub mod constants;
pub mod data;
pub mod game;
pub mod helpers;
pub mod settings;
pub mod ui;
#[cfg(feature = "updates")]
pub mod updates;

use cmaze::{core, gameboard};
