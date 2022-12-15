use std::io::stdout;

use masof::Renderer;
use tmaze::ui::{popup, CrosstermError};

fn main() -> Result<(), CrosstermError> {
    let mut renderer = Renderer::default();

    renderer.term_on(&mut stdout())?;

    popup::popup(
        &mut renderer,
        popup::ContentStyle::default(),
        popup::ContentStyle::default(),
        "Title",
        &["Line 1", "Line 2", "Line 3"],
    )?;

    renderer.term_off(&mut stdout())?;

    Ok(())
}
