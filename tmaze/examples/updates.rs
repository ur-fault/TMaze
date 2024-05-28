use tmaze::{
    app::{
        app::{App, AppData},
        Activity, ActivityHandler,
    },
    ui::Popup,
};

#[cfg(feature = "updates")]
use tmaze::updates::UpdateCheckerActivity;

#[cfg(feature = "updates")]
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

#[cfg(not(feature = "updates"))]
fn main() {
    panic!("Cannot run `updates` example without the `updates` feature");
}

#[cfg(feature = "updates")]
struct MyActivity(bool, Popup);

#[cfg(feature = "updates")]
impl ActivityHandler for MyActivity {
    fn update(
        &mut self,
        events: Vec<tmaze::app::Event>,
        data: &mut AppData,
    ) -> Option<tmaze::app::Change> {
        if !self.0 {
            self.0 = true;

            let update_act = Activity::new_base(
                "update",
                Box::new(UpdateCheckerActivity::new(&data.settings, &data.save)),
            );

            return Some(tmaze::app::Change::Push(update_act));
        }

        self.1.update(events, data)
    }

    fn screen(&self) -> &dyn tmaze::ui::Screen {
        &self.1
    }
}
