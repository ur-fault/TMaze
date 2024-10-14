use std::io;

use tmaze::{
    app::{app::AppData, Activity, ActivityHandler, App, Change, Event},
    helpers::is_release,
    renderer::Frame,
    settings::theme::Theme,
    ui::Screen,
};

use crossterm::event::{Event as TermEvent, KeyEvent};

fn main() {
    let mut app = App::new(Activity::new("example", "box", Box::new(MyActivity)), true);

    log::info!("Starting app");

    app.run();
}

struct MyActivity;

impl ActivityHandler for MyActivity {
    fn update(&mut self, events: Vec<Event>, _: &mut AppData) -> Option<Change> {
        for event in events {
            match event {
                Event::Term(TermEvent::Key(KeyEvent { code, kind, .. })) if !is_release(kind) => {
                    if code == crossterm::event::KeyCode::Char('q') {
                        return Some(Change::pop_top());
                    }

                    match code {
                        crossterm::event::KeyCode::Char(ch) if ch as u32 % 2 == 0 => {
                            log::warn!("Even key pressed: '{ch}'");
                        }
                        crossterm::event::KeyCode::Char(ch) if ch as u32 % 2 == 1 => {
                            log::error!("Odd key pressed: '{ch}'");
                        }
                        _ => {}
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

impl Screen for MyActivity {
    fn draw(&self, _: &mut Frame, _: &Theme) -> io::Result<()> {
        Ok(())
    }
}
