use std::io::stdout;

use masof::Renderer;
use tmaze::ui::{menu, CrosstermError, MenuError};

fn main() -> Result<(), CrosstermError> {
    let mut renderer = Renderer::default();

    renderer.term_on(&mut stdout())?;

    let res = menu::menu(
        &mut renderer,
        menu::ContentStyle::default(),
        menu::ContentStyle::default(),
        "Menu",
        &["Option 1", "Option 2", "Option 3"],
        Some(0),
        true,
    );

    renderer.term_off(&mut stdout())?;

    match res {
        Ok(i) => println!("Selected option {}", i),
        Err(MenuError::CrosstermError(err)) => return Err(err),
        Err(MenuError::EmptyMenu) => println!("No options"),
        _ => {}
    }

    Ok(())
}
