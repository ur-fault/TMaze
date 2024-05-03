use std::io;

use cmaze::gameboard::Dims;
use crossterm::style::ContentStyle;
use tmaze::{renderer::Renderer, ui::draw};

fn main() -> io::Result<()> {
    let mut renderer = Renderer::new()?;

    let mut events = vec![];

    loop {
        draw::draw_box(
            &mut &mut renderer,
            Dims(0, 0),
            Dims(10, 10),
            ContentStyle::default(),
        );
        renderer.show()?;

        let event = crossterm::event::read()?;
        events.push(event.clone());

        match event {
            crossterm::event::Event::Key(crossterm::event::KeyEvent {
                code:
                    crossterm::event::KeyCode::Char('q')
                    | crossterm::event::KeyCode::Esc
                    | crossterm::event::KeyCode::Enter,
                kind,
                ..
            }) if kind != crossterm::event::KeyEventKind::Release => break,
            _ => {}
        }
    }

    drop(renderer);

    println!("Events: {:#?}", events);

    Ok(())
}
