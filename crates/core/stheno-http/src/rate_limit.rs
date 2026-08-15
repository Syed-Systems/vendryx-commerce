//! `stheno-http::rate_limit`: IP and actor rate limiting with trusted proxy resolution.

use axum::http::HeaderMap;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use stheno_types::{Error, Result};

pub fn resolve_client_ip(
    headers: &HeaderMap,
    peer_addr: Option<&str>,
    trusted_proxies: &[&str],
) -> String {
    if let Some(peer) = peer_addr {
        if trusted_proxies.contains(&peer) {
            if let Some(xff) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
                if let Some(client) = xff.split(',').next() {
                    return client.trim().to_string();
                }
            }
        }
        return peer.to_string();
    }

    "127.0.0.1".to_string()
}

#[derive(Clone)]
pub struct RateLimiter {
    max_requests: usize,
    window_secs: u64,
    buckets: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            max_requests,
            window_secs,
            buckets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn check(&self, key: &str) -> Result<()> {
        let mut buckets = self.buckets.lock().unwrap();
        let now = Instant::now();
        let window = std::time::Duration::from_secs(self.window_secs);

        let timestamps = buckets.entry(key.to_string()).or_default();
        timestamps.retain(|&t| now.duration_since(t) < window);

        if timestamps.len() >= self.max_requests {
            return Err(Error::Forbidden {
                permission: "rate_limit_exceeded",
            });
        }

        timestamps.push(now);
        Ok(())
    }
}
