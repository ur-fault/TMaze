pub mod app;
pub mod game_state;

use std::io;

pub use app::App;
pub use game_state::{GameState, GameViewMode};
use thiserror::Error;

use crate::settings::EditableFieldError;

#[derive(Debug, Error)]
pub enum GameError {
    #[error("Crossterm error: {0}")]
    CrosstermError(#[from] io::Error),
    // #[error("Empty menu, nothing to select")]
    // EmptyMenu,
    #[error("Back")]
    Back,
    #[error("Full quit")]
    FullQuit,
    #[error("New game")]
    NewGame,
}

impl From<EditableFieldError> for GameError {
    fn from(error: EditableFieldError) -> Self {
        match error {
            EditableFieldError::Back => Self::Back,
            EditableFieldError::Quit => Self::FullQuit,
            EditableFieldError::Crossterm(error) => Self::CrosstermError(error),
        }
    }
}
