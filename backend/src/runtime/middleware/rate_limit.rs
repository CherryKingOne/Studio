use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct RateLimiter {
    max: usize,
    window: Duration,
    hits: Mutex<HashMap<String, VecDeque<Instant>>>,
}

impl RateLimiter {
    pub fn new(max: usize, window: Duration) -> Self {
        Self {
            max,
            window,
            hits: Mutex::new(HashMap::new()),
        }
    }

    pub fn check(&self, key: &str) -> bool {
        let now = Instant::now();
        let mut hits = self.hits.lock().unwrap_or_else(|error| error.into_inner());

        if hits.len() > 10_000 {
            hits.retain(|_, entries| !entries.is_empty());
        }

        let entries = hits.entry(key.to_string()).or_default();
        while let Some(front) = entries.front() {
            if now.duration_since(*front) > self.window {
                entries.pop_front();
            } else {
                break;
            }
        }

        if entries.len() >= self.max {
            false
        } else {
            entries.push_back(now);
            true
        }
    }
}
