use std::sync::{Arc, Mutex, MutexGuard, RwLock};

#[derive(Clone, Debug)]
pub struct Flag(Arc<RwLock<bool>>);

impl Flag {
    pub fn new() -> Self {
        Flag(Arc::new(RwLock::new(false)))
    }

    pub fn stop(&self) {
        *self.0.write().unwrap() = true;
    }

    pub fn is_stopped(&self) -> bool {
        *self.0.read().unwrap()
    }
}

impl Default for Flag {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct ProgressHandler {
    jobs: Arc<Mutex<Vec<ProgressHandle>>>,
}

impl ProgressHandler {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add(&self) -> ProgressHandle {
        let progress = ProgressHandle::new(self.clone());
        self.jobs.lock().unwrap().push(progress.clone());
        progress
    }

    pub fn progress(&self) -> Progress {
        self.jobs.lock().unwrap().iter().fold(
            Progress {
                done: 0,
                from: 0,
                is_done: true,
            },
            |prog, job| prog.combine(&job.progress.lock().unwrap()),
        )
    }
}

#[derive(Clone)]
pub struct ProgressHandle {
    progress: Arc<Mutex<Progress>>,
    handler: ProgressHandler,
}

impl ProgressHandle {
    pub fn new(handler: ProgressHandler) -> Self {
        Self {
            progress: Arc::new(Mutex::new(Progress::new_empty())),
            handler,
        }
    }

    pub fn split(&self) -> Self {
        self.handler.add()
    }

    pub fn lock(&self) -> MutexGuard<Progress> {
        self.progress.lock().unwrap()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Progress {
    pub done: usize,
    pub from: usize,
    pub is_done: bool,
}

impl Progress {
    pub fn new(done: usize, from: usize) -> Self {
        Self {
            done,
            from,
            is_done: false,
        }
    }

    pub fn new_empty() -> Self {
        Self::new(0, 1)
    }

    pub fn percent(&self) -> f32 {
        self.done as f32 / self.from as f32
    }

    pub fn finish(&mut self) {
        self.done = self.from;
        self.is_done = true;
    }

    pub fn combine(&self, other: &Self) -> Self {
        Self {
            done: self.done + other.done,
            from: self.from + other.from,
            is_done: self.is_done && other.is_done,
        }
    }
}
