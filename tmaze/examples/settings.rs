use cmaze::gameboard::Dims;
use crossterm::style::ContentStyle;
use tmaze::{
    renderer::Renderer,
    settings::{self, editable::EditableField},
    ui::{draw_box, wait_for_key, CRResult},
};

fn main() -> CRResult<()> {
    let mut renderer = Renderer::new()?;
    let mut settings = settings::Settings::new();
    settings
        .edit(&mut renderer, settings::ColorScheme::default())
        .unwrap();
    renderer.render()?;
    Ok(())
}
