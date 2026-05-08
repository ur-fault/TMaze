use std::{
    rc::Rc,
    sync::{mpsc, Arc},
    time::{Duration, Instant},
};

use cmaze::{
    algorithms::{
        region_generator::{DepthFirstSearch, RndKruskals},
        region_splitter::DefaultRegionSplitter,
        GeneratorRegistry, SplitterRegistry,
    },
    dims::*,
};

use crossterm::event::{read, KeyCode, KeyEvent, KeyEventKind};
// use indexmap::IndexMap;

use crate::{
    app::event::{EventReceiver, EventReceiverFn},
    data::SaveData,
    helpers::{constants::paths, on_off},
    logging::{self, AppLogger, LoggerOptions, UiLogs},
    renderer::{self, draw::Draw, CellContent, GMutView, Renderer},
    settings::{
        model::{Config, TerminalSchemeDef},
        theme::{SharedScheme, TerminalColorScheme, Theme, ThemeDefinition, ThemeResolver},
        ConfigSource, Settings,
    },
    ui,
};

#[cfg(feature = "sound")]
use crate::sound::{track::MusicTrack, SoundPlayer};

#[cfg(feature = "sound")]
use rodio::{self, Source};

use super::{
    activity::{Activities, Activity, ActivityResult, Change},
    event::Event,
    game,
    jobs::Qer,
    Jobs,
};

pub struct App {
    renderer: Renderer,
    activities: Activities,
    data: AppData,
    event_drain: mpsc::Receiver<Event>,
}

pub struct AppData {
    pub settings: Settings,
    pub save: SaveData,
    pub use_data: AppStateData,
    pub appearance: Appearance,
    pub screen_size: Dims,
    pub logs: UiLogs,
    pub registries: Registries,
    jobs: Jobs,
    pub event_sink: EventSink,
    pub event_receivers: Vec<EventReceiverFn>,

    app_start: Instant,
    read_only: bool,

    #[cfg(feature = "sound")]
    pub sound_player: SoundPlayer,
    #[cfg(feature = "sound")]
    bgm_track: Option<MusicTrack>,
}

impl AppData {
    pub fn from_start(&self) -> Duration {
        self.app_start.elapsed()
    }

    #[cfg(feature = "sound")]
    pub fn play_bgm(&mut self, track: MusicTrack) {
        if let Some(prev_track) = self.bgm_track {
            if prev_track == track {
                return;
            }
        }

        let cfg = &self.settings.read().audio;
        let volume = if cfg.global.enable && cfg.music.enable {
            cfg.global.volume * cfg.music.volume
        } else {
            0.0
        } as f32;

        self.sound_player.set_volume(volume);

        self.bgm_track = Some(track);
        let track = track.get_track().repeat_infinite();
        self.sound_player.play_track(Box::new(track));
    }

    pub fn queuer(&self) -> Qer {
        self.jobs.queuer()
    }

    pub fn is_ro(&self) -> bool {
        self.read_only
    }
}

pub struct Registries {
    pub region_splitters: SplitterRegistry,
    pub region_generator: GeneratorRegistry,
}

