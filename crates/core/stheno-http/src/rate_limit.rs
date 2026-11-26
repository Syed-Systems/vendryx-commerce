//! `stheno-http::rate_limit`: IP and actor rate limiting with trusted proxy resolution.
//!
//! Scope, stated plainly: **this counter is per-process.** There is no shared backend,
//! because there is no Redis client in this codebase — `stheno-caching-redis` is an
//! in-process stand-in with a simulated outage flag, and `fred` is declared but never
//! called. That is honest for the current single-process deployment. The moment a second
//! replica is started, every limit here silently doubles, and a distributed backend
//! becomes required rather than optional.

use axum::http::HeaderMap;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};
use stheno_types::{Error, Result};

/// Ceiling on distinct tracked keys.
///
/// Without it, `hits` grows for the lifetime of the process: every distinct client IP adds
/// an entry that is never removed, because a key is only revisited if that same client
/// calls again. Rotating source addresses is cheap, so this was an unbounded map fed by
/// untrusted input.
///
/// Past the cap, *new* keys are refused while keys already in the table keep working. That
/// degrades toward the attacker rather than the legitimate client, and it keeps the
/// limiter failing closed instead of silently disabling itself.
const MAX_TRACKED_KEYS: usize = 50_000;

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
    state: Arc<Mutex<LimiterState>>,
}

struct LimiterState {
    /// key -> request timestamps still inside the current window
    hits: HashMap<String, Vec<Instant>>,
    /// When the whole table was last pruned, so sweeping is not O(keys) per request.
    last_sweep: Instant,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            max_requests,
            window_secs,
            state: Arc::new(Mutex::new(LimiterState {
                hits: HashMap::new(),
                last_sweep: Instant::now(),
            })),
        }
    }

    /// A poisoned mutex here means some other request panicked while holding it. The map
    /// is still structurally valid, and taking the inner value is strictly better than
    /// panicking this request too — one panic should not cascade into every later request
    /// that touches the limiter.
    fn lock(&self) -> MutexGuard<'_, LimiterState> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Drops every key whose window has fully elapsed.
    fn sweep(state: &mut LimiterState, now: Instant, window: Duration) {
        state.hits.retain(|_, ts| {
            ts.retain(|&t| now.duration_since(t) < window);
            !ts.is_empty()
        });
        state.last_sweep = now;
    }

    /// Records a hit against `key`, or refuses it if the window is full.
    pub fn check(&self, key: &str) -> Result<()> {
        self.check_with_max(key, self.max_requests)
    }

    /// As `check`, but with a per-call ceiling instead of the configured one.
    ///
    /// The cart, payment and promotion services each exposed a limiter taking the limit as
    /// an argument. Keeping that shape lets their public methods delegate here without
    /// their callers or their tests changing.
    pub fn check_with_max(&self, key: &str, max_requests: usize) -> Result<()> {
        let now = Instant::now();
        let window = Duration::from_secs(self.window_secs);

        let mut state = self.lock();

        if now.duration_since(state.last_sweep) >= window {
            Self::sweep(&mut state, now, window);
        }

        let is_new = !state.hits.contains_key(key);
        if is_new && state.hits.len() >= MAX_TRACKED_KEYS {
            return Err(Error::Forbidden {
                permission: "rate_limit_exceeded",
            });
        }

        let timestamps = state.hits.entry(key.to_string()).or_default();
        timestamps.retain(|&t| now.duration_since(t) < window);

        if timestamps.len() >= max_requests {
            return Err(Error::Forbidden {
                permission: "rate_limit_exceeded",
            });
        }

        timestamps.push(now);
        Ok(())
    }

    /// Clears one key, e.g. after a successful login so a legitimate user who fumbled
    /// their password a few times is not left throttled for the rest of the window.
    pub fn reset(&self, key: &str) {
        self.lock().hits.remove(key);
    }

    /// Number of distinct keys currently tracked. Exposed so a test can assert the table
    /// actually shrinks; without that, the sweep above is unobservable.
    pub fn tracked_keys(&self) -> usize {
        self.lock().hits.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limiter(max: usize, window: u64) -> RateLimiter {
        RateLimiter::new(max, window)
    }

    #[test]
    fn allows_up_to_the_limit_then_refuses() {
        let rl = limiter(3, 60);
        assert!(rl.check("a").is_ok());
        assert!(rl.check("a").is_ok());
        assert!(rl.check("a").is_ok());
        assert!(rl.check("a").is_err(), "the fourth call is over the limit");
    }

    #[test]
    fn keys_are_independent() {
        let rl = limiter(1, 60);
        assert!(rl.check("a").is_ok());
        assert!(
            rl.check("b").is_ok(),
            "b must not be throttled by a's usage"
        );
        assert!(rl.check("a").is_err());
    }

    #[test]
    fn reset_clears_one_key() {
        let rl = limiter(1, 60);
        assert!(rl.check("a").is_ok());
        assert!(rl.check("a").is_err());
        rl.reset("a");
        assert!(rl.check("a").is_ok(), "reset must lift the throttle");
    }

    #[test]
    fn expired_keys_are_pruned_rather_than_kept_forever() {
        let rl = limiter(5, 60);
        for i in 0..100 {
            let _ = rl.check(&format!("ip-{i}"));
        }
        assert_eq!(rl.tracked_keys(), 100);

        // Window of 0 means every timestamp is instantly stale, so the next check sweeps.
        let expiring = RateLimiter::new(5, 0);
        for i in 0..100 {
            let _ = expiring.check(&format!("ip-{i}"));
        }
        // The first check sweeps the table, so these 100 keys collapse rather than persist.
        let _ = expiring.check("fresh");
        assert!(
            expiring.tracked_keys() <= 2,
            "expired keys must be dropped, got {}",
            expiring.tracked_keys()
        );
    }

    #[test]
    fn survives_a_poisoned_mutex() {
        let rl = limiter(1, 60);
        let state = rl.state.clone();
        // Poison it the way a panicking request would.
        let _ = std::panic::catch_unwind(|| {
            let _guard = state.lock().unwrap();
            panic!("request panicked while holding the limiter");
        });
        assert!(
            rl.check("a").is_ok(),
            "a poisoned lock must not take down every later request"
        );
    }
}
