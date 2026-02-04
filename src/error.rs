use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum Error {
    #[error("invalid duration format: {0}")]
    InvalidDuration(String),

    #[error("invalid exit code: {0}")]
    InvalidExitCode(String),

    #[error("wait phase timed out")]
    WaitTimeout,

    #[error("health check failed: {0}")]
    HealthCheckFailed(String),

    #[error("process failed to start: {0}")]
    ProcessSpawn(#[source] io::Error),

    #[error("process terminated by signal")]
    ProcessSignaled,

    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
