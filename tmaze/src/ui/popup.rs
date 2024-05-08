use crossterm::{
    event::{Event as TermEvent, KeyEvent},
    style::ContentStyle,
};

use std::cell::RefCell;

use super::draw::*;
use super::*;
use crate::app::{app::AppData, ActivityHandler, Change, Event};
use crate::helpers::is_release;

pub struct Popup {
    title: String,
    texts: Vec<String>,
}

impl Popup {
    pub fn new(title: String, texts: Vec<String>) -> Self {
        Self { title, texts }
    }
}

impl ActivityHandler for Popup {
    fn update(&mut self, events: Vec<crate::app::Event>, _: &mut AppData) -> Option<crate::app::Change> {
        for event in events {
            match event {
                Event::Term(TermEvent::Key(KeyEvent { code, kind, .. })) => {
                    if !is_release(kind) {
                        return Some(Change::pop_top_with(code));
                    }
                }
                _ => {}
            }
        }

        None
    }

    fn screen(&self) -> &dyn Screen {
        self
    }
}

impl Screen for Popup {
    fn draw(&self, frame: &mut Frame) -> io::Result<()> {
        let box_style = ContentStyle::default();
        let text_style = ContentStyle::default();

        let box_size = popup_size(&self.title, &self.texts);
        let title_pos = box_center_screen(Dims(self.title.len() as i32 + 2, 1)).0;
        let pos = box_center_screen(box_size);

        let mut context = DrawContext {
            frame: &RefCell::new(frame),
            style: box_style,
            rect: None,
        };

        context.draw_box(pos, box_size);
        context.draw_str_styled(
            Dims(title_pos, pos.1 + 1),
            &format!(" {} ", self.title),
            text_style,
        );

        if !self.texts.is_empty() {
            context.draw_str(pos + Dims(1, 2), &"─".repeat(box_size.0 as usize - 2));
            for (i, text) in self.texts.iter().enumerate() {
                context.draw_str_styled(pos + Dims(2, i as i32 + 3), text, text_style);
            }
        }

        Ok(())
    }
}

pub fn popup_size(title: &str, texts: &[String]) -> Dims {
    match texts.iter().map(|text| text.len()).max() {
        Some(l) => Dims(
            2 + 2 + l.max(title.len()) as i32,
            2 + 2 + texts.len() as i32,
        ),
        None => Dims(4 + title.len() as i32, 3),
    }
}
