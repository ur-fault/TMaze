use crossterm::{
    event::{Event as TermEvent, KeyEvent, MouseButton, MouseEvent, MouseEventKind},
    style::ContentStyle,
};
use unicode_width::UnicodeWidthStr;

use super::draw_fn::*;
use super::*;
use crate::helpers::is_release;
use crate::{
    app::{app::AppData, ActivityHandler, Change, Event},
    settings::Settings,
};

pub struct Popup {
    title: String,
    texts: Vec<String>,
    box_style: Option<ContentStyle>,
    text_style: Option<ContentStyle>,
    title_style: Option<ContentStyle>,
}

impl Popup {
    pub fn new(title: String, texts: impl Into<Vec<String>>) -> Self {
        Self {
            title,
            texts: texts.into(),
            box_style: None,
            text_style: None,
            title_style: None,
        }
    }

    pub fn box_style(mut self, style: ContentStyle) -> Self {
        self.box_style = Some(style);
        self
    }

    pub fn text_style(mut self, style: ContentStyle) -> Self {
        self.text_style = Some(style);
        self
    }

    pub fn title_style(mut self, style: ContentStyle) -> Self {
        self.title_style = Some(style);
        self
    }

    pub fn styles_from_settings(mut self, settings: &Settings) -> Self {
        let colorscheme = settings.get_color_scheme();
        self.box_style = Some(colorscheme.normals());
        self.text_style = Some(colorscheme.texts());
        self
    }
}

impl ActivityHandler for Popup {
    fn update(&mut self, events: Vec<Event>, _: &mut AppData) -> Option<Change> {
        for event in events {
            #[allow(clippy::single_match)] // for more events to come
            match event {
                Event::Term(event) => match event {
                    TermEvent::Key(KeyEvent { code, kind, .. }) => {
                        if !is_release(kind) {
                            return Some(Change::pop_top_with(code));
                        }
                    }
                    TermEvent::Mouse(MouseEvent {
                        kind: MouseEventKind::Up(MouseButton::Left),
                        ..
                    }) => {
                        return Some(Change::pop_top());
                    }
                    _ => {}
                },
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
    fn draw(&self, frame: &mut Frame, color_scheme: &ColorScheme) -> io::Result<()> {
        let box_size = popup_size(&self.title, &self.texts);
        let title_pos = center_box_in_screen(Dims(self.title.width() as i32, 1)).0;
        let pos = center_box_in_screen(box_size);

        let box_style = self.box_style.unwrap_or(color_scheme.normals());
        let text_style = self.text_style.unwrap_or(color_scheme.texts());
        let title_style = self.title_style.or(self.text_style).unwrap_or(text_style);

        draw_box(frame, pos, box_size, box_style);
        frame.draw_styled(Dims(title_pos, pos.1 + 1), self.title.as_str(), title_style);

        if !self.texts.is_empty() {
            frame.draw_styled(
                pos + Dims(1, 2),
                "─".repeat(box_size.0 as usize - 2),
                box_style,
            );

            for (i, text) in self.texts.iter().enumerate() {
                frame.draw_styled(pos + Dims(2, i as i32 + 3), text.as_str(), text_style);
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
