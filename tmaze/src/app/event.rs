use crossterm::event::Event as TermEvent;

pub enum Event {
    Term(TermEvent),
}
