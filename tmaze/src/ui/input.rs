use crate::{helpers::is_release, ui};
use cmaze::gameboard::Dims;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    style::{Color, ContentStyle},
};

use crate::renderer::Renderer;

use super::{box_center_screen, GenericUIError};

pub fn input(
    mut renderer: &mut Renderer,
    box_style: ContentStyle,
    text_style: ContentStyle,
    title: impl AsRef<str>,
    default: Option<String>,
    watermark: Option<impl AsRef<str>>,
) -> Result<String, GenericUIError> {
    let mut value = default.unwrap_or_default();
    let min_width = title
        .as_ref()
        .len()
        .max(watermark.as_ref().map(|w| w.as_ref().len()).unwrap_or(0)).max(20);

    loop {
        let box_size = Dims(min_width.max(value.len()) as i32 + 2, 3);
        let box_pos = box_center_screen(box_size)?;
        ui::draw_box(&mut renderer, box_pos, box_size, box_style);

        renderer.frame().draw(
            (box_pos.0 as u16 + 1, box_pos.1 as u16),
            (title.as_ref(), text_style),
        );
        renderer.frame().draw(
            (box_pos.0 as u16 + 1, box_pos.1 as u16 + 1),
            (value.as_str(), text_style),
        );

        if value.is_empty() {
            if let Some(watermark) = &watermark {
                renderer.frame().draw(
                    (box_pos.0 as u16 + 1, box_pos.1 as u16 + 1),
                    (
                        watermark.as_ref(),
                        ContentStyle {
                            foreground_color: Some(Color::DarkGrey),
                            ..Default::default()
                        },
                    ),
                );
            }
        }

        renderer.render()?;

        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind,
                ..
            }) if !is_release(kind) => return Ok(value),
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                kind,
                ..
            }) if !is_release(kind) => {
                value.pop();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                kind,
                ..
            }) if !is_release(kind) => {
                value.push(c);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                kind,
                ..
            }) if !is_release(kind) => return Err(GenericUIError::Back),
            _ => {}
        }
    }
}
