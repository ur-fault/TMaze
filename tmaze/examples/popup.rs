use crossterm::style::ContentStyle;
use tmaze::{
    renderer::Renderer,
    ui::{
        popup::{self},
        CrosstermError,
    },
};

fn main() -> Result<(), CrosstermError> {
    let mut renderer = Renderer::new()?;

    // renderer.term_on(&mut stdout())?;

    popup::popup(
        &mut renderer,
        ContentStyle::default(),
        ContentStyle::default(),
        "Title",
        &["Line 1", "Line 2", "Line 3"],
    )?;

    // renderer.term_off(&mut stdout())?;

    Ok(())
}
