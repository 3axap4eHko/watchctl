mod check;
mod cli;
mod config;
mod duration;
mod error;
mod process;
mod retry;
mod signal;
mod wait;
mod watch;

use config::Config;
use error::Result;
use process::Process;
use retry::RetryState;
use std::fs::File;
use std::process::ExitCode;
use tokio::select;
use tracing::{error, info, warn};
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
    let mut term = signal::TerminationListener::new();

    loop {
        if run_wait {
            select! {
                biased;
                signal = term.recv() => {
                    warn!("received {} during wait phase, exiting", signal.name);
                    return Ok(ExitCode::from(signal.exit_code));
                }
                result = wait::run_wait_phase(&config.wait) => {
                    if let Err(e) = result {
                        error!("wait phase failed: {e}");
                        return Err(e);
                    }
                }
            }
        }

        info!("starting command: {:?}", config.command);
        let process = Process::spawn(&config.command)?;

        let status = match watch::run_watch_phase(&config.watch, process, &mut term).await? {
            WatchResult::ProcessExited(status) => status,
            WatchResult::HealthCheckFailed(_) | WatchResult::Timeout => {
                return Ok(ExitCode::FAILURE);
            }
            WatchResult::Terminated(term) => {
                return Ok(ExitCode::from(term.exit_code));
            }
        };

        if !retry_state.should_retry(&config.retry, Some(status)) {
            return Ok(exit_code_from_status(status));
        }

        select! {
            biased;
            signal = term.recv() => {
                warn!("received {} before retry, exiting", signal.name);
                return Ok(ExitCode::from(signal.exit_code));
            }
            _ = retry_state.wait_before_retry(&config.retry) => {}
        }
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
