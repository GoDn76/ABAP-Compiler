use crate::platform::types::RapOperation;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct RedisSessionStore {
    pub url: Option<String>,
    // In-memory fallback if Redis is not configured or in local development
    fallback_buffers: Arc<RwLock<HashMap<String, Vec<RapOperation>>>>,
}

impl RedisSessionStore {
    pub fn new(url: Option<String>) -> Self {
        Self {
            url,
            fallback_buffers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn stage_operation(&self, session_id: &str, op: RapOperation) -> Result<usize, String> {
        let mut buffers = self.fallback_buffers.write().await;
        let list = buffers.entry(session_id.to_string()).or_default();
        list.push(op);
        Ok(list.len())
    }

    pub async fn get_pending(&self, session_id: &str) -> Vec<RapOperation> {
        let buffers = self.fallback_buffers.read().await;
        buffers.get(session_id).cloned().unwrap_or_default()
    }

    pub async fn take_pending(&self, session_id: &str) -> Vec<RapOperation> {
        let mut buffers = self.fallback_buffers.write().await;
        buffers.remove(session_id).unwrap_or_default()
    }

    pub async fn clear_session(&self, session_id: &str) -> usize {
        let mut buffers = self.fallback_buffers.write().await;
        buffers.remove(session_id).map(|v| v.len()).unwrap_or(0)
    }
}
