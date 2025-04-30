use crossterm::event::{Event as TermEvent, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use unicode_width::UnicodeWidthStr;

use cmaze::dims::Dims;

use super::{draw_fn::*, *};
use crate::{
    app::{app::AppData, ActivityHandler, Change, Event},
    helpers::is_release,
};

pub struct Popup {
    title: String,
    texts: Vec<String>,
}

impl Popup {
    pub fn new(title: String, texts: impl Into<Vec<String>>) -> Self {
        Self {
            title,
            texts: texts.into(),
        }
    }
}

impl ActivityHandler for Popup {
    fn update(&mut self, events: Vec<Event>, _: &mut AppData) -> Option<Change> {
        for event in events {
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
    fn draw(&self, frame: &mut Frame, theme: &Theme) -> io::Result<()> {
        let box_size = popup_size(&self.title, &self.texts);
        let title_pos = center_box_in_screen(Dims(self.title.width() as i32, 1)).0;
        let pos = center_box_in_screen(box_size);

        let box_style = theme["ui.popup.border"];
        let text_style = theme["ui.popup.text"];
        let title_style = theme["ui.popup.title"];

        draw_box(frame, pos, box_size, box_style);
        frame.draw(Dims(title_pos, pos.1 + 1), self.title.as_str(), title_style);

        if !self.texts.is_empty() {
            frame.draw(
                pos + Dims(1, 2),
                "─".repeat(box_size.0 as usize - 2),
                box_style,
            );

            for (i, text) in self.texts.iter().enumerate() {
                frame.draw(pos + Dims(2, i as i32 + 3), text.as_str(), text_style);
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

pub fn popup_theme_resolver() -> ThemeResolver {
    let mut resolver = ThemeResolver::new();

    resolver
        .link("ui.popup.border", "border")
        .link("ui.popup.text", "text")
        .link("ui.popup.title", "text");

    resolver
}
