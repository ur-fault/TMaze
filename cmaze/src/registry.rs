use std::hash::Hash;

use hashbrown::{Equivalent, HashMap};

pub struct Registry<T, K = String> {
    items: HashMap<K, T>,
    default: Option<T>,
}

impl<T, K> Registry<T, K> {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            default: None,
        }
    }

    pub fn with_default(default: T) -> Self {
        Self {
            items: HashMap::new(),
            default: Some(default),
        }
    }

    pub fn get_default(&self) -> Option<&T> {
        self.default.as_ref()
    }
}

impl<T, K> Registry<T, K>
where
    K: Hash + Eq,
{
    pub fn register(&mut self, key: K, item: T) {
        self.items.insert(key, item);
    }

    pub fn get<Q>(&self, k: &Q) -> Option<&T>
    where
        Q: Hash + Equivalent<K>,
    {
        self.items.get(k)
    }
}
