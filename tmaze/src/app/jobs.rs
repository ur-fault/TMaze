use std::sync::{Arc, Mutex};

use super::app::AppData;

pub struct Jobs {
    jobs: Arc<Mutex<Vec<Job>>>,
}

impl Jobs {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn pop(&self) -> Option<Job> {
        self.jobs.lock().unwrap().pop()
    }

    pub fn is_empty(&self) -> bool {
        self.jobs.lock().unwrap().is_empty()
    }

    pub fn queuer(&self) -> Qer {
        Qer {
            jobs: self.jobs.clone(),
        }
    }
}

impl Default for Jobs {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Qer {
    jobs: Arc<Mutex<Vec<Job>>>,
}

impl Qer {
    pub fn queue(&self, job: Job) {
        self.jobs.lock().unwrap().push(job);
    }
}

pub struct Job {
    name: Option<String>,
    task: Box<dyn FnOnce(&mut AppData) + Sync + Send>,
}

impl Job {
    pub fn new(task: impl FnOnce(&mut AppData) + 'static + Send + Sync) -> Self {
        Self {
            name: None,
            task: Box::new(task),
        }
    }

    pub fn named(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn call(self, data: &mut AppData) {
        (self.task)(data);
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}
