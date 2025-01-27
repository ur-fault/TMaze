use std::{hash::Hash, sync::Arc};

use hashbrown::{Equivalent, HashMap};

pub struct Registry<T: ?Sized, K = String> {
    items: HashMap<K, Arc<T>>,
    default: Option<Arc<T>>,
}

impl<T: ?Sized, K> Registry<T, K> {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            default: None,
        }
    }

    pub fn with_default(default: Arc<T>) -> Self {
        Self {
            items: HashMap::new(),
            default: Some(default),
        }
    }

    pub fn get_default(&self) -> Option<Arc<T>> {
        self.default.clone()
    }
}

impl<T: ?Sized, K> Registry<T, K>
where
    K: Hash + Eq,
{
    pub fn register(&mut self, key: K, item: Arc<T>) {
        self.items.insert(key, item);
    }

    pub fn get<Q>(&self, k: &Q) -> Option<Arc<T>>
    where
        Q: Hash + Equivalent<K>,
    {
        self.items.get(k).cloned()
    }
}
