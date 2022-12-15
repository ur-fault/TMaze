pub use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent},
    terminal::size,
};
pub use masof::{Color, ContentStyle, Renderer};
use std::{cell::RefCell, io::stdout};

use super::draw::*;
use super::*;

pub fn popup_size(title: &str, texts: &[&str]) -> Dims {
    match texts.iter().map(|text| text.len()).max() {
        Some(l) => Dims(
            2 + 2 + l.max(title.len()) as i32,
            2 + 2 + texts.len() as i32,
        ),
        None => Dims(4 + title.len() as i32, 3),
    }
}

pub fn popup(
    renderer: &mut Renderer,
    box_style: ContentStyle,
    text_style: ContentStyle,
    title: &str,
    texts: &[&str],
) -> Result<KeyCode, CrosstermError> {
    render_popup(renderer, box_style, text_style, title, texts)?;

    loop {
        let event = read()?;
        if let Event::Key(KeyEvent { code, modifiers: _ }) = event {
            break Ok(code);
        }

        renderer.event(&event);

        render_popup(renderer, box_style, text_style, title, texts)?;
    }
}

pub fn render_popup(
    renderer: &mut Renderer,
    box_style: ContentStyle,
    text_style: ContentStyle,
    title: &str,
    texts: &[&str],
) -> Result<(), CrosstermError> {
    let box_size = popup_size(title, texts);
    let title_pos = box_center_screen(Dims(title.len() as i32 + 2, 1))?.0;
    let pos = box_center_screen(box_size)?;

    renderer.begin()?;
    {
        let mut context = DrawContext {
            renderer: &RefCell::new(renderer),
            style: box_style,
            frame: None,
        };

        context.draw_box(pos, box_size);
        context.draw_str_styled(
            Dims(title_pos, pos.1 + 1),
            &format!(" {} ", title),
            text_style,
        );

        if !texts.is_empty() {
            context.draw_str(pos + Dims(1, 2), &"─".repeat(box_size.0 as usize - 2));
            for (i, text) in texts.iter().enumerate() {
                context.draw_str_styled(pos + Dims(2, i as i32 + 3), text, text_style);
            }
        }
    }

    renderer.end(&mut stdout())?;

    Ok(())
}
