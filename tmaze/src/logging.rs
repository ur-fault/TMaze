use std::{
    io::Write,
    ops,
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard, RwLock},
    time::Duration,
};

use cmaze::dims::Dims;
use log::{Log, Metadata, Record};
use unicode_width::UnicodeWidthStr;

use crate::{
    helpers::constants::paths,
    renderer::{self, drawable::Drawable},
    settings::{
        theme::{Color, NamedColor, Style, Theme},
        Settings,
    },
};

const DEFAULT_DECAY: Duration = Duration::from_secs(5);
const DEFAULT_MAX_VISIBLE: usize = 5;

#[derive(Clone)]
pub struct Message {
    pub level: log::Level,
    pub pushed: std::time::Instant, // TODO: rename to `timestamp`
    pub message: String,
    pub source: String,
}

struct Logs {
    logs: [Vec<Message>; 5], // there are 5 levels
}

impl Logs {
    fn push(&mut self, message: Message) {
        self.logs[message.level as usize - 1].insert(0, message);
    }

    fn clear(&mut self, decay: Duration) {
        let now = std::time::Instant::now();
        for level in self.logs.iter_mut() {
            level.retain(|msg| now.duration_since(msg.pushed) < decay);
        }
    }
}

#[derive(Clone)]
struct LogsView {
    logs: Arc<Mutex<Logs>>,
}

impl ops::Deref for LogsView {
    type Target = Mutex<Logs>;

    fn deref(&self) -> &Self::Target {
        &self.logs
    }
}

pub struct LogsIter<'a> {
    logs: MutexGuard<'a, Logs>,
    level: usize,
    index: usize,
    min_level: log::Level,
}

impl<'a> Iterator for LogsIter<'a> {
    type Item = Message;

    fn next(&mut self) -> Option<Self::Item> {
        while self.level < self.logs.logs.len() && self.index >= self.logs.logs[self.level].len() {
            self.level += 1;
            self.index = 0;
        }

        if self.level >= self.logs.logs.len() || self.level > self.min_level as usize {
            return None;
        }

        let log = self.logs.logs[self.level][self.index].clone();
        self.index += 1;
        Some(log)
    }
}

pub struct UiLogs {
    logs: LogsView,
    pub decay: Duration,
    pub max_visible: usize,
    pub debug: RwLock<bool>,
    pub min_level: RwLock<log::Level>,
}

impl UiLogs {
    pub fn iter(&self) -> impl Iterator<Item = Message> + '_ {
        let mut logs = self.borrow_mut_logs();
        logs.clear(self.decay);

        LogsIter {
            logs,
            level: 0,
            index: 0,
            min_level: *self.min_level.read().unwrap(),
        }
    }

    pub fn switch_debug(&self, settings: &Settings) {
        let mut debug = self.debug.write().unwrap();
        *debug = !*debug;

        if *debug {
            *self.min_level.write().unwrap() = settings.get_debug_logging_level();
        } else {
            *self.min_level.write().unwrap() = settings.get_logging_level();
        }
    }

    fn borrow_mut_logs(&self) -> MutexGuard<Logs> {
        self.logs.lock().expect("a thread holding log panicked")
    }
}

impl Drawable<&Theme> for UiLogs {
    fn draw(&self, pos: Dims, frame: &mut renderer::Frame, theme: &Theme) {
        let [msg_style, source_style, extra] =
            theme.extract(["log.message", "log.source", "log.extra"]);

        // NOTE: please don't call any `log!` macro in this loop, it will cause a deadlock
        for (i, log) in self.iter().take(self.max_visible).enumerate() {
            let color = match log.level {
                log::Level::Error => NamedColor::Red,
                log::Level::Warn => NamedColor::Yellow,
                log::Level::Info => NamedColor::White,
                log::Level::Debug => NamedColor::Blue,
                log::Level::Trace => NamedColor::Grey,
            };

            let indicator_style = Style::fg(Color::Named(color));

            let y = pos.1 + i as i32;
            let len = log.source.width() + 4 + log.message.width();

            let src_x = frame.size.0 - len as i32 - 2;
            let msg_x = src_x + log.source.width() as i32 + 4;
            let src_pos = Dims(src_x, y);
            let msg_pos = Dims(msg_x, y);

            const INDICATOR_CHAR: char = '|';

            log.source.draw(src_pos, frame, source_style);
            "->".draw(Dims(msg_x - 3, y), frame, extra);
            log.message.draw(msg_pos, frame, msg_style);
            INDICATOR_CHAR.draw(Dims(frame.size.0 - 1, y), frame, indicator_style);
        }
    }
}

