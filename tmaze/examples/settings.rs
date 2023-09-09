use std::io;

use tmaze::{
    renderer::Renderer,
    settings::{self, editable::EditableField},
};

fn main() -> io::Result<()> {
    let mut renderer = Renderer::new()?;
    let mut settings = settings::Settings::new();

    settings
        .edit(&mut renderer, settings::ColorScheme::default())
        .unwrap();
    renderer.render()?;

    Ok(())
}
