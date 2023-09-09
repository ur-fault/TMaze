use std::io;

use crossterm::style::ContentStyle;
use tmaze::{
    renderer::Renderer,
    ui::{menu, MenuError},
};

fn main() -> io::Result<()> {
    let mut renderer = Renderer::new()?;

    let res = menu::menu(
        &mut renderer,
        ContentStyle::default(),
        ContentStyle::default(),
        "Menu",
        &["Option 1", "Option 2", "Option 3"],
        Some(0),
        true,
    );

    drop(renderer);

    match res {
        Ok(i) => println!("Selected option {}", i),
        Err(MenuError::CrosstermError(err)) => return Err(err),
        Err(MenuError::EmptyMenu) => println!("No options"),
        _ => {}
    }

    Ok(())
}
