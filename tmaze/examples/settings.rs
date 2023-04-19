use crossterm::Result as CResult;
use tmaze::{
    renderer::Renderer,
    settings::{self, editable::EditableField},
};

fn main() -> CResult<()> {
    let mut renderer = Renderer::new()?;
    let mut settings = settings::Settings::new();
    settings
        .edit(&mut renderer, settings::ColorScheme::default())
        .unwrap();
    renderer.render()?;
    Ok(())
}
