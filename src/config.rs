use crate::cli::Args;
use crate::duration::parse_duration;
use crate::error::Result;
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
    pub times: u32,
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
            http_interval: parse_duration(&args.watch_http_interval)?,
            http_timeout: parse_duration(&args.watch_http_timeout)?,
            tcp: args.watch_tcp,
            tcp_interval: parse_duration(&args.watch_tcp_interval)?,
            tcp_timeout: parse_duration(&args.watch_tcp_timeout)?,
            files: args.watch_file,
            file_interval: parse_duration(&args.watch_file_interval)?,
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
