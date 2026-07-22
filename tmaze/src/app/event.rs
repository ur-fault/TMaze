use crossterm::event::Event as TermEvent;

use crate::app::app::AppData;

use super::activity::ActivityResult;

#[derive(Debug)]
pub enum ActivityEvent {
    Term(TermEvent),
    ActiveAfterPop(Option<ActivityResult>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalEvent {
    SettingsChanged,
    ThemeChanged,
}

pub type EventReceiverFn = Box<dyn FnMut(GlobalEvent, &mut AppData)>;

// FIXME: fix the API, this is really bad
//
// There are two main problems with this API:
// - This API leaks the implementation details of the event system.
// - This cannot be optimized in any way, it would be much better if we knew what events a receiver
//   is interested in, so we could avoid calling it for irrelevant events.
pub trait EventReceiver {
    fn register(self) -> EventReceiverFn;
}
