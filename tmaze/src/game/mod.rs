pub mod app;
pub mod game_state;
use std::io;

pub use app::App;
pub use game_state::{GameState, GameViewMode};

use crate::{
    settings::EditableFieldError,
    ui::{CrosstermError, MenuError},
};

#[derive(Debug)]
pub enum GameError {
    CrosstermError(CrosstermError),
    EmptyMaze,
    Back,
    FullQuit,
    NewGame,
}

impl From<MenuError> for GameError {
    fn from(error: MenuError) -> Self {
        match error {
            MenuError::CrosstermError(error) => Self::CrosstermError(error),
            // TODO: this shouldn't be EmptyMaze or at least it doesn't make sense
            MenuError::EmptyMenu => Self::EmptyMaze,
            MenuError::Exit => Self::Back,
            MenuError::FullQuit => Self::FullQuit,
        }
    }
}

impl From<CrosstermError> for GameError {
    fn from(error: CrosstermError) -> Self {
        Self::CrosstermError(error)
    }
}

impl From<io::Error> for GameError {
    fn from(error: io::Error) -> Self {
        Self::CrosstermError(CrosstermError::from(error))
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
