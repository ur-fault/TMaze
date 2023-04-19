use super::{box_center_screen, GenericUIError};
use crate::{
    helpers::is_release,
    renderer::{helpers::style, Renderer},
    ui,
};
use cmaze::gameboard::Dims;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    style::{Color, ContentStyle},
};

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
        .max(watermark.as_ref().map(|w| w.as_ref().len()).unwrap_or(0))
        .max(20);

    loop {
        let box_size = Dims(min_width.max(value.len()) as i32 + 2, 3);
        let box_pos = box_center_screen(box_size)?;
        ui::draw_box(&mut renderer, box_pos, box_size, box_style);
        let Dims(bx, by) = box_size;
        let (bx, by) = (bx as u16, by as u16);

        renderer
            .frame()
            .draw((bx + 1, by), (title.as_ref(), text_style));
        renderer
            .frame()
            .draw((bx + 1, by + 1), (value.as_str(), text_style));

        if value.is_empty() {
            if let Some(watermark) = &watermark {
                renderer.frame().draw(
                    (bx + 1, by + 1),
                    (watermark.as_ref(), style().f(Color::DarkGrey).build()),
                );
            }
        }

        renderer.render()?;

        if let Event::Key(KeyEvent {
            code,
            kind: KeyEventKind::Press | KeyEventKind::Repeat,
            ..
        }) = event::read()?
        {
            match code {
                KeyCode::Enter => return Ok(value),
                KeyCode::Backspace => {
                    value.pop();
                }
                KeyCode::Char(c) => {
                    value.push(c);
                }
                KeyCode::Esc => return Err(GenericUIError::Back),
                _ => {}
            }
        }
    }
}
