use crate::config::RetryConfig;
use std::process::ExitStatus;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

const MAX_BACKOFF_DELAY: Duration = Duration::from_secs(300);

pub struct RetryState {
    pub attempts_remaining: u32,
    pub current_delay: Duration,
}

impl RetryState {
    pub fn new(config: &RetryConfig) -> Self {
        Self {
            attempts_remaining: config.times,
            current_delay: config.delay,
        }
    }

    pub fn should_retry(&self, config: &RetryConfig, exit_status: Option<ExitStatus>) -> bool {
        if self.attempts_remaining == 0 {
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
        info!(
            "retrying in {:?} ({} attempts remaining)",
            self.current_delay, self.attempts_remaining
        );
        sleep(self.current_delay).await;

        self.attempts_remaining -= 1;

        if config.backoff {
            self.current_delay = self.current_delay.saturating_mul(2).min(MAX_BACKOFF_DELAY);
        }
    }
}
