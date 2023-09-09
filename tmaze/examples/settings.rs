use tmaze::{
    renderer::Renderer,
    settings::{self, editable::EditableField},
    ui::CRResult,
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
