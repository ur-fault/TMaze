use crossterm::event::Event as TermEvent;

pub enum Event {
    TermEvent(TermEvent),
}
