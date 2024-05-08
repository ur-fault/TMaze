use std::{
    sync::{Arc, Mutex, MutexGuard, OnceLock},
    time::Duration,
};

use crossterm::style::{Color, ContentStyle};
use log::{Log, Metadata, Record};
use unicode_width::UnicodeWidthStr;

use crate::renderer::{self, drawable::Drawable};

static LOGGER: OnceLock<AppLogger> = OnceLock::new();

pub fn get_logger() -> &'static AppLogger {
    // default configuration
    const DEFAULT_DECAY: Duration = Duration::from_secs(1);
    const DEFAULT_MAX_VISIBLE: usize = 5;

    LOGGER.get_or_init(|| AppLogger::new(log::Level::Debug, DEFAULT_DECAY, DEFAULT_MAX_VISIBLE))
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
    pub min_level: log::Level,
    pub decay: Duration,
    pub max_visible: usize,
    logs: Arc<Mutex<Logs>>,
}

impl AppLogger {
    fn new(min_level: log::Level, decay: Duration, max_visible: usize) -> Self {
        Self {
            min_level,
            decay,
            max_visible,
            logs: Arc::new(Mutex::new(Logs {
                logs: Default::default(),
            })),
        }
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
}

impl Log for AppLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.min_level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            self.borrow_mut_logs().push(Message {
                level: record.level(),
                pushed: std::time::Instant::now(),
                message: record.args().to_string(),
            });
        }
    }

    fn flush(&self) {
        todo!()
    }
}

impl Drawable for AppLogger {
    fn draw(&self, pos: renderer::Pos, frame: &mut renderer::Frame) {
        self.draw_with_style(pos, frame, crossterm::style::ContentStyle::default());
    }

    fn draw_with_style(
        &self,
        pos: renderer::Pos,
        frame: &mut renderer::Frame,
        style: ContentStyle,
    ) {
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

            let y = pos.1 + i as u16;
            let len = log.message.width();
            let x = frame.size.0 as usize - len - 2;
            let pos = (x as u16, y);

            // TODO: make this a setting
            // const INDICATOR_CHAR: char = '|';
            const INDICATOR_CHAR: char = '*';
            // const INDICATOR_CHAR: char = '█';
            // const INDICATOR_CHAR: char = '•';

            log.message.draw_with_style(pos, frame, style);
            INDICATOR_CHAR.draw_with_style((frame.size.0 - 1, y), frame, indicator_style);
        }
    }
}
