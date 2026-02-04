mod check;
mod cli;
mod config;
mod duration;
mod error;
mod process;
mod retry;
mod wait;
mod watch;

use config::Config;
use error::Result;
use process::Process;
use retry::RetryState;
use std::fs::File;
use std::process::ExitCode;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use watch::WatchResult;

#[tokio::main]
async fn main() -> ExitCode {
    let args = cli::parse();
    let log_file = args.log.clone();

    init_logging(log_file.as_deref());

    match run(args).await {
        Ok(code) => code,
        Err(e) => {
            error!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn init_logging(log_file: Option<&str>) {
    let Some(path) = log_file else {
        return;
    };

    let file = File::create(path).expect("failed to create log file");
    let writer = BoxMakeWriter::new(file);

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .with_ansi(false)
        .with_writer(writer)
        .init();
}

async fn run(args: cli::Args) -> Result<ExitCode> {
    let config = Config::from_args(args)?;

    let mut retry_state = RetryState::new(&config.retry);
    let mut run_wait = true;

    loop {
        if run_wait && let Err(e) = wait::run_wait_phase(&config.wait).await {
            error!("wait phase failed: {e}");
            return Err(e);
        }

        info!("starting command: {:?}", config.command);
        let process = Process::spawn(&config.command)?;

        let result = watch::run_watch_phase(&config.watch, process).await?;

        let exit_status = match result {
            WatchResult::ProcessExited(status) => Some(status),
            WatchResult::HealthCheckFailed(_) | WatchResult::Timeout => {
                return Ok(ExitCode::FAILURE);
            }
        };

        if !retry_state.should_retry(&config.retry, exit_status) {
            return Ok(exit_code_from_status(exit_status.unwrap()));
        }

        retry_state.wait_before_retry(&config.retry).await;
        run_wait = config.retry.with_wait;
    }
}

fn exit_code_from_status(status: std::process::ExitStatus) -> ExitCode {
    match status.code() {
        Some(0) => ExitCode::SUCCESS,
        Some(code) => ExitCode::from(code.clamp(1, 255) as u8),
        None => ExitCode::FAILURE,
    }
}
