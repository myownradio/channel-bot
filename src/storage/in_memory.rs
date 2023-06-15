use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Mutex;

pub(crate) struct InMemoryStorage {
    storage: Mutex<HashMap<String, HashMap<String, String>>>,
}

impl InMemoryStorage {
    pub(crate) fn new() -> Self {
        Self {
            storage: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn save(&self, prefix: &str, key: &str, value: &str) {
        let mut guard = self.storage.lock().unwrap();

        guard
            .entry(prefix.into())
            .or_default()
            .insert(key.into(), value.into());
    }

    pub(crate) fn get(&self, prefix: &str, key: &str) -> Option<String> {
        let guard = self.storage.lock().unwrap();

        guard.get(prefix).and_then(|m| m.get(key)).cloned()
    }

    pub(crate) fn get_all(&self, prefix: &str) -> Option<HashMap<String, String>> {
        let guard = self.storage.lock().unwrap();

        guard.get(prefix).cloned()
    }

    pub(crate) fn delete(&self, prefix: &str, key: &str) {
        let mut guard = self.storage.lock().unwrap();

        if let Entry::Occupied(mut entry) = guard.entry(prefix.into()) {
            let map = entry.get_mut();
            map.remove(key);

            if map.is_empty() {
                entry.remove();
            }
        }
    }
}
