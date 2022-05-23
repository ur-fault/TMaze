use crate::tmcore::*;
pub use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent},
    terminal::size,
};
pub use masof::{Color, ContentStyle, Renderer};
use std::io::Stdout;

use super::draw::*;
use super::*;

pub fn popup_size(title: &str, texts: &[&str]) -> Dims {
    match texts.iter().map(|text| text.len()).max() {
        Some(l) => (
            2 + 2 + l.max(title.len()) as i32,
            2 + 2 + texts.len() as i32,
        ),
        None => (4 + title.len() as i32, 3),
    }
}

pub fn popup(
    renderer: &mut Renderer,
    style: ContentStyle,
    stdout: &mut Stdout,
    title: &str,
    texts: &[&str],
) -> Result<KeyCode, Error> {
    render_popup(renderer, style, stdout, title, texts)?;

    loop {
        let event = read()?;
        if let Event::Key(KeyEvent { code, modifiers: _ }) = event {
            break Ok(code);
        }

        renderer.event(&event);

        render_popup(renderer, style, stdout, title, texts)?;
    }
}

pub fn render_popup(
    renderer: &mut Renderer,
    style: ContentStyle,
    stdout: &mut Stdout,
    title: &str,
    texts: &[&str],
) -> Result<(), Error> {
    let box_size = popup_size(title, texts);
    let title_pos = box_center_screen((title.len() as i32 + 2, 1))?.0;
    let pos = box_center_screen(box_size)?;

    renderer.begin()?;
    {
        let mut context = DrawContext { renderer, style };

        context.draw_box(pos, box_size);
        context.draw_str(title_pos, pos.1 + 1, &format!(" {} ", title));

        if texts.len() != 0 {
            context.draw_str(pos.0 + 1, pos.1 + 2, &"─".repeat(box_size.0 as usize - 2));
            for (i, text) in texts.iter().enumerate() {
                context.draw_str(pos.0 + 2, pos.1 + 3 + i as i32, text);
            }
        }
    }

    renderer.end(stdout)?;

    Ok(())
}
