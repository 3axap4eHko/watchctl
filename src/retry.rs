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

        let Some(code) = exit_status.and_then(|s| s.code()) else {
            return true;
        };

        if config.retry_if.is_empty() && config.retry_except.is_empty() {
            return code != 0;
        }

        let allowed = config.retry_if.contains(&code);
        let not_excepted =
            !config.retry_except.is_empty() && code != 0 && !config.retry_except.contains(&code);

        allowed || not_excepted
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;

    fn retry_config(retry_if: HashSet<i32>, retry_except: HashSet<i32>) -> RetryConfig {
        RetryConfig {
            times: Some(3),
            delay: Duration::from_secs(1),
            backoff: false,
            retry_if,
            retry_except,
            with_wait: false,
        }
    }

    fn status(code: i32) -> ExitStatus {
        #[cfg(unix)]
        {
            ExitStatusExt::from_raw(code << 8)
        }

        #[cfg(windows)]
        {
            std::os::windows::process::ExitStatusExt::from_raw(code as u32)
        }
    }

    #[cfg(unix)]
    fn signaled_status(signal: i32) -> ExitStatus {
        ExitStatusExt::from_raw(signal)
    }

    #[test]
    fn any_non_zero_retries_only_failures() {
        let config = retry_config(HashSet::new(), HashSet::new());
        let state = RetryState::new(&config);

        assert!(!state.should_retry(&config, Some(status(0))));
        assert!(state.should_retry(&config, Some(status(1))));
    }

    #[test]
    fn only_retries_selected_exit_codes() {
        let config = retry_config(HashSet::from([1, 3]), HashSet::new());
        let state = RetryState::new(&config);

        assert!(state.should_retry(&config, Some(status(1))));
        assert!(!state.should_retry(&config, Some(status(2))));
        assert!(!state.should_retry(&config, Some(status(0))));
    }

    #[test]
    fn except_skips_excluded_exit_codes_and_success() {
        let config = retry_config(HashSet::new(), HashSet::from([2, 78]));
        let state = RetryState::new(&config);

        assert!(state.should_retry(&config, Some(status(1))));
        assert!(!state.should_retry(&config, Some(status(2))));
        assert!(!state.should_retry(&config, Some(status(78))));
        assert!(!state.should_retry(&config, Some(status(0))));
    }

    #[test]
    fn combined_if_and_except_form_a_union() {
        let config = retry_config(HashSet::from([0]), HashSet::from([42]));
        let state = RetryState::new(&config);

        assert!(state.should_retry(&config, Some(status(0))));
        assert!(state.should_retry(&config, Some(status(1))));
        assert!(!state.should_retry(&config, Some(status(42))));
    }

    #[cfg(unix)]
    #[test]
    fn signaled_processes_always_retry() {
        let any_non_zero = retry_config(HashSet::new(), HashSet::new());
        let only = retry_config(HashSet::from([1]), HashSet::new());
        let except = retry_config(HashSet::new(), HashSet::from([1]));
        let combined = retry_config(HashSet::from([0]), HashSet::from([42]));
        let signaled = Some(signaled_status(9));

        assert!(RetryState::new(&any_non_zero).should_retry(&any_non_zero, signaled));
        assert!(RetryState::new(&only).should_retry(&only, signaled));
        assert!(RetryState::new(&except).should_retry(&except, signaled));
        assert!(RetryState::new(&combined).should_retry(&combined, signaled));
    }

    #[test]
    fn exhausted_retries_override_condition() {
        let config = retry_config(HashSet::new(), HashSet::new());
        let state = RetryState {
            attempts_remaining: Some(0),
            current_delay: Duration::from_secs(1),
        };

        assert!(!state.should_retry(&config, Some(status(1))));
    }
}
