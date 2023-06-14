use crate::services::track_request_processor::{
    RequestId, StateStorage, StateStorageError, TrackRequestProcessingContext,
    TrackRequestProcessingState, UserId,
};
use async_trait::async_trait;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::sync::Mutex;

pub(crate) struct MemoryBasedStorage {
    storage: Mutex<HashMap<String, HashMap<String, String>>>,
}

impl MemoryBasedStorage {
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

        if let Entry::Occupied(mut entry) = guard.entry(prefix.to_string()) {
            let map = entry.get_mut();

            map.remove(key);
            if map.is_empty() {
                entry.remove();
            }
        }
    }
}

#[async_trait]
impl StateStorage for MemoryBasedStorage {
    async fn create_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        state: TrackRequestProcessingState,
    ) -> Result<(), StateStorageError> {
        let prefix = format!("{}-state", user_id);
        let key = request_id.to_string();
        let state_str = serde_json::to_string(&state).unwrap();

        self.save(&prefix, &key, &state_str);

        Ok(())
    }

    async fn create_context(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        state: TrackRequestProcessingContext,
    ) -> Result<(), StateStorageError> {
        let prefix = format!("{}-ctx", user_id);
        let key = request_id.to_string();
        let state_str = serde_json::to_string(&state).unwrap();

        self.save(&prefix, &key, &state_str);

        Ok(())
    }

    async fn update_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
        state: &TrackRequestProcessingState,
    ) -> Result<(), StateStorageError> {
        let prefix = format!("{}-state", user_id);
        let key = request_id.to_string();
        let state_str = serde_json::to_string(&state).unwrap();

        self.save(&prefix, &key, &state_str);

        Ok(())
    }

    async fn load_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<TrackRequestProcessingState, StateStorageError> {
        let prefix = format!("{}-state", user_id);
        let key = request_id.to_string();

        let state_str = self
            .get(&prefix, &key)
            .ok_or_else(|| StateStorageError(Box::new(Error::from(ErrorKind::NotFound))))?;
        let state =
            serde_json::from_str(&state_str).map_err(|error| StateStorageError(Box::new(error)))?;

        Ok(state)
    }

    async fn load_context(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<TrackRequestProcessingContext, StateStorageError> {
        let prefix = format!("{}-ctx", user_id);
        let key = request_id.to_string();

        let state_str = self
            .get(&prefix, &key)
            .ok_or_else(|| StateStorageError(Box::new(Error::from(ErrorKind::NotFound))))?;
        let state =
            serde_json::from_str(&state_str).map_err(|error| StateStorageError(Box::new(error)))?;

        Ok(state)
    }

    async fn delete_state(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<(), StateStorageError> {
        let prefix = format!("{}-ctx", user_id);
        let key = request_id.to_string();

        self.delete(&prefix, &key);

        Ok(())
    }

    async fn delete_context(
        &self,
        user_id: &UserId,
        request_id: &RequestId,
    ) -> Result<(), StateStorageError> {
        let prefix = format!("{}-ctx", user_id);
        let key = request_id.to_string();

        self.delete(&prefix, &key);

        Ok(())
    }
}
