use std::{
    any::Any,
    ops::{Deref, DerefMut},
};

use crate::ui::Screen;

use super::{app::AppData, event::Event};

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
    PopUntil {
        name: String,
        res: Option<ActivityResult>,
    },
    Replace(Activity),
    ReplaceAt {
        index: usize,
        activity: Activity,
    },
}

impl Change {
    pub fn push(activity: Activity) -> Self {
        Self::Push(activity)
    }

    pub fn pop(n: usize) -> Self {
        Self::Pop { n, res: None }
    }

    pub fn pop_with<T: 'static>(n: usize, res: T) -> Self {
        Self::Pop {
            n,
            res: Some(Box::new(res)),
        }
    }

    pub fn pop_top() -> Self {
        Self::Pop { n: 1, res: None }
    }

    pub fn pop_top_with<T: 'static>(res: T) -> Self {
        Self::Pop {
            n: 1,
            res: Some(Box::new(res)),
        }
    }

    pub fn pop_all() -> Self {
        Self::Pop {
            n: usize::MAX,
            res: None,
        }
    }

    pub fn pop_until(name: impl Into<String>) -> Self {
        Self::PopUntil {
            name: name.into(),
            res: None,
        }
    }

    pub fn pop_until_with<T: 'static>(name: impl Into<String>, res: T) -> Self {
        Self::PopUntil {
            name: name.into(),
            res: Some(Box::new(res)),
        }
    }

    pub fn replace(activity: Activity) -> Self {
        Self::Replace(activity)
    }

    pub fn replace_at(index: usize, activity: Activity) -> Self {
        Self::ReplaceAt { index, activity }
    }
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
        self.activities
            .truncate(self.activities.len() - n.min(self.activities.len()));
    }

    pub fn pop_until(&mut self, name: &str) -> usize {
        if let Some(index) = self.activities.iter().rposition(|a| a.name() == name) {
            self.activities.truncate(index);
            index
        } else {
            log::warn!("Activity `{}` not found, popping top", name);
            self.pop_n(1);
            1
        }
    }

    pub fn replace(&mut self, activity: Activity) {
        self.activities.pop();
        self.activities.push(activity);
    }

    pub fn replace_at(&mut self, index: usize, activity: Activity) {
        self.activities[index] = activity;
        self.activities.truncate(index + 1);
    }

    // -- Getters
    pub fn active(&self) -> Option<&Activity> {
        self.activities.last()
    }

    pub fn active_mut(&mut self) -> Option<&mut Activity> {
        self.activities.last_mut()
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

    pub fn source(&self) -> &str {
        &self.source
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
    fn update(&mut self, events: Vec<Event>, data: &mut AppData) -> Option<Change>;

    fn screen(&self) -> &dyn Screen;
}
