use std::{cell::RefCell, rc::Rc};

use crate::{
    app::{
        activity::ActivityHandlerExt as _, app::AppData, Activity, ActivityEvent, ActivityHandler,
        Change,
    },
    helpers::constants::paths::{config, theme},
    match_scheme_field, menu_actions,
    renderer::MouseGuard,
    settings::{
        model::{PartialTerminalSchemeDef, TerminalSchemeDef},
        theme::{Rgb, TerminalColorScheme},
    },
    sound::create_audio_settings,
    ui::{
        menu_result, simple_menu, simple_menu_ex, Menu, MenuConfig, MenuItem, OptionDef, Popup,
        Screen, SimpleMenuOptions, SliderDef,
    },
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

            fn terminal_scheme_settings(data: &AppData) -> Activity {
                fn named_settings(data: &AppData) -> Activity {
                    let appearance = &data.settings.read().general.appearance;

                    simple_menu_ex(
                        "Choose named scheme",
                        TerminalColorScheme::all_schemes()
                            .into_iter()
                            // FIXME(hack): cannot currently display large number of schemes in menu
                            .take(10)
                            .map(|scheme| {
                                (
                                    MenuItem::static_text(*scheme),
                                    Box::new(move |data: &mut AppData| -> Change {
                                        data.settings.update_ui(|cfg| {
                                            *cfg.general().appearance().terminal_scheme() =
                                                PartialTerminalSchemeDef::Named((*scheme).into())
                                        });
                                        Change::pop_top()
                                    })
                                        as Box<dyn Fn(&mut AppData) -> Change + 'static>,
                                )
                            })
                            .collect(),
                        SimpleMenuOptions {
                            default: match &appearance.terminal_scheme {
                                TerminalSchemeDef::Named(original) => {
                                    TerminalColorScheme::all_schemes()
                                        .iter()
                                        // FIXME(hack): cannot currently display large number of schemes in menu
                                        .take(10)
                                        .position(|scheme| *scheme == original)
                                }
                                _ => None,
                            },
                        },
                    )
                    .to_base_activity("named scheme settings")
                }

                fn custom_settings(data: &AppData) -> Activity {
                    let scheme = match &data.settings.read().general.appearance.terminal_scheme {
                        TerminalSchemeDef::Custom(scheme) => scheme.clone(),
                        _ => TerminalColorScheme::default(),
                    };

                    fn field<'a>(
                        field: &'a str,
                        title: &'a str,
                        scheme: &TerminalColorScheme,
                    ) -> (&'a str, &'a str, Rgb, Rc<dyn Fn(&mut AppData, Rgb) -> Rgb>)
                    {
                        let scheme_field = match_scheme_field!(field, scheme,);

                        let field2 = field.to_string();
                        let fn_ = Rc::new(move |data: &mut AppData, color| {
                            data.settings.update_ui(|cfg| {
                                let PartialTerminalSchemeDef::Custom(scheme) =
                                    cfg.general().appearance().terminal_scheme()
                                else {
                                    panic!()
                                };
                                *match_scheme_field!(field2.as_str(), scheme, &mut) = Some(color);
                            });

                            color
                        });

                        (field, title, scheme_field, fn_)
                    }

                    fn fun_name<'a>(
                        (name, title, color, fn_): (
                            &'a str,
                            &'a str,
                            (u8, u8, u8),
                            Rc<dyn Fn(&mut AppData, (u8, u8, u8)) -> (u8, u8, u8)>,
                        ),
                    ) -> (MenuItem, Box<dyn Fn(&mut AppData) -> Change + 'a>) {
                        (
                            MenuItem::text(name.into()),
                            Box::new(move |_: &mut _| {
                                let fn2 = fn_.clone();

                                #[derive(Clone)]
                                struct ColorState {
                                    color: Rc<RefCell<Rgb>>,
                                    set: Rc<dyn Fn(&mut AppData, Rgb) -> Rgb>,
                                }

                                impl ColorState {
                                    fn red(&self, data: &mut AppData, r: i32) -> i32 {
                                        self.set(data, 0, r)
                                    }

                                    fn green(&self, data: &mut AppData, g: i32) -> i32 {
                                        self.set(data, 1, g)
                                    }

                                    fn blue(&self, data: &mut AppData, b: i32) -> i32 {
                                        self.set(data, 2, b)
                                    }

                                    fn set(&self, data: &mut AppData, i: usize, val: i32) -> i32 {
                                        assert!(i < 3);
                                        let mut borrow = self.color.borrow_mut();
                                        *match i {
                                            0 => &mut borrow.0,
                                            1 => &mut borrow.1,
                                            2 => &mut borrow.2,
                                            _ => unreachable!(),
                                        } = val as u8;
                                        (self.set)(data, *borrow);
                                        val
                                    }
                                }

                                let color_state = ColorState {
                                    color: Rc::new(RefCell::new(color)),
                                    set: fn2,
                                };

                                let slider =
                                    |f: fn(&ColorState, &mut AppData, i32) -> i32,
                                     clf: Box<dyn Fn(Rgb) -> u8>,
                                     text: &'static str| {
                                        let u1 = color_state.clone();
                                        let u2 = color_state.clone();

                                        MenuItem::Slider(SliderDef {
                                            text: text.into(),
                                            val: clf(color) as i32,
                                            range: 0..=255,
                                            update_fn: Box::new(move |val, data| {
                                                f(&u1, data, val);
                                            }),
                                            reset_fn: Some(Box::new(move |data| {
                                                f(&u2, data, clf(color) as i32)
                                            })),
                                            as_num: true,
                                        })
                                    };

                                use ColorState as CS;
                                let config = MenuConfig::new(
                                    title,
                                    vec![
                                        slider(CS::red, Box::new(|c| c.0), "Red"),
                                        slider(CS::green, Box::new(|c| c.1), "Green"),
                                        slider(CS::blue, Box::new(|c| c.2), "Blue"),
                                    ],
                                );
                                Change::push(
                                    Menu::new(config).to_base_activity("scheme color settings"),
                                )
                            }) as Box<dyn Fn(&mut _) -> _>,
                        )
                    }

                    simple_menu(
                        "Custom scheme",
                        [
                            field("primary_fg", "Primary Fg", &scheme),
                            field("primary_bg", "Primary Bg", &scheme),
                            field("black", "Black", &scheme),
                            field("dark_grey", "Dark grey", &scheme),
                            field("red", "Red", &scheme),
                            field("dark_red", "Dark red", &scheme),
                            field("green", "Green", &scheme),
                            field("dark_green", "Dark green", &scheme),
                            field("yellow", "Yellow", &scheme),
                            field("dark_yellow", "Dark Yellow", &scheme),
                            field("blue", "Blue", &scheme),
                            field("dark_blue", "Dark Blue", &scheme),
                            field("magenta", "Magenta", &scheme),
                            field("dark_magenta", "Dark Magenta", &scheme),
                            field("cyan", "Cyan", &scheme),
                            field("dark_cyan", "Dark_cyan", &scheme),
                            field("white", "White", &scheme),
                            field("grey", "Grey", &scheme),
                        ]
                        .into_iter()
                        .map(fun_name)
                        .collect(),
                    )
                    .to_base_activity("custom scheme settings")
                }

                let menu = simple_menu_ex(
                    "Terminal Color Scheme",
                    menu_actions!(
                        "Named" -> data => Change::push(named_settings(data)),
                        "Custom" -> data => Change::push(custom_settings(data)),
                        "Back" -> _ => Change::pop_top(),
                    ),
                    SimpleMenuOptions {
                        default: Some(
                            match &data.settings.read().general.appearance.terminal_scheme {
                                TerminalSchemeDef::Named(_) => 0,
                                TerminalSchemeDef::Custom(_) => 1,
                            },
                        ),
                    },
                );

                menu.to_base_activity("terminal scheme settings")
            }

            Activity::new_base_boxed(
                "appearance settings",
                simple_menu(
                    "Appearance settings",
                    menu_actions!(
                        "Theme" -> data => Change::push(theme_settings(data)),
                        "Terminal scheme" -> data => Change::push(terminal_scheme_settings(data)),
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
                                    *match self.dst {
                                        LoggingDst::Normal => logging.normal(),
                                        LoggingDst::Debug => logging.debug(),
                                        LoggingDst::File => logging.file(),
                                    } = level;
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
