use crate::check::{Check, FileCheck, HttpCheck, TcpCheck, build_http_client};
use crate::config::WaitConfig;
use crate::error::{Error, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{Instant, sleep, timeout};
use tracing::{debug, info};

pub async fn run_wait_phase(config: &WaitConfig) -> Result<()> {
    let start = Instant::now();
    let deadline = start + config.timeout;

    let mut checks: Vec<Box<dyn Check>> = Vec::new();

    for addr in &config.tcp {
        checks.push(Box::new(TcpCheck::new(addr.clone(), config.tcp_timeout)));
    }

    let http_client = if !config.http.is_empty() {
        Some(Arc::new(build_http_client(config.http_timeout)?))
    } else {
        None
    };

    for url in &config.http {
        checks.push(Box::new(HttpCheck::new(
            url.clone(),
            Arc::clone(http_client.as_ref().unwrap()),
        )));
    }

    for path in &config.files {
        checks.push(Box::new(FileCheck::new(path)));
    }

    if checks.is_empty() && config.delays.is_empty() {
        debug!("no wait conditions specified, skipping wait phase");
        return Ok(());
    }

    info!("starting wait phase with {} checks", checks.len());

    for delay in &config.delays {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(Error::WaitTimeout);
        }
        let wait_time = (*delay).min(remaining);
        info!("waiting for {:?}", wait_time);
        sleep(wait_time).await;
    }

    let check_futures: Vec<_> = checks
        .iter()
        .map(|c| wait_for_check(c.as_ref(), deadline))
        .collect();

    let results = futures::future::join_all(check_futures).await;

    for result in results {
        result?;
    }

    info!("wait phase completed in {:?}", start.elapsed());
    Ok(())
}

async fn wait_for_check(check: &dyn Check, deadline: Instant) -> Result<()> {
    let desc = check.description();
    debug!("waiting for {desc}");

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(Error::WaitTimeout);
        }

        match timeout(remaining, check.check()).await {
            Ok(Ok(())) => {
                info!("{desc} ready");
                return Ok(());
            }
            Ok(Err(e)) => {
                debug!("{desc} not ready: {e}");
            }
            Err(_) => {
                return Err(Error::WaitTimeout);
            }
        }

        let retry_delay = Duration::from_millis(500).min(remaining);
        sleep(retry_delay).await;
    }
}
