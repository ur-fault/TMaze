use std::io;

use tmaze::{
    app::{Activity, ActivityHandler, App, Change, Event},
    helpers::is_release,
    renderer::Frame,
    ui::Screen,
};

use crossterm::event::{Event as TermEvent, KeyEvent};

fn main() {
    let mut app = App::new(Activity::new("example", "box", Box::new(MyActivity)));

    log::info!("Starting app");

    app.run();
}

struct MyActivity;

impl ActivityHandler for MyActivity {
    fn update(&mut self, events: Vec<Event>) -> Option<Change> {
        for event in events {
            match event {
                Event::Term(TermEvent::Key(KeyEvent { kind, .. })) if !is_release(kind) => {
                    return Some(Change::PopTop { res: None });
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
    fn draw(&self, frame: &mut Frame) -> io::Result<()> {
        Ok(())
    }
}
