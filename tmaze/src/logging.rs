use std::{
    sync::{Arc, Mutex, MutexGuard, OnceLock, RwLock},
    time::Duration,
};

use cmaze::core::Dims;
use crossterm::style::{Attribute, Color, ContentStyle};
use log::{Log, Metadata, Record};
use unicode_width::UnicodeWidthStr;

use crate::{
    renderer::{self, drawable::Drawable},
    ui::style_with_attribute,
};

static LOGGER: OnceLock<AppLogger> = OnceLock::new();

pub fn get_logger() -> &'static AppLogger {
    // default configuration
    const DEFAULT_DECAY: Duration = Duration::from_secs(5);
    const DEFAULT_MAX_VISIBLE: usize = 5;

    let level = log::Level::Warn;
    // let level = log::Level::Info;

    LOGGER.get_or_init(|| AppLogger::new(level, DEFAULT_DECAY, DEFAULT_MAX_VISIBLE))
}

pub fn init() {
    log::set_logger(get_logger()).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
}

#[derive(Clone)]
pub struct Message {
    pub level: log::Level,
    pub pushed: std::time::Instant,
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

    fn clear_old(&mut self, decay: Duration) {
        let now = std::time::Instant::now();
        for level in self.logs.iter_mut() {
            level.retain(|msg| now.duration_since(msg.pushed) < decay);
        }
    }
}

pub struct LogsIter<'a> {
    logs: MutexGuard<'a, Logs>,
    level: usize,
    index: usize,
}

impl<'a> Iterator for LogsIter<'a> {
    type Item = Message;

    fn next(&mut self) -> Option<Self::Item> {
        while self.level < self.logs.logs.len() && self.index >= self.logs.logs[self.level].len() {
            self.level += 1;
            self.index = 0;
        }
        if self.level >= self.logs.logs.len() {
            return None;
        }

        let log = self.logs.logs[self.level][self.index].clone();
        self.index += 1;
        Some(log)
    }
}

pub struct AppLogger {
    pub min_level: Arc<RwLock<log::Level>>,
    pub decay: Duration,
    pub max_visible: usize,
    logs: Arc<Mutex<Logs>>,
}

impl AppLogger {
    fn new(min_level: log::Level, decay: Duration, max_visible: usize) -> Self {
        Self {
            min_level: Arc::new(RwLock::new(min_level)),
            decay,
            max_visible,
            logs: Arc::new(Mutex::new(Logs {
                logs: Default::default(),
            })),
        }
    }

    pub fn min_level(&self) -> log::Level {
        *self.min_level.read().unwrap()
    }

    fn borrow_mut_logs(&self) -> MutexGuard<Logs> {
        self.logs
            .lock()
            // TODO: create new mutex when poisoned,
            // we will lose logs, but at least we can continue
            .expect("thread holding log panicked, cannot use this logger")
    }

    pub fn get_logs(&self) -> impl Iterator<Item = Message> + '_ {
        let mut logs = self.borrow_mut_logs();
        logs.clear_old(self.decay);

        LogsIter {
            logs,
            level: 0,
            index: 0,
        }
    }

    pub fn switch_debug(&self) {
        if self.min_level() == log::Level::Debug {
            *self.min_level.write().unwrap() = log::Level::Warn;
        } else {
            *self.min_level.write().unwrap() = log::Level::Debug;
        }
    }
}

impl Log for AppLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.min_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            self.borrow_mut_logs().push(Message {
                level: record.level(),
                pushed: std::time::Instant::now(),
                message: record.args().to_string(),
                source: record.module_path().unwrap_or("unknown").to_string(),
            });
        }
    }

    fn flush(&self) {
        todo!()
    }
}

impl Drawable for AppLogger {
    fn draw(&self, pos: Dims, frame: &mut renderer::Frame) {
        self.draw_with_style(pos, frame, crossterm::style::ContentStyle::default());
    }

    fn draw_with_style(&self, pos: Dims, frame: &mut renderer::Frame, style: ContentStyle) {
        for (i, log) in self.get_logs().take(self.max_visible).enumerate() {
            let color = match log.level {
                log::Level::Error => Color::Red,
                log::Level::Warn => Color::Yellow,
                log::Level::Info => Color::White,
                log::Level::Debug => Color::Blue,
                log::Level::Trace => Color::Grey,
            };

            let indicator_style = ContentStyle {
                foreground_color: Some(color),
                ..style
            };

            let source_style = style_with_attribute(style, Attribute::Dim);

            let y = pos.1 + i as i32;
            let len = log.source.width() + 4 + log.message.width();

            let src_x = frame.size.0 - len as i32 - 2;
            let msg_x = src_x + log.source.width() as i32 + 4;
            let src_pos = Dims(src_x, y);
            let msg_pos = Dims(msg_x, y);

            // TODO: make this a setting
            const INDICATOR_CHAR: char = '|';
            // const INDICATOR_CHAR: char = '*';
            // const INDICATOR_CHAR: char = '█';
            // const INDICATOR_CHAR: char = '•';

            log.source.draw_with_style(src_pos, frame, source_style);
            "->".draw_with_style(Dims(msg_x - 3, y), frame, style);
            log.message.draw_with_style(msg_pos, frame, style);
            INDICATOR_CHAR.draw_with_style(Dims(frame.size.0 - 1, y), frame, indicator_style);
        }
    }
}
