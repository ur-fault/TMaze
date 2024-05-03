use crossterm::event::Event as TermEvent;

use super::activity::ActivityResult;

pub enum Event {
    Term(TermEvent),
    ActiveAfterPop(Option<ActivityResult>),
}
