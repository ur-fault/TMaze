use crate::{
    app::{self, app::AppData, Activity, ActivityHandler, Change},
    helpers::constants::paths::settings_path,
    menu_actions,
    renderer::MouseGuard,
    settings::Settings,
    sound::create_audio_settings,
    ui::{split_menu_actions, Menu, MenuAction, MenuConfig, MenuItem, OptionDef, Popup, Screen},
};

struct OtherSettingsPopup(Popup, MouseGuard);

impl OtherSettingsPopup {
    fn new() -> Self {
        let popup = Popup::new(
            "Other settings".to_string(),
            vec![
                "Path to the current settings:".to_string(),
                format!(" {}", settings_path().to_string_lossy()),
                "".to_string(),
                "Other settings are not implemented in UI yet.".to_string(),
                "Please edit the settings file directly.".to_string(),
            ],
        );

        Self(popup, MouseGuard::new().unwrap())
    }
}

impl ActivityHandler for OtherSettingsPopup {
    fn update(&mut self, events: Vec<app::Event>, data: &mut AppData) -> Option<Change> {
        self.0.update(events, data)
    }

    fn screen(&mut self) -> &mut dyn Screen {
        &mut self.0
    }
}

pub struct SettingsActivity {
    actions: Vec<MenuAction<Change>>,
    menu: Menu,
}

impl SettingsActivity {
    fn other_settings_popup() -> Activity {
        Activity::new_base_boxed("settings".to_string(), OtherSettingsPopup::new())
    }
}

#[allow(clippy::new_without_default)]
impl SettingsActivity {
    pub fn new() -> Self {
        let options = menu_actions!(
            "Audio" on "sound" -> data => Change::push(create_audio_settings(data)),
            "Controls" -> data => Change::push(create_controls_settings(data)),
            "Other settings" -> _ => Change::push(SettingsActivity::other_settings_popup()),
            "Back" -> _ => Change::pop_top(),
        );

        let (options, actions) = split_menu_actions(options);

        let menu_config = MenuConfig::new("Settings", options).subtitle("Changes are not saved");

        Self {
            actions,
            menu: Menu::new(menu_config),
        }
    }

    pub fn new_activity() -> Activity {
        Activity::new_base_boxed("settings".to_string(), Self::new())
    }
}

impl ActivityHandler for SettingsActivity {
    fn update(&mut self, events: Vec<app::Event>, data: &mut AppData) -> Option<Change> {
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

pub fn create_controls_settings(data: &mut AppData) -> Activity {
    let cfg = &data.settings.read().nagivation;

    let menu_config = MenuConfig::new(
        "Controls settings",
        [
            MenuItem::Option(OptionDef {
                text: "Enable mouse input".into(),
                val: cfg.enable_mouse,
                fun: Box::new(|enabled, data| {
                    *enabled = !*enabled;
                    // data.settings.set_enable_mouse(*enabled);
                }),
            }),
            MenuItem::Option(OptionDef {
                text: "Enable dpad".into(),
                val: cfg.enable_dpad,
                fun: Box::new(|enabled, data| {
                    *enabled = !*enabled;
                    // data.settings.set_enable_dpad(*enabled);
                }),
            }),
            MenuItem::Option(OptionDef {
                text: "Left-handed dpad".into(),
                val: cfg.landscape_dpad_on_left,
                fun: Box::new(|is_on_left, data| {
                    *is_on_left = !*is_on_left;
                    // data.settings.set_landscape_dpad_on_left(*is_on_left);
                }),
            }),
            MenuItem::Option(OptionDef {
                text: "Swap Up and Down buttons".into(),
                val: cfg.dpad_swap_up_down,
                fun: Box::new(|do_swap, data| {
                    *do_swap = !*do_swap;
                    // data.settings.set_dpad_swap_up_down(*do_swap);
                }),
            }),
            MenuItem::Option(OptionDef {
                text: "Enable margin around dpad".into(),
                val: cfg.enable_margin_around_dpad,
                fun: Box::new(|enabled, data| {
                    *enabled = !*enabled;
                    // data.settings.set_enable_margin_around_dpad(*enabled);
                }),
            }),
            MenuItem::Option(OptionDef {
                text: "Enable dpad highlight".into(),
                val: cfg.enable_dpad_highlight,
                fun: Box::new(|enabled, data| {
                    *enabled = !*enabled;
                    // data.settings.set_enable_dpad_highlight(*enabled);
                }),
            }),
            MenuItem::Separator,
            MenuItem::Text("Exit".into()),
        ],
    );

    Activity::new_base_boxed("controls settings", Menu::new(menu_config))
}
