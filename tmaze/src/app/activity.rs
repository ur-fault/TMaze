use std::{
    any::Any,
    ops::{Deref, DerefMut},
};

use crate::ui::Screen;

use super::event::Event;

pub type ActivityResult = Box<dyn Any>;

pub struct Acitivties {
    activities: Vec<Activity>,
}

// pub struct StackChanges {
//     changes: Vec<Change>,
// }
//
// impl StackChanges {
//     pub fn push(&mut self, activity: Activity) {
//         self.changes.push(Change::Push(activity));
//     }
//
//     pub fn pop(&mut self, n: usize) {
//         self.changes.push(Change::Pop { n, result: None });
//     }
//
//     pub fn pop_top(&mut self) {
//         self.pop(1);
//     }
// }

pub enum Change {
    Push(Activity),
    Pop {
        n: usize,
        res: Option<ActivityResult>,
    },
    PopTop {
        res: Option<ActivityResult>,
    },
}

impl Acitivties {
    pub fn new(base: Activity) -> Self {
        Self {
            activities: vec![base],
        }
    }

    pub fn push(&mut self, activity: Activity) {
        self.activities.push(activity);
    }

    pub fn pop(&mut self) {
        self.activities.pop();
    }

    pub fn active(&self) -> Option<&Activity> {
        self.activities.last()
    }

    pub fn active_mut(&mut self) -> Option<&mut Activity> {
        self.activities.last_mut()
    }

    pub fn update(&mut self, events: Vec<Event>) -> bool {
        if let Some(activity) = self.activities.last_mut() {
            let stack_change = activity.handler.update(events);

            if let Some(change) = stack_change {
                match change {
                    Change::Push(activity) => self.push(activity),
                    Change::Pop { n, res: _ } => {
                        self.activities.truncate(self.activities.len() - n);

                        if let Some(active) = self.active_mut() {
                            active.handler.update(vec![]);
                        }
                    }
                    Change::PopTop { res: _ } => {
                        self.activities.pop();

                        if let Some(active) = self.active_mut() {
                            active.handler.update(vec![]);
                        }
                    }
                }
            }

            return false;
        } else {
            return true;
        }
    }
}

pub struct Activity {
    source: String, // source, ie. mod or base game
    name: String,

    handler: Box<dyn ActivityHandler>,
}

impl Activity {
    pub fn new(
        source: impl Into<String>,
        name: impl Into<String>,
        handler: Box<dyn ActivityHandler>,
    ) -> Self {
        Self {
            source: source.into(),
            name: name.into(),
            handler,
        }
    }

    pub fn new_base(name: impl Into<String>, handler: Box<dyn ActivityHandler>) -> Self {
        Self::new("tmaze".to_string(), name.into(), handler)
    }
}

impl Deref for Activity {
    type Target = dyn ActivityHandler;

    fn deref(&self) -> &Self::Target {
        &*self.handler
    }
}

impl DerefMut for Activity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.handler
    }
}

pub trait ActivityHandler {
    #[must_use]
    fn update(&mut self, events: Vec<Event>) -> Option<Change>;

    fn screen(&self) -> &dyn Screen;
}
