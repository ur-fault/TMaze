use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("CrossTerm error; {0}")]
    CrossTermError(#[from] crossterm::ErrorKind),
    #[error("Renderer error; {0}")]
    DrawBufferError(#[from] masof::renderer::Error),
    #[error("Quit")]
    Quit,
    #[error("FullQuit")]
    FullQuit,
    #[error("EmptyMenu")]
    EmptyMenu,
    #[error("InvalidValue")]
    InvalidValue,
}

pub type Dims = (i32, i32);
pub type Dims3D = (i32, i32, i32);
pub type DimsU = (usize, usize);
