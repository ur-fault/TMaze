use tmaze::{
    app::{
        app::{App, AppData},
        Activity, ActivityHandler,
    },
    ui::{menu, Popup},
};

fn main() {
    let mut app = App::new(Activity::new_base(
        "activity",
        Box::new(MyActivity(
            false,
            Popup::new("Press any key to quit".to_string(), vec![]),
        )),
    ));

    app.run();
}

struct MyActivity(bool, Popup);

impl ActivityHandler for MyActivity {
    fn update(
        &mut self,
        events: Vec<tmaze::app::Event>,
        data: &mut AppData,
    ) -> Option<tmaze::app::Change> {
        if !self.0 {
            self.0 = true;

            let menu_config =
                menu::MenuConfig::new_from_strings("Menu", vec!["Option 1".to_string()])
                    .counted()
                    .default(1);

            let menu = menu::Menu::new(menu_config).into_activity();
            return Some(tmaze::app::Change::Push(menu));
        }

        self.1.update(events, data)
    }

    fn screen(&self) -> &dyn tmaze::ui::Screen {
        &self.1
    }
}