impl App {
    /// Create a new app with no activities
    ///
    /// This method intializes all of the needed components of the app.
    /// - Loads settings,
    /// - loads save data,
    /// - initializes the renderer,
    /// - initializes the sound player (if the feature is enabled),
    /// - initializes the logging system,
    /// - initializes the job queue,
    /// - initializes the registries,
    pub fn new(
        AppOptions {
            read_only,
            main_activity,
            config_source,
        }: AppOptions,
    ) -> Self {
        if !read_only {
            Self::prepare_dirs()
                .expect("Failed to prepare application directories. Please check permissions.");
        }

        let (event_sink, event_drain) = Self::init_event_sink();
        let mut event_receivers = vec![];

        let (settings, settings_errors, settings_warnings) =
            Settings::load(event_sink.clone(), &config_source);
        let config = settings.read();
        event_receivers.push(settings.register());

        let scheme = Appearance::load_scheme(&config);
        let renderer = Renderer::new(scheme).expect("failed to create renderer");

        let mut activities = Activities::empty();
        if let Some(activity) = main_activity {
            activities.push(activity);
        }

        let (logger, logs) = AppLogger::new_with_options(
            config.general.logging.normal,
            LoggerOptions::default()
                .read_only(read_only)
                .file_level(config.general.logging.file),
        );
        logger.init();

        if let Some(errors) = settings_errors {
            log::error!("Errors were encountered while loading the config.");
            for err in errors {
                log::error!(" - {}", err);
            }
        }

        if let Some(warnings) = settings_warnings {
            log::warn!("Warnings were encountered while loading the config.");
            for warn in warnings {
                log::warn!(" - {}", warn);
            }
        }

        let save = SaveData::load().expect("failed to load save data");
        let use_data = AppStateData::default();
        let jobs = Jobs::new();
        let app_start = Instant::now();
        let frame_size = renderer.frame_size();
        let registries = Registries {
            region_splitters: SplitterRegistry::with_default(
                Arc::new(DefaultRegionSplitter),
                "default",
            ),
            region_generator: {
                let mut reg =
                    GeneratorRegistry::with_default(Arc::new(RndKruskals), "rnd_kruskals");
                reg.register("dfs", Arc::new(DepthFirstSearch));
                reg
            },
        };

        log::info!("Loading theme");

        #[cfg(feature = "sound")]
        let sound_player = SoundPlayer::new(settings.clone());
        event_receivers.push(sound_player.register());

        let appearance = Appearance::new(&config);

        drop(config);

        Self {
            renderer,
            activities,
            event_drain,
            data: AppData {
                app_start,
                settings,
                save,
                use_data,
                appearance,
                screen_size: frame_size,
                jobs,
                event_sink,
                event_receivers,
                logs,
                registries,
                read_only,

                #[cfg(feature = "sound")]
                sound_player,
                #[cfg(feature = "sound")]
                bgm_track: None,
            },
        }
    }

    pub fn run(&mut self) -> Option<ActivityResult> {
        log::trace!("Starting main loop");

        let rem_events = 'mainloop: loop {
            while let Some(job) = self.data.jobs.pop() {
                log::trace!("Running job: {:?}", job.name().unwrap_or("<unnamed>"));
                job.call(&mut self.data);
            }

            let mut events = vec![];

            // FIXME: better polling strategy, IO will need faster response times
            let mut delay = Duration::from_millis(10);
            while let Ok(true) = crossterm::event::poll(delay) {
                let event = read().unwrap();

                self.renderer.on_event(&event);
                self.data.screen_size = self.renderer.frame_size();

                match event {
                    crossterm::event::Event::Key(KeyEvent {
                        code: KeyCode::F(3),
                        kind: KeyEventKind::Press,
                        ..
                    }) => self.switch_debug(),
                    event @ crossterm::event::Event::Mouse(_) => {
                        if self.data.settings.read().controls.mouse.enable {
                            events.push(Event::Term(event));
                        }
                    }
                    event => events.push(Event::Term(event)),
                }

                // just so we read all events in the frame
                delay = Duration::from_nanos(1)
            }

            // Read events from the drain
            while let Ok(event) = self.event_drain.try_recv() {
                events.push(event);
            }

            // Update handle the event receivers
            // (hack): due to borrow issues
            let mut receivers = std::mem::take(&mut self.data.event_receivers);
            for receiver in receivers.iter_mut() {
                for event in &events {
                    receiver(event, &mut self.data);
                }
            }
            self.data.event_receivers = receivers;

            while let Some(change) = match self.activities.active_mut() {
                Some(active) => active,
                None => break 'mainloop events,
            }
            .update(std::mem::take(&mut events), &mut self.data)
            {
                match change {
                    Change::Push(activity) => {
                        log::trace!(
                            "Pushed new activity '{}/{}'",
                            activity.source(),
                            activity.name()
                        );
                        self.activities.push(activity);
                    }
                    Change::Pop { n, res } => {
                        self.activities.pop_n(n);
                        events.push(Event::ActiveAfterPop(res));
                        log::trace!("Popped {} activities", n);
                    }
                    Change::PopUntil { name, res } => {
                        self.activities.pop_until(&name);
                        events.push(Event::ActiveAfterPop(res));
                        log::trace!("Popped until '{}'", name);
                    }
                    Change::Replace(activity) => self.activities.replace(activity),
                    Change::ReplaceAt { index, activity } => {
                        self.activities.replace_at(index, activity);
                    }
                }
            }

            let theme = &self.data.appearance.theme;
            self.renderer
                .frame()
                .mut_view()
                .fill(CellContent::styled(' ', theme.get("background")));

            match self
                .activities
                .active_mut()
                .expect("No active active")
                .screen()
                .draw(&mut self.renderer.frame().mut_view(), &theme)
            {
                Ok(_) => {}
                Err(ui::ScreenError::SmallScreen) => {
                    draw_small_screen_info(&mut self.renderer.frame().mut_view(), &theme)
                }
            }

            self.data
                .logs
                .draw_on(Dims(0, 0), &mut self.renderer.frame().mut_view(), &theme);

            // TODO: let activities show debug info and about the app itself
            // then we can draw it here

            self.renderer.show().unwrap();
        };

