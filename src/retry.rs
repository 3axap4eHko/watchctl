use crate::config::RetryConfig;
use std::process::ExitStatus;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

const MAX_BACKOFF_DELAY: Duration = Duration::from_secs(300);

pub struct RetryState {
    // None = infinite, Some(0) = exhausted, Some(N) = N retries remaining
    pub attempts_remaining: Option<u32>,
    pub current_delay: Duration,
}

impl RetryState {
    pub fn new(config: &RetryConfig) -> Self {
        let attempts_remaining = match config.times {
            None => Some(0), // not specified: no retries
            Some(0) => None, // 0 means infinite
            Some(n) => Some(n),
        };
        Self {
            attempts_remaining,
            current_delay: config.delay,
        }
    }

    pub fn should_retry(&self, config: &RetryConfig, exit_status: Option<ExitStatus>) -> bool {
        if self.attempts_remaining == Some(0) {
            return false;
        }

        let code = exit_status.and_then(|s| s.code());

        match (&config.exit_codes, code) {
            (None, Some(0)) => false,
            (None, _) => true,
            (Some(codes), Some(c)) => codes.contains(&c),
            (Some(_), None) => true,
        }
    }

    pub async fn wait_before_retry(&mut self, config: &RetryConfig) {
        let remaining_label = self
            .attempts_remaining
            .map_or("infinite".to_string(), |n| n.to_string());
        info!(
            "retrying in {:?} ({} attempts remaining)",
            self.current_delay, remaining_label
        );
        sleep(self.current_delay).await;

        if let Some(ref mut n) = self.attempts_remaining {
            *n -= 1;
        }

        if config.backoff {
            self.current_delay = self.current_delay.saturating_mul(2).min(MAX_BACKOFF_DELAY);
        }
    }
}