#[derive(Debug)]
pub struct LoggerOptions {
    pub decay: Duration,
    pub max_visible: usize,
    pub path: Option<PathBuf>,
    pub file_level: log::Level,
}

impl LoggerOptions {
    pub fn read_only(self, ro: bool) -> Self {
        Self {
            path: if ro { None } else { self.path },
            ..self
        }
    }

    pub fn file_level(mut self, level: log::Level) -> Self {
        self.file_level = level;
        self
    }
}

impl Default for LoggerOptions {
    fn default() -> Self {
        Self {
            decay: DEFAULT_DECAY,
            max_visible: DEFAULT_MAX_VISIBLE,
            path: Some(paths::log_file_path()),
            file_level: log::Level::Debug,
        }
    }
}

pub struct AppLogger {
    logs: LogsView,
    file: Option<Mutex<std::fs::File>>,
    pub file_level: RwLock<log::Level>,
}

impl AppLogger {
    pub fn new(min_level: log::Level) -> (Self, UiLogs) {
        Self::new_with_options(min_level, LoggerOptions::default())
    }

    pub fn new_with_options(min_level: log::Level, options: LoggerOptions) -> (Self, UiLogs) {
        let logs = LogsView {
            logs: Arc::new(Mutex::new(Logs {
                logs: Default::default(),
            })),
        };
        let logger = Self {
            logs: logs.clone(),
            file: options.path.map(|path| {
                let file = std::fs::File::options()
                    .create(true)
                    .append(true)
                    .open(&path);
                assert!(file.is_ok(), "Failed to open log file at {:?}", path);
                Mutex::new(file.unwrap())
            }),
            file_level: RwLock::new(options.file_level),
        };
        let ui_logs = UiLogs {
            logs,
            decay: options.decay,
            max_visible: options.max_visible,
            debug: RwLock::new(false),
            min_level: RwLock::new(min_level),
        };

        (logger, ui_logs)
    }

    pub fn init(self) {
        let log_ref = Box::<_>::leak(Box::new(self));
        log::set_logger(log_ref).unwrap();
        log::set_max_level(log::LevelFilter::Trace);
    }

    fn borrow_mut_logs(&self) -> MutexGuard<Logs> {
        self.logs.lock().expect("a thread holding log panicked")
    }
}

impl Log for AppLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        self.borrow_mut_logs().push(Message {
            level: record.level(),
            pushed: std::time::Instant::now(),
            message: record.args().to_string(),
            source: record.target().to_string(),
        });

        if let Some(file) = &self.file {
            if record.level() <= *self.file_level.read().unwrap() {
                let mut file = file.lock().unwrap();
                let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");

                if let Err(err) = writeln!(
                    file,
                    "[{}][{}][{}] {}",
                    record.level(),
                    timestamp,
                    record.target(),
                    record.args()
                ) {
                    log::error!("Failed to write to log file: {}", err);
                }
            }
        }
    }

    fn flush(&self) {
        if let Some(file) = &self.file {
            file.lock().unwrap().flush().unwrap();
        }
    }
}

pub fn logging_theme_resolver() -> crate::settings::theme::ThemeResolver {
    let mut resolver = crate::settings::theme::ThemeResolver::new();

    resolver
        .link("log.message", "text")
        .link("log.source", "text")
        .link("log.extra", "border");

    resolver
}
