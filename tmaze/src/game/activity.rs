use crate::ui::Screen;

use super::event::Event;

pub struct Acitivties {
    activities: Vec<Activity>,
}

pub struct StackChanges {
    changes: Vec<Change>,
}

pub enum Change {
    Push(Activity),
    Pop(usize),
    Replace(Activity),
    Insert(usize, Activity),
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

    pub fn pop(&mut self) -> Option<Activity> {
        self.activities.pop()
    }

    pub fn active(&self) -> Option<&Activity> {
        self.activities.last()
    }

    pub fn update(&mut self, events: Vec<Event>) -> bool {
        if let Some(activity) = self.activities.last_mut() {
            let mut changes = StackChanges { changes: vec![] };
            activity.handler.update(&mut changes, events);

            for change in changes.changes {
                match change {
                    Change::Push(activity) => self.push(activity),
                    Change::Pop(n) => {
                        self.activities.truncate(self.activities.len() - n);
                    }
                    Change::Replace(activity) => {
                        self.pop();
                        self.push(activity);
                    }
                    Change::Insert(index, activity) => {
                        self.activities.insert(index, activity);
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
    // source, ie. mod or base game
    source: String,
    name: String,

    handler: Box<dyn ActivityHandler>,
}

pub trait ActivityHandler {
    fn update(&mut self, stack: &mut StackChanges, events: Vec<Event>);
    fn screen(&self) -> Box<dyn Screen>;
}
