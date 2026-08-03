use std::{iter::once, rc::Rc};

use cmaze::dims::Offset;

use crate::{
    app::{
        activity::ActivityHandlerExt as _, app::AppData, Activity, ActivityEvent, ActivityHandler,
        Change,
    },
    helpers::constants::paths::{config, theme},
    match_scheme_field, menu_actions_2,
    renderer::MouseGuard,
    settings::{
        model::{
            CameraMode, Logging, PartialTerminalSchemeDef, TerminalSchemeDef, UpdateCheckInterval,
        },
        theme::{PartialTerminalColorScheme, Rgb, TerminalColorScheme},
    },
    ui::{Menu, MenuConfig, MenuItemObj, Popup, Screen, Slider, SliderDisplay, Switch, Text},
    update_settings,
};

#[cfg(feature = "sound")]
use crate::sound::create_audio_settings;

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

                let themes = match themes {
                    Ok(themes) if themes.is_empty() => {
                        let theme = theme().to_string_lossy().into_owned();
                        return Popup::new(
                            "No themes found".into(),
                            vec![
                                format!("No theme files found in {}", theme),
                                "Please add a theme file to the themes directory.".into(),
                            ],
                        )
                        .to_base_activity("no themes popup");
                    }
                    Err(err) => {
                        return Popup::new("Error listing themes".into(), vec![err])
                            .to_base_activity("error listing themes popup")
                    }
                    Ok(themes) => themes,
                };
                let current_theme = data.settings.read().general.appearance.theme.clone();

                Menu::new(
                    MenuConfig::new(
                        "Theme",
                        themes
                            .iter()
                            .cloned()
                            .map(|theme| {
                                Text::new(theme.clone()).click(move |d| {
                                    update_settings!(
                                        d.settings,
                                        general.appearance.theme = theme.clone()
                                    );
                                    Some(Change::pop_top())
                                }) as MenuItemObj
                            })
                            .chain([Text::new("None").click(|d| {
                                update_settings!(
                                    d.settings,
                                    general.appearance.theme = String::new()
                                );
                                Some(Change::pop_top())
                            }) as MenuItemObj])
                            .chain([back_button()])
                            .collect::<Vec<_>>(),
                    )
                    .default(
                        themes
                            .iter()
                            .chain(once(&String::new()))
                            .position(|theme| theme == &current_theme)
                            .unwrap_or_default(),
                    ),
                )
                .to_base_activity("theme settings")
            }

            fn terminal_scheme_settings(data: &AppData) -> Activity {
                fn named_settings(data: &AppData) -> Activity {
                    let appearance = &data.settings.read().general.appearance;

                    Menu::new(
                        MenuConfig::new(
                            "Choose named scheme",
                            TerminalColorScheme::all_schemes()
                                .into_iter()
                                // FIXME(hack): cannot currently display large number of schemes in menu
                                .take(10)
                                .map(|scheme| {
                                    Text::new(scheme.to_string()).click(|d| {
                                        update_settings!(
                                            d.settings,
                                            general.appearance.terminal_scheme =
                                                PartialTerminalSchemeDef::Named((*scheme).into())
                                        );
                                        Some(Change::pop_top())
                                    }) as MenuItemObj
                                })
                                .chain(once(back_button()))
                                .collect::<Vec<_>>(),
                        )
                        .maybe_default(match &appearance.terminal_scheme {
                            TerminalSchemeDef::Named(original) => {
                                TerminalColorScheme::all_schemes()
                                    .iter()
                                    // FIXME(hack): cannot currently display large number of schemes in menu
                                    .take(10)
                                    .position(|scheme| *scheme == original)
                            }
                            _ => None,
                        }),
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
                    ) -> (&'a str, Rgb, Rc<dyn Fn(&mut AppData, Rgb) -> Rgb>) {
                        let scheme_field = match_scheme_field!(field, scheme,);

                        let field2 = field.to_string();
                        let fn_ = Rc::new(move |data: &mut AppData, color| {
                            data.settings.update_ui(|cfg| {
                                use crate::settings::model::PartialTerminalSchemeDef::*;
                                use PartialTerminalColorScheme as PTCS;

                                let scheme = match cfg.general().appearance().terminal_scheme() {
                                    mut_scheme @ Named(_) => {
                                        *mut_scheme = Custom(PTCS::default());
                                        match mut_scheme {
                                            Named(_) => unreachable!(),
                                            Custom(scheme) => scheme,
                                        }
                                    }
                                    Custom(scheme) => scheme,
                                };
                                *match_scheme_field!(field2.as_str(), scheme, &mut) = Some(color);
                            });

                            color
                        });

                        (title, scheme_field, fn_)
                    }

                    fn channel_field_item<'a>(
                        (title, color, update_global): (
                            &'a str,
                            (u8, u8, u8),
                            Rc<dyn Fn(&mut AppData, (u8, u8, u8)) -> (u8, u8, u8)>,
                        ),
                    ) -> MenuItemObj {
                        let title = title.to_string();

                        Text::new(&title).click(Box::new(move |_: &mut _| {
                            let shared_color = Rc::new(std::cell::Cell::new(color));
                            let update_global = update_global.clone();

                            let make_slider = |label: &'static str, channel: u8, initial: u8| {
                                let shared_color = shared_color.clone();
                                let update_global = update_global.clone();

                                Slider::new(label, initial, 0..=255, move |val, data| {
                                    let mut c = shared_color.get();
                                    match channel {
                                        0 => c.0 = val,
                                        1 => c.1 = val,
                                        2 => c.2 = val,
                                        _ => unreachable!(),
                                    }
                                    shared_color.set(c);
                                    update_global(data, c);
                                })
                                .display(SliderDisplay::Value)
                                    as MenuItemObj
                            };

                            let config = MenuConfig::new(
                                &title,
                                vec![
                                    make_slider("Red", 0, color.0),
                                    make_slider("Green", 1, color.1),
                                    make_slider("Blue", 2, color.2),
                                ],
                            );

                            Change::push(
                                Menu::new(config).to_base_activity("scheme color settings"),
                            )
                        }))
                    }

                    Menu::new(MenuConfig::new(
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
                        .map(channel_field_item)
                        .chain(once(back_button()))
                        .collect::<Vec<_>>(),
                    ))
                    .to_base_activity("custom scheme settings")
                }

                let default = match &data.settings.read().general.appearance.terminal_scheme {
                    TerminalSchemeDef::Named(_) => 0,
                    TerminalSchemeDef::Custom(_) => 1,
                };
                Menu::new(
                    MenuConfig::new(
                        "Terminal Color Scheme",
                        menu_actions_2!(
                            "Named" -> data => Change::push(named_settings(data)),
                            "Custom" -> data => Change::push(custom_settings(data)),
                            "Back" -> _ => Change::pop_top(),
                        ),
                    )
                    .default(default),
                )
                .to_base_activity("terminal scheme settings")
            }

            Menu::new(MenuConfig::new(
                "Appearance",
                menu_actions_2!(
                    "Theme" -> data => Change::push(theme_settings(data)),
                    "Terminal scheme" -> data => Change::push(terminal_scheme_settings(data)),
                    "Back" -> _ => Change::pop_top(),
                ),
            ))
            .to_base_activity("appearance settings")
        }

        fn logging_settings() -> Activity {
            #[derive(Clone, Copy)]
            enum LoggingDst {
                Normal,
                Debug,
                File,
            }

            fn logging_dst_settings(dst: LoggingDst, data: &AppData) -> Activity {
                use log::Level;
                use LoggingDst::*;

                const LOG_LEVELS: [(&str, log::Level); 5] = [
                    ("Error", Level::Error),
                    ("Warn", Level::Warn),
                    ("Info", Level::Info),
                    ("Debug", Level::Debug),
                    ("Trace", Level::Trace),
                ];

                const fn level_to_index(level: Level) -> usize {
                    match level {
                        Level::Error => 0,
                        Level::Warn => 1,
                        Level::Info => 2,
                        Level::Debug => 3,
                        Level::Trace => 4,
                    }
                }

                let title = match dst {
                    Normal => "UI logging",
                    Debug => "UI Debug logging",
                    File => "File logging",
                };

                let Logging {
                    normal,
                    debug,
                    file,
                } = &data.settings.read().general.logging;

                Menu::new(
                    MenuConfig::new(
                        "Logging level",
                        LOG_LEVELS
                            .into_iter()
                            .map(|(name, lvl)| {
                                Text::new(name).click(move |data| {
                                    data.settings.update_ui(|cfg| {
                                        let logging = cfg.general().logging();
                                        *match dst {
                                            Normal => logging.normal(),
                                            Debug => logging.debug(),
                                            File => logging.file(),
                                        } = lvl;
                                    });
                                    Some(Change::pop_top())
                                }) as MenuItemObj
                            })
                            .chain([back_button()])
                            .collect::<Vec<_>>(),
                    )
                    .default(level_to_index(match dst {
                        Normal => *normal,
                        Debug => *debug,
                        File => *file,
                    })),
                )
                .to_base_activity(format!("{} settings", title.to_lowercase()))
            }

            Menu::new(MenuConfig::new(
                    "Logging settings",
            menu_actions_2!(
                "UI logging" -> data => Change::push(logging_dst_settings(LoggingDst::Normal, data)),
                "UI Debug logging" -> data => Change::push(logging_dst_settings(LoggingDst::Debug, data)),
                "File logging" -> data => Change::push(logging_dst_settings(LoggingDst::File, data)),
                "Back" -> _ => Change::pop_top(),
            ),
            ))
            .to_base_activity("logging settings")
        }

        Menu::new(MenuConfig::new(
            "General settings",
            menu_actions_2!(
                "Appearance" -> _ => Change::push(appearance_settings()),
                "Logging" -> _ => Change::push(logging_settings()),
                "Back" -> _ => Change::pop_top(),
            ),
        ))
        .to_base_activity("general settings")
    }

    fn game_settings(data: &mut AppData) -> Activity {
        fn view_settings(data: &mut AppData) -> Activity {
            fn free_follow_settings() -> Activity {
                fn offset_settings(is_y: bool) -> Activity {
                    fn abs_settings(data: &mut AppData, is_y: bool) -> Activity {
                        let val = match data.settings.read().game.view.camera_mode {
                            CameraMode::EdgeFollow { x, y } => {
                                match match is_y {
                                    true => y,
                                    false => x,
                                } {
                                    Offset::Abs(v) => v,
                                    Offset::Rel(_) => 0,
                                }
                            }
                            _ => 0,
                        };

                        let config = MenuConfig::new(
                            "Absolute Offset",
                            vec![Slider::new(
                                if is_y { "y" } else { "x" },
                                val,
                                0..=100,
                                move |v, d| {
                                    d.settings.update_ui(|cfg| {
                                        let (mut x, mut y) = match cfg.game().view().camera_mode {
                                            Some(CameraMode::EdgeFollow { x, y }) => (x, y),
                                            _ => (Offset::Abs(0), Offset::Abs(0)),
                                        };

                                        if is_y {
                                            y = Offset::Abs(v);
                                        } else {
                                            x = Offset::Abs(v);
                                        }

                                        *cfg.game().view().camera_mode() =
                                            CameraMode::EdgeFollow { x, y };
                                    });
                                },
                            ) as MenuItemObj],
                        );

                        Activity::new_base_boxed("free follow settings", Menu::new(config))
                    }

                    fn rel_settings() -> Activity {
                        todo!("Relative offset settings not implemented yet")
                    }

                    Menu::new(MenuConfig::new(
                        "Offset",
                        menu_actions_2!(move
                            "Absolute" -> d => Change::push(abs_settings(d, is_y)),
                            "Relative" -> _ => Change::push(rel_settings()),
                            "Back" -> _ => Change::pop_top(),
                        ),
                    ))
                    .to_base_activity("")
                }

                Menu::new(MenuConfig::new(
                    "Free follow settings",
                    menu_actions_2!(
                        "x offset" -> _ => Change::push(offset_settings(false)),
                        "y offset" -> _ => Change::push(offset_settings(true)),
                        "Back" -> _ => Change::pop_top(),
                    ),
                ))
                .to_base_activity("")
            }

            let settings = &data.settings.read().game.view;

            let config = MenuConfig::new(
                "Game View Settings",
                vec![
                    Text::new("Camera Mode").click(|_| {
                        Some(Change::push(
                            Menu::new(MenuConfig::new(
                                "Camera Mode",
                                menu_actions_2!(
                                    "Follow player" -> d => {
                                        update_settings!(d.settings, game.view.camera_mode = CameraMode::CloseFollow);
                                        Change::pop_top()
                                    },
                                    "Free" -> _ => Change::push(free_follow_settings()),
                                    "Back" -> _ => Change::pop_top(),
                                )
                            ))
                            .to_base_activity("camera mode settings"),
                        ))
                    }) as MenuItemObj,
                    Slider::new(
                        "Camera smoothing",
                        1.0 - settings.camera_smoothing,
                        0.0..=0.5,
                        |v, d| update_settings!(d.settings, game.view.camera_smoothing = 1.0 - v)
                    ),
                    Slider::new(
                        "Player smoothing",
                        1.0 - settings.player_smoothing,
                        0.0..=0.5,
                        |v, d| update_settings!(d.settings, game.view.player_smoothing = 1.0 - v)
                    ),
                    Text::new("Viewport margin (todo)"),
                    back_button(),
                ],
            );

            Activity::new_base_boxed("view settings", Menu::new(config))
        }

        let settings = &data.settings.read().game;

        let config = MenuConfig::new(
            "Game settings",
            vec![
                Switch::new("Slow", settings.slow, |enabled, data| {
                    update_settings!(data.settings, game.slow = enabled)
                }) as MenuItemObj,
                Switch::new(
                    "Don't auto advance up",
                    settings.disable_tower_auto_up,
                    |enabled, d| update_settings!(d.settings, game.disable_tower_auto_up = enabled),
                ),
                Text::new("View").click(|d| Some(Change::push(view_settings(d)))),
                Text::new("Content (todo)"),
                back_button(),
            ],
        );

        Activity::new_base_boxed("game settings", Menu::new(config))
    }

    fn control_settings() -> Activity {
        fn mouse_settings(data: &mut AppData) -> Activity {
            fn dpad_settings(data: &mut AppData) -> Activity {
                let cfg = &data.settings.read().controls.mouse.dpad;

                let menu_config = MenuConfig::new(
                    "DPad settings",
                    [
                        Switch::new("Enable", cfg.enable, |enabled, d| {
                            update_settings!(d.settings, controls.mouse.dpad.enable = enabled);
                        }) as MenuItemObj,
                        Switch::new("Left-handed", cfg.landscape_on_left, |is_on_left, d| {
                            update_settings!(
                                d.settings,
                                controls.mouse.dpad.landscape_on_left = is_on_left
                            );
                        }),
                        Switch::new(
                            "Swap Up and Down buttons",
                            cfg.swap_up_down,
                            |do_swap, d| {
                                update_settings!(
                                    d.settings,
                                    controls.mouse.dpad.swap_up_down = do_swap
                                );
                            },
                        ),
                        Switch::new("Enable margin", cfg.enable_margin, |enabled, d| {
                            update_settings!(
                                d.settings,
                                controls.mouse.dpad.enable_margin = enabled
                            );
                        }),
                        Switch::new("Enable highlight", cfg.enable_highlight, |enabled, d| {
                            update_settings!(
                                d.settings,
                                controls.mouse.dpad.enable_highlight = enabled
                            );
                        }),
                        Text::new("Space (todo)"),
                        Text::new("Min size (todo)"),
                        Text::new("Max size (todo)"),
                        back_button(),
                    ],
                );

                Activity::new_base_boxed("dpad settings", Menu::new(menu_config))
            }

            let cfg = &data.settings.read().controls.mouse;

            let menu_config = MenuConfig::new(
                "Controls settings",
                [
                    Switch::new("Enable mouse input", cfg.enable, |enabled, data| {
                        update_settings!(data.settings, controls.mouse.enable = enabled);
                    }) as MenuItemObj,
                    Text::new("DPad").click(|d| Some(Change::push(dpad_settings(d)))),
                    back_button(),
                ],
            );

            Activity::new_base_boxed("mouse settings", Menu::new(menu_config))
        }

        Menu::new(MenuConfig::new(
            "Control settings",
            menu_actions_2!(
                "Mouse" -> data => Change::push(mouse_settings(data)),
                "Back" -> _ => Change::pop_top(),
            ),
        ))
        .to_base_activity("control settings")
    }

    fn update_settings() -> Activity {
        fn interval_settings(data: &AppData) -> Activity {
            use UpdateCheckInterval::*;

            let default = match data.settings.read().updates.check_interval {
                Never => 0,
                Daily => 1,
                Weekly => 2,
                Monthly => 3,
                Yearly => 4,
                Always => 5,
            };

            Menu::new(
                MenuConfig::new(
                    "Check interval",
                    vec![
                        ("Never", Never),
                        ("Daily", Daily),
                        ("Weekly", Weekly),
                        ("Monthly", Monthly),
                        ("Yearly", Yearly),
                        ("Always", Always),
                    ]
                    .into_iter()
                    .map(|(name, int)| {
                        Text::new(name).click(move |data| {
                            update_settings!(data.settings, updates.check_interval = int);
                            Some(Change::pop_top())
                        }) as MenuItemObj
                    })
                    .chain(once(back_button()))
                    .collect::<Vec<_>>(),
                )
                .default(default),
            )
            .to_base_activity("update check interval")
        }

        Menu::new(MenuConfig::new(
            "Update settings",
            menu_actions_2!(
                "Check interval" -> data => Change::push(interval_settings(data)),
                "Back" -> _ => Change::pop_top(),
            ),
        ))
        .to_base_activity("update settings")
    }

    fn reset_settings_dialog() -> Activity {
        Menu::new(MenuConfig::new(
            "Are you sure?",
            vec![
                Text::new("No").click(|_| Some(Change::pop_top())),
                Text::new("Yes").click(|data| {
                    data.settings.reset();
                    Some(Change::pop_top())
                }) as MenuItemObj,
            ],
        ))
        .to_base_activity("reset settings")
    }

    fn back_button() -> MenuItemObj {
        Text::new("Back").click(|_| Some(Change::pop_top()))
    }

    Menu::new(MenuConfig::new("Settings",
        menu_actions_2!(
            "General" -> _ => Change::push(general_settings()),
            "Game" -> data => Change::push(game_settings(data)),
            "Controls" -> _ => Change::push(control_settings()),
            "Updates" -> _ => Change::push(update_settings()),
            "Audio" on "sound" -> data => Change::push(create_audio_settings(data)),
            "Other settings" -> _ => Change::push(OtherSettingsPopup::new().to_base_activity("other settings")),
            "Restore defaults" -> _ => Change::push(reset_settings_dialog()),
            "Back" -> _ => Change::pop_top(),
        ),
    ))
    .to_base_activity("settings")
}