        log::trace!("Main loop ended");

        rem_events.into_iter().find_map(|e| match e {
            Event::ActiveAfterPop(Some(res)) => Some(res),
            _ => None,
        })
    }

    fn switch_debug(&mut self) {
        self.data.use_data.show_debug = !self.data.use_data.show_debug;
        self.data.logs.switch_debug(&self.data.settings.read());
        log::warn!(
            "Debug mode: {}",
            on_off(self.data.use_data.show_debug, false)
        );
    }

    fn prepare_dirs() -> std::io::Result<()> {
        for dir in paths::all_dirs() {
            std::fs::create_dir_all(&dir)?;
        }

        Ok(())
    }

    pub fn init_event_sink() -> (EventSink, mpsc::Receiver<Event>) {
        mpsc::channel()
    }

    pub fn activity_count(&self) -> usize {
        self.activities.len()
    }

    pub fn activities(&self) -> &Activities {
        &self.activities
    }

    pub fn activities_mut(&mut self) -> &mut Activities {
        &mut self.activities
    }

    pub fn active_name(&self) -> Option<&str> {
        self.activities.active().map(|a| a.name())
    }

    pub fn data(&self) -> &AppData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut AppData {
        &mut self.data
    }
}

pub struct AppOptions<'a> {
    pub read_only: bool,
    pub main_activity: Option<Activity>,
    pub config_source: ConfigSource<'a>,
}

pub type EventSink = mpsc::Sender<Event>;

#[derive(Default)]
pub struct AppStateData {
    // pub last_selected_preset: IndexMap<usize, usize>,
    pub show_debug: bool,
}

pub struct Appearance {
    theme: Theme,
    scheme: SharedScheme,
    resolver: ThemeResolver,
}

impl Appearance {
    pub fn new(config: &Config) -> Self {
        let resolver = init_theme_resolver();

        Self {
            theme: Self::load_theme(config, &resolver),
            scheme: Self::load_scheme(config),
            resolver,
        }
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    pub fn scheme(&self) -> &Rc<TerminalColorScheme> {
        &self.scheme
    }

    pub fn resolver(&self) -> &ThemeResolver {
        &self.resolver
    }
}

impl Appearance {
    fn load_theme(config: &Config, resolver: &ThemeResolver) -> Theme {
        match config.general.appearance.theme.as_str() {
            "" => resolver.resolve(&ThemeDefinition::parse_default()),
            name => match ThemeDefinition::load_by_name(name) {
                Ok(def) => resolver.resolve(&def),
                Err(err) => {
                    log::error!("Failed to load theme '{}': {}", name, err);
                    resolver.resolve(&ThemeDefinition::parse_default())
                }
            },
        }
    }

    fn load_scheme(config: &Config) -> SharedScheme {
        let scheme = config.general.appearance.terminal_scheme.clone();
        let scheme = match scheme {
            TerminalSchemeDef::Named(name) => match TerminalColorScheme::named(&name) {
                Some(scheme) => scheme,
                None => TerminalColorScheme::default(),
            },
            TerminalSchemeDef::Custom(scheme) => scheme,
        };
        Rc::new(scheme)
    }
}

pub fn init_theme_resolver() -> ThemeResolver {
    let mut resolver = ThemeResolver::new();

    resolver
        .link("default", "")
        .link("background", "")
        .link("empty", "");

    resolver
        .extend(ui::theme_resolver())
        .extend(game::game_theme_resolver())
        .extend(logging::logging_theme_resolver());

    resolver
}

fn draw_small_screen_info(frame: &mut GMutView, theme: &Theme) {
    let size = frame.size();
    frame.clear();
    frame.centered(Dims(size.0, 2), |f| {
        f.draw_aligned(
            renderer::draw::Align::TopCenter,
            "Screen is too small",
            theme["text"],
        );
        f.draw_aligned(
            renderer::draw::Align::BottomCenter,
            format!("actual size: {}x{}", size.0, size.1),
            theme["text"],
        );
    });
}
