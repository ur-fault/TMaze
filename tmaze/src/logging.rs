use std::{
    sync::{Arc, Mutex, MutexGuard, OnceLock},
    time::Duration,
};

use log::{Log, Metadata, Record};

const DEFAULT_DECAY: Duration = Duration::from_secs(5);
static LOGGER: OnceLock<AppLogger> = OnceLock::new();

pub fn get_logger() -> &'static AppLogger {
    LOGGER.get_or_init(|| AppLogger::new(true, DEFAULT_DECAY))
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
        self.logs[message.level as usize].push(message);
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
        if self.index >= self.logs.logs[self.level].len() {
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
    pub show_info: bool,
    decay: Duration,
    logs: Arc<Mutex<Logs>>,
}

impl AppLogger {
    fn new(show_info: bool, decay: Duration) -> Self {
        Self {
            show_info,
            decay,
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
        let level = if self.show_info {
            log::Level::Info
        } else {
            log::Level::Warn
        };
        metadata.level() <= level
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
