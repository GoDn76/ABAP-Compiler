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

use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct IpRateLimiter {
    pub redis_url: Option<String>,
    pub max_requests: usize,
    pub window_secs: u64,
    // In-memory sliding window fallback: IP -> list of request timestamps
    memory_store: Arc<parking_lot::Mutex<HashMap<String, Vec<Instant>>>>,
}

impl IpRateLimiter {
    pub fn new(redis_url: Option<String>, max_requests: usize, window_secs: u64) -> Self {
        Self {
            redis_url,
            max_requests,
            window_secs,
            memory_store: Arc::new(parking_lot::Mutex::new(HashMap::new())),
        }
    }

    /// Checks if a request from `ip` is allowed under the rate limit.
    /// Returns Ok(()) if allowed, or Err(retry_after_secs) if rate limited.
    pub async fn check_rate_limit(&self, ip: &str) -> Result<(), u64> {
        if self.max_requests == 0 {
            return Ok(());
        }
        let max_requests = self.max_requests;
        let window = Duration::from_secs(self.window_secs);

        #[cfg(feature = "database")]
        if let Some(ref url) = self.redis_url {
            if let Ok(client) = redis::Client::open(url.as_str()) {
                if let Ok(mut con) = client.get_tokio_connection().await {
                    let key = format!("ratelimit:{}", ip);
                    let count: Result<i64, _> = redis::cmd("INCR").arg(&key).query_async(&mut con).await;
                    if let Ok(c) = count {
                        if c == 1 {
                            let _: Result<(), _> = redis::cmd("EXPIRE").arg(&key).arg(self.window_secs).query_async(&mut con).await;
                        }
                        let ttl: i64 = redis::cmd("TTL").arg(&key).query_async(&mut con).await.unwrap_or(self.window_secs as i64);
                        if c > max_requests as i64 {
                            return Err(ttl.max(1) as u64);
                        }
                        return Ok(());
                    }
                }
            }
        }

        let now = Instant::now();
        let mut store = self.memory_store.lock();
        let timestamps = store.entry(ip.to_string()).or_default();

        // Evict timestamps older than configured window
        timestamps.retain(|&t| now.duration_since(t) < window);

        if timestamps.len() >= max_requests {
            let oldest = timestamps[0];
            let elapsed = now.duration_since(oldest);
            let retry_after = if elapsed < window {
                (window - elapsed).as_secs().max(1)
            } else {
                1
            };
            return Err(retry_after);
        }

        timestamps.push(now);
        Ok(())
    }
}
