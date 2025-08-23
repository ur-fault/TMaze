use cmaze::dims::Dims;
use tmaze::{
    app::{app::AppData, Activity, ActivityHandler, App, Change, Event},
    helpers::is_release,
    renderer::{CellContent, GMutView},
    settings::theme::Theme,
    ui::{Rect, Screen, ScreenError},
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
                Event::Term(TermEvent::Key(KeyEvent { kind, .. })) if !is_release(kind) => {
                    return Some(Change::pop_top());
                }
                _ => {}
            }
        }

        None
    }

    fn screen(&mut self) -> &mut dyn Screen {
        self
    }
}

impl Screen for MyActivity {
    fn draw(&mut self, frame: &mut GMutView, _: &Theme) -> Result<(), ScreenError> {
        frame.fill_rect(
            Rect::new(Dims::ZERO, Dims(5, 5)),
            CellContent::styled('█', Default::default()),
        );

        Ok(())
    }
}
