use crate::{
    app::{self, app::AppData, Activity, ActivityHandler, Change},
    helpers::constants::paths::config,
    menu_actions,
    renderer::MouseGuard,
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
                format!(" {}", config().to_string_lossy()),
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
    let cfg = &data.settings.read().controls;

    let menu_config = MenuConfig::new(
        "Controls settings",
        [
            MenuItem::Option(OptionDef {
                text: "Enable mouse input".into(),
                val: cfg.mouse.enable,
                update_fn: Box::new(|enabled, data| {
                    data.settings.update_ui(|cfg| {
                        *cfg.controls().mouse().enable() = enabled;
                    });
                }),
                reset_fn: Some(Box::new(|data| {
                    data.settings.update_ui(|cfg| {
                        cfg.controls().mouse().enable = None;
                    });
                    data.settings.read().controls.mouse.enable
                })),
            }),
            MenuItem::Option(OptionDef {
                text: "Enable dpad".into(),
                val: cfg.mouse.dpad.enable,
                update_fn: Box::new(|enabled, data| {
                    data.settings.update_ui(|cfg| {
                        *cfg.controls().mouse().dpad().enable() = enabled;
                    });
                }),
                reset_fn: Some(Box::new(|data| {
                    data.settings.update_ui(|cfg| {
                        cfg.controls().mouse().dpad().enable = None;
                    });
                    data.settings.read().controls.mouse.dpad.enable
                })),
            }),
            MenuItem::Option(OptionDef {
                text: "Left-handed dpad".into(),
                val: cfg.mouse.dpad.landscape_on_left,
                update_fn: Box::new(|is_on_left, data| {
                    data.settings.update_ui(|cfg| {
                        *cfg.controls().mouse().dpad().landscape_on_left() = is_on_left;
                    });
                }),
                reset_fn: Some(Box::new(|data| {
                    data.settings.update_ui(|cfg| {
                        cfg.controls().mouse().dpad().landscape_on_left = None;
                    });
                    data.settings.read().controls.mouse.dpad.landscape_on_left
                })),
            }),
            MenuItem::Option(OptionDef {
                text: "Swap Up and Down buttons".into(),
                val: cfg.mouse.dpad.swap_up_down,
                update_fn: Box::new(|do_swap, data| {
                    data.settings.update_ui(|cfg| {
                        *cfg.controls().mouse().dpad().swap_up_down() = do_swap;
                    });
                }),
                reset_fn: Some(Box::new(|data| {
                    data.settings.update_ui(|cfg| {
                        cfg.controls().mouse().dpad().swap_up_down = None;
                    });
                    data.settings.read().controls.mouse.dpad.swap_up_down
                })),
            }),
            MenuItem::Option(OptionDef {
                text: "Enable margin around dpad".into(),
                val: cfg.mouse.dpad.enable_margin,
                update_fn: Box::new(|enabled, data| {
                    data.settings.update_ui(|cfg| {
                        *cfg.controls().mouse().dpad().enable_margin() = enabled;
                    });
                }),
                reset_fn: Some(Box::new(|data| {
                    data.settings.update_ui(|cfg| {
                        cfg.controls().mouse().dpad().enable_margin = None;
                    });
                    data.settings.read().controls.mouse.dpad.enable_margin
                })),
            }),
            MenuItem::Option(OptionDef {
                text: "Enable dpad highlight".into(),
                val: cfg.mouse.dpad.enable_highlight,
                update_fn: Box::new(|enabled, data| {
                    data.settings.update_ui(|cfg| {
                        *cfg.controls().mouse().dpad().enable_highlight() = enabled;
                    });
                }),
                reset_fn: Some(Box::new(|data| {
                    data.settings.update_ui(|cfg| {
                        cfg.controls().mouse().dpad().enable_highlight = None;
                    });
                    data.settings.read().controls.mouse.dpad.enable_highlight
                })),
            }),
            MenuItem::Separator,
            MenuItem::text("Exit".into()),
        ],
    );

    Activity::new_base_boxed("controls settings", Menu::new(menu_config))
}
