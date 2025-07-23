use crate::app::{app::AppData, Activity, ActivityHandler, Change, Event};

use super::{Menu, MenuAction, Screen};

pub struct RedirectMenu {
    pub actions: Vec<MenuAction<Change>>,
    pub menu: Menu,
}

impl RedirectMenu {}

impl RedirectMenu {
    pub fn to_activity(self, name: impl Into<String>) -> Activity {
        Activity::new_base_boxed(name, self)
    }
}

impl ActivityHandler for RedirectMenu {
    fn update(&mut self, events: Vec<Event>, data: &mut AppData) -> Option<Change> {
        match self.menu.update(events, data)? {
            Change::Pop {
                res: Some(sub_activity),
                ..
            } => {
                let index = *sub_activity
                    .downcast::<usize>()
                    .expect("menu should return index");
                Some((self.actions[index])(data))
            }
            res => Some(res),
        }
    }

    fn screen(&mut self) -> &mut dyn Screen {
        &mut self.menu
    }
}
