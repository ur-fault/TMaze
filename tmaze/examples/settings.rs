use cmaze::gameboard::Dims;
use tmaze::{renderer::Renderer, ui::{wait_for_key, draw_box}, settings::{self, editable::EditableField}};
use crossterm::{Result as CResult, style::ContentStyle};

fn main() -> CResult<()> {
    let mut renderer = Renderer::new()?;
    let mut settings = settings::Settings::new();
    settings.edit(&mut renderer, settings::ColorScheme::default()).unwrap();
    renderer.render()?;
    Ok(())
}