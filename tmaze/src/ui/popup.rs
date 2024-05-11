use crossterm::{
    event::{Event as TermEvent, KeyEvent},
    style::ContentStyle,
};
use unicode_width::UnicodeWidthStr;

use super::draw::*;
use super::*;
use crate::app::{app::AppData, ActivityHandler, Change, Event};
use crate::helpers::is_release;

pub struct Popup {
    title: String,
    texts: Vec<String>,
    title_style: ContentStyle,
    text_style: ContentStyle,
    box_style: ContentStyle,
}

impl Popup {
    pub fn new(title: String, texts: Vec<String>) -> Self {
        let style = ContentStyle::default();
        Self {
            title,
            texts,
            title_style: style,
            text_style: style,
            box_style: style,
        }
    }

    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title_style = style;
        self
    }

    pub fn text_style(mut self, style: ContentStyle) -> Self {
        self.text_style = style;
        self
    }

    pub fn box_style(mut self, style: ContentStyle) -> Self {
        self.box_style = style;
        self
    }
}

impl ActivityHandler for Popup {
    fn update(
        &mut self,
        events: Vec<crate::app::Event>,
        _: &mut AppData,
    ) -> Option<crate::app::Change> {
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
        let box_size = popup_size(&self.title, &self.texts);
        let title_pos = box_center_screen(Dims(self.title.width() as i32 + 3, 1)).0;
        let pos = box_center_screen(box_size);

        draw_box(frame, pos, box_size, self.box_style);
        frame.draw_styled(
            (Dims(title_pos, pos.1 + 1)).into(),
            self.title.as_str(),
            self.title_style,
        );

        if !self.texts.is_empty() {
            frame.draw(
                (pos + Dims(1, 2)).into(),
                "─".repeat(box_size.0 as usize - 2),
            );
            for (i, text) in self.texts.iter().enumerate() {
                frame.draw_styled(
                    (pos + Dims(2, i as i32 + 3)).into(),
                    text.as_str(),
                    self.text_style,
                );
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
