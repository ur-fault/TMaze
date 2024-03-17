pub mod app;
pub mod game_state;

use std::io;

pub use app::App;
pub use game_state::{GameState, GameViewMode};
use thiserror::Error;

use crate::{settings::EditableFieldError, ui::MenuError};

#[derive(Debug, Error)]
pub enum GameError {
    #[error("Crossterm error: {0}")]
    CrosstermError(#[from] io::Error),
    #[error("Empty menu, nothing to select")]
    EmptyMenu,
    #[error("Back")]
    Back,
    #[error("Full quit")]
    FullQuit,
    #[error("New game")]
    NewGame,
}

impl From<MenuError> for GameError {
    fn from(error: MenuError) -> Self {
        match error {
            MenuError::CrosstermError(error) => Self::CrosstermError(error),
            // TODO: this shouldn't be EmptyMaze or at least it doesn't make sense
            MenuError::EmptyMenu => Self::EmptyMenu,
            MenuError::Exit => Self::Back,
            MenuError::FullQuit => Self::FullQuit,
        }
    }
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
