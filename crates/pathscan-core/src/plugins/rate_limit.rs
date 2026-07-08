use crate::plugin::Plugin;
use crate::types::ResponseContext;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub struct RateLimitPlugin {
    delay: Arc<AtomicU32>,
    base_ms: u32,
}

impl RateLimitPlugin {
    pub fn new(base_ms: u32) -> Self {
        Self { delay: Arc::new(AtomicU32::new(0)), base_ms }
    }

    pub fn current_delay(&self) -> Option<Duration> {
        let ms = self.delay.load(Ordering::Relaxed);
        if ms > 0 { Some(Duration::from_millis(ms as u64)) } else { None }
    }
}

impl Plugin for RateLimitPlugin {
    fn name(&self) -> &str { "rate_limit" }
    fn version(&self) -> &str { "0.1.0" }

    fn on_response(&self, resp: ResponseContext) -> ResponseContext {
        if resp.status == 429 {
            let cur = self.delay.load(Ordering::Relaxed);
            let new = if cur == 0 { self.base_ms } else { (cur * 2).min(10_000) };
            self.delay.store(new, Ordering::Relaxed);
        } else if resp.status < 400 {
            let cur = self.delay.load(Ordering::Relaxed);
            if cur > 0 {
                self.delay.store((cur / 2).max(self.base_ms), Ordering::Relaxed);
            }
        }
        resp
    }
}
