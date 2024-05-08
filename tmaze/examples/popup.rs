use std::io;

use crossterm::event::KeyCode;
use tmaze::{
    app::{Activity, App},
    ui::popup,
};

fn main() -> io::Result<()> {
    let mut app = App::new(Activity::new_base(
        "popup",
        Box::new(popup::Popup::new(
            "Title".to_string(),
            vec![
                "Line 1".to_string(),
                "Line 2".to_string(),
                "Line 3".to_string(),
            ],
        )),
    ));

    let res = app.run();

    // Manually drop the app, so we can se the printout
    drop(app);

    match res {
        Some(res) => {
            println!("Result: {:?}", res.downcast::<KeyCode>());
        }
        None => {
            println!("No result");
        }
    }

    Ok(())
}
