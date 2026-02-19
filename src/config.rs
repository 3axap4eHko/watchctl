use crate::cli::Args;
use crate::duration::parse_duration;
use crate::error::{Error, Result};
use std::collections::HashSet;
use std::time::Duration;

#[derive(Debug)]
pub struct Config {
    pub wait: WaitConfig,
    pub watch: WatchConfig,
    pub retry: RetryConfig,
    pub command: Vec<String>,
}

#[derive(Debug)]
pub struct WaitConfig {
    pub tcp: Vec<String>,
    pub tcp_timeout: Duration,
    pub http: Vec<String>,
    pub http_timeout: Duration,
    pub files: Vec<String>,
    pub delays: Vec<Duration>,
    pub timeout: Duration,
}

#[derive(Debug)]
pub struct WatchConfig {
    pub http: Vec<String>,
    pub http_interval: Duration,
    pub http_timeout: Duration,
    pub tcp: Vec<String>,
    pub tcp_interval: Duration,
    pub tcp_timeout: Duration,
    pub files: Vec<String>,
    pub file_interval: Duration,
    pub timeout: Option<Duration>,
}

#[derive(Debug)]
pub struct RetryConfig {
    pub times: Option<u32>,
    pub delay: Duration,
    pub backoff: bool,
    pub exit_codes: Option<HashSet<i32>>,
    pub with_wait: bool,
}

impl Config {
    pub fn from_args(args: Args) -> Result<Self> {
        let wait = WaitConfig {
            tcp: args.wait_tcp,
            tcp_timeout: parse_duration(&args.wait_tcp_timeout)?,
            http: args.wait_http,
            http_timeout: parse_duration(&args.wait_http_timeout)?,
            files: args.wait_file,
            delays: args
                .wait_delay
                .iter()
                .map(|s| parse_duration(s))
                .collect::<Result<Vec<_>>>()?,
            timeout: parse_duration(&args.wait_timeout)?,
        };

        let watch = WatchConfig {
            http: args.watch_http,
            http_interval: parse_non_zero_duration(
                &args.watch_http_interval,
                "--watch-http-interval",
            )?,
            http_timeout: parse_duration(&args.watch_http_timeout)?,
            tcp: args.watch_tcp,
            tcp_interval: parse_non_zero_duration(
                &args.watch_tcp_interval,
                "--watch-tcp-interval",
            )?,
            tcp_timeout: parse_duration(&args.watch_tcp_timeout)?,
            files: args.watch_file,
            file_interval: parse_non_zero_duration(
                &args.watch_file_interval,
                "--watch-file-interval",
            )?,
            timeout: args.watch_timeout.map(|s| parse_duration(&s)).transpose()?,
        };

        let exit_codes = if args.retry_if.is_empty() {
            None
        } else {
            let mut codes = HashSet::new();
            for s in &args.retry_if {
                for code_str in s.split(',') {
                    let code: i32 = code_str
                        .trim()
                        .parse()
                        .map_err(|_| crate::error::Error::InvalidExitCode(code_str.to_string()))?;
                    codes.insert(code);
                }
            }
            Some(codes)
        };

        let retry = RetryConfig {
            times: args.retry_times,
            delay: parse_duration(&args.retry_delay)?,
            backoff: args.retry_backoff,
            exit_codes,
            with_wait: args.retry_with_wait,
        };

        Ok(Config {
            wait,
            watch,
            retry,
            command: args.command,
        })
    }
}

fn parse_non_zero_duration(raw: &str, option_name: &str) -> Result<Duration> {
    let duration = parse_duration(raw)?;
    if duration.is_zero() {
        return Err(Error::InvalidDuration(format!(
            "{option_name} must be greater than 0"
        )));
    }
    Ok(duration)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_args() -> Args {
        Args {
            wait_tcp: Vec::new(),
            wait_tcp_timeout: "5s".to_string(),
            wait_http: Vec::new(),
            wait_http_timeout: "5s".to_string(),
            wait_file: Vec::new(),
            wait_delay: Vec::new(),
            wait_timeout: "30s".to_string(),
            watch_http: Vec::new(),
            watch_http_interval: "10s".to_string(),
            watch_http_timeout: "5s".to_string(),
            watch_tcp: Vec::new(),
            watch_tcp_interval: "10s".to_string(),
            watch_tcp_timeout: "5s".to_string(),
            watch_file: Vec::new(),
            watch_file_interval: "10s".to_string(),
            watch_timeout: None,
            retry_times: None,
            retry_delay: "1s".to_string(),
            retry_backoff: false,
            retry_if: Vec::new(),
            retry_with_wait: false,
            log: None,
            command: vec!["true".to_string()],
        }
    }

    #[test]
    fn rejects_zero_watch_http_interval() {
        let mut args = base_args();
        args.watch_http_interval = "0s".to_string();

        let err = Config::from_args(args).expect_err("watch interval of zero should be rejected");
        match err {
            Error::InvalidDuration(msg) => {
                assert!(msg.contains("--watch-http-interval"));
            }
            other => panic!("unexpected error: {other}"),
        }
    }
}
