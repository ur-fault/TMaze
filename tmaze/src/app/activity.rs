use std::{
    any::Any,
    ops::{Deref, DerefMut},
};

use crate::ui::Screen;

use super::event::Event;

pub type ActivityResult = Box<dyn Any>;

pub struct Activities {
    activities: Vec<Activity>,
}

pub enum Change {
    Push(Activity),
    Pop {
        n: usize,
        res: Option<ActivityResult>,
    },
    PopTop(Option<ActivityResult>),
}

impl Activities {
    pub fn new(base: Activity) -> Self {
        Self {
            activities: vec![base],
        }
    }

    pub fn empty() -> Self {
        Self { activities: vec![] }
    }

    pub fn push(&mut self, activity: Activity) {
        self.activities.push(activity);
    }

    pub fn pop(&mut self) {
        self.activities.pop();
    }

    pub fn pop_n(&mut self, n: usize) {
        self.activities.truncate(self.activities.len() - n);
    }

    pub fn active(&self) -> Option<&Activity> {
        self.activities.last()
    }

    pub fn active_mut(&mut self) -> Option<&mut Activity> {
        self.activities.last_mut()
    }

    pub fn update(&mut self, mut events: Vec<Event>) -> bool {
        if self.active().is_none() {
            return true;
        }

        while let Some(change) = self
            .active_mut()
            .expect("No active activity")
            // clone vec while clearing it
            .update(events.drain(..).collect())
        {
            match change {
                Change::Push(activity) => self.activities.push(activity),
                Change::Pop { n, res } => {
                    self.pop_n(n);
                    events.push(Event::ActiveAfterPop(res));
                }
                Change::PopTop(res) => {
                    self.activities.pop();
                    events.push(Event::ActiveAfterPop(res));
                }
            }
        }

        return false;
    }

    pub fn len(&self) -> usize {
        self.activities.len()
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

    pub fn name(&self) -> &str {
        &self.name
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
