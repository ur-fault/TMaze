pub mod constants;
pub mod data;
pub mod game;
pub mod helpers;
pub mod renderer;
pub mod settings;
pub mod ui;
#[cfg(feature = "updates")]
pub mod updates;
#[cfg(feature = "sound")]
pub mod sound;

use cmaze::{core, gameboard};
