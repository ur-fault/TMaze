use crate::{
    app::{
        activity::ActivityHandlerExt as _, app::AppData, Activity, ActivityEvent, ActivityHandler,
        Change,
    },
    helpers::constants::paths::{config, theme},
    menu_actions,
    renderer::MouseGuard,
    sound::create_audio_settings,
    ui::{menu_result, simple_menu, Menu, MenuConfig, MenuItem, OptionDef, Popup, Screen},
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
    fn update(&mut self, events: Vec<ActivityEvent>, data: &mut AppData) -> Option<Change> {
        self.0.update(events, data)
    }

    fn screen(&mut self) -> &mut dyn Screen {
        &mut self.0
    }
}

pub fn create_settings_activity() -> Activity {
    fn general_settings() -> Activity {
        fn appearance_settings() -> Activity {
            fn theme_settings(data: &AppData) -> Activity {
                use std::{fs::read_dir, path::Path};

                let themes = match read_dir(theme()) {
                    Ok(iter) => Ok(iter
                        .filter_map(|entry| {
                            const SUPPORTED_EXTENSIONS: &[&str] = &["json", "json5"];

                            let entry = entry.ok()?;
                            if !entry.file_type().ok()?.is_file() {
                                return None;
                            }

                            let filename = entry.file_name();
                            let filename = Path::new(&filename);
                            let extension = filename.extension()?.to_str()?;
                            if !SUPPORTED_EXTENSIONS.contains(&extension) {
                                return None;
                            }

                            Some(filename.to_str()?.to_string())
                        })
                        .collect::<Vec<_>>()),
                    Err(err) => Err(err.to_string()),
                };

                let menu = match themes {
                    Ok(themes) if themes.is_empty() => {
                        return Activity::new_base_boxed(
                            "theme settings",
                            Popup::new(
                                "No themes found".into(),
                                vec![
                                    format!(
                                        "No theme files found in {}",
                                        theme().to_string_lossy()
                                    ),
                                    "Please add a theme file to the themes directory.".into(),
                                ],
                            ),
                        )
                    }
                    Err(err) => {
                        return Activity::new_base_boxed(
                            "err theme settings",
                            Popup::new("Error listing themes".into(), vec![err]),
                        )
                    }

                    Ok(themes) => {
                        let current_theme = data.settings.read().general.appearance.theme.clone();
                        let selected = themes.iter().position(|t| *t == current_theme);

                        let themes = themes
                            .into_iter()
                            .map(|theme| MenuItem::text(theme.into()))
                            .collect::<Vec<_>>();

                        MenuConfig::new("Select a theme", themes).maybe_default(selected)
                    }
                };

                struct ThemesActivity {
                    menu: Menu,
                }

                impl ActivityHandler for ThemesActivity {
                    fn update(
                        &mut self,
                        events: Vec<ActivityEvent>,
                        data: &mut AppData,
                    ) -> Option<Change> {
                        match self.menu.update(events, data)? {
                            Change::Pop {
                                res: Some(result), ..
                            } => {
                                let index = *result
                                    .downcast::<usize>()
                                    .expect("menu should return index");
                                let MenuItem::Text { text, .. } =
                                    self.menu.config().options.get(index).unwrap()
                                else {
                                    panic!("menu should return index of a text item");
                                };

                                data.settings.update_ui(|cfg| {
                                    *cfg.general().appearance().theme() = text.as_ref_cow().into();
                                });

                                None
                            }
                            res => Some(res),
                        }
                    }

                    fn screen(&mut self) -> &mut dyn Screen {
                        &mut self.menu
                    }
                }

                Activity::new_base_boxed(
                    "theme settings",
                    ThemesActivity {
                        menu: Menu::new(menu),
                    },
                )
            }

            Activity::new_base_boxed(
                "appearance settings",
                simple_menu(
                    "Appearance settings",
                    menu_actions!(
                        "Theme" -> data => Change::push(theme_settings(data)),
                        // "Terminal scheme" -> data => Change::push(terminal_scheme_settings(data)),
                        "Back" -> _ => Change::pop_top(),
                    ),
                ),
            )
        }

        fn logging_settings() -> Activity {
            enum LoggingDst {
                Normal,
                Debug,
                File,
            }

            fn logging_dst_settings(dst: LoggingDst, data: &AppData) -> Activity {
                const LOG_LEVELS: [(&str, log::Level); 5] = [
                    ("Error", log::Level::Error),
                    ("Warn", log::Level::Warn),
                    ("Info", log::Level::Info),
                    ("Debug", log::Level::Debug),
                    ("Trace", log::Level::Trace),
                ];

                const fn level_to_index(level: log::Level) -> usize {
                    match level {
                        log::Level::Error => 0,
                        log::Level::Warn => 1,
                        log::Level::Info => 2,
                        log::Level::Debug => 3,
                        log::Level::Trace => 4,
                    }
                }

                let title = match dst {
                    LoggingDst::Normal => "UI logging",
                    LoggingDst::Debug => "UI Debug logging",
                    LoggingDst::File => "File logging",
                };

                struct LoggingDstActivity {
                    dst: LoggingDst,
                    menu: Menu,
                }

                impl ActivityHandler for LoggingDstActivity {
                    fn update(
                        &mut self,
                        events: Vec<ActivityEvent>,
                        data: &mut AppData,
                    ) -> Option<Change> {
                        match self.menu.update(events, data)? {
                            Change::Pop {
                                n: 1,
                                res: Some(res),
                            } => {
                                let level = LOG_LEVELS[menu_result(res)].1;
                                data.settings.update_ui(|cfg| {
                                    let logging = cfg.general().logging();
                                    match self.dst {
                                        LoggingDst::Normal => *logging.normal() = level,
                                        LoggingDst::Debug => *logging.debug() = level,
                                        LoggingDst::File => *logging.file() = level,
                                    }
                                });
                                Some(Change::pop_top())
                            }
                            change => Some(change),
                        }
                    }

                    fn screen(&mut self) -> &mut dyn Screen {
                        &mut self.menu
                    }
                }

                let logging = &data.settings.read().general.logging;

                let menu = MenuConfig::new_from_strings(
                    title,
                    LOG_LEVELS
                        .iter()
                        .map(|(name, _)| String::from(*name))
                        .collect::<Vec<_>>(),
                )
                .default(match dst {
                    LoggingDst::Normal => level_to_index(logging.normal),
                    LoggingDst::Debug => level_to_index(logging.debug),
                    LoggingDst::File => level_to_index(logging.file),
                })
                .counted();

                Activity::new_base_boxed(
                    format!("{} settings", title.to_lowercase()),
                    LoggingDstActivity {
                        dst,
                        menu: Menu::new(menu),
                    },
                )
            }

            simple_menu(
            "Logging",
            menu_actions!(
                "UI logging" -> data => Change::push(logging_dst_settings(LoggingDst::Normal, data)),
                "UI Debug logging" -> data => Change::push(logging_dst_settings(LoggingDst::Debug, data)),
                "File logging" -> data => Change::push(logging_dst_settings(LoggingDst::File, data)),
                "Back" -> _ => Change::pop_top(),
            ),
        )
        .to_base_activity("logging settings")
        }

        Activity::new_base_boxed(
            "general settings",
            simple_menu(
                "General settings",
                menu_actions!(
                    "Appearance" -> _ => Change::push(appearance_settings()),
                    "Logging" -> _ => Change::push(logging_settings()),
                    "Back" -> _ => Change::pop_top(),
                ),
            ),
        )
    }

    fn control_settings(data: &mut AppData) -> Activity {
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

    simple_menu(
        "Settings",
        menu_actions!(
            "General" -> _ => Change::push(general_settings()),
            "Audio" on "sound" -> data => Change::push(create_audio_settings(data)),
            "Controls" -> data => Change::push(control_settings(data)),
            "Other settings" -> _ => Change::push(OtherSettingsPopup::new().to_base_activity("other settings")),
            "Back" -> _ => Change::pop_top(),
        ),
    )
    .to_base_activity("settings")
}
