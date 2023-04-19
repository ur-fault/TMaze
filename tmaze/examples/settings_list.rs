use crossterm::Result as CResult;
use tmaze::{
    renderer::Renderer,
    settings::{self, editable::EditableField},
};

fn main() -> CResult<()> {
    let mut renderer = Renderer::new()?;
    let mut v = vec![1, 2, 3];
    v.edit(&mut renderer, settings::ColorScheme::default())
        .unwrap();

    renderer.render()?;
    Ok(())
}
