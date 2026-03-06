use crate::check::{Check, FileCheck, HttpCheck, TcpCheck, build_http_client};
use crate::config::WatchConfig;
use crate::error::Result;
use crate::process::Process;
use std::process::ExitStatus;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::task::JoinSet;
use tokio::time::{Instant, interval, sleep};
use tracing::{debug, info, warn};

pub enum WatchResult {
    ProcessExited(ExitStatus),
    #[allow(dead_code)]
    HealthCheckFailed(String),
    Timeout,
}

pub async fn run_watch_phase(config: &WatchConfig, mut process: Process) -> Result<WatchResult> {
    let start = Instant::now();

    let has_health_checks =
        !config.http.is_empty() || !config.tcp.is_empty() || !config.files.is_empty();

    if !has_health_checks && config.timeout.is_none() {
        debug!("no watch conditions, waiting for process to exit");
        let status = process.wait().await?;
        return Ok(WatchResult::ProcessExited(status));
    }

    let watch_future = async {
        if has_health_checks {
            run_health_checks(config).await
        } else {
            std::future::pending().await
        }
    };

    let timeout_future = async {
        if let Some(t) = config.timeout {
            sleep(t).await;
            Some(())
        } else {
            std::future::pending().await
        }
    };

    select! {
        status = process.wait() => {
            let status = status?;
            info!("process exited with {:?} after {:?}", status.code(), start.elapsed());
            Ok(WatchResult::ProcessExited(status))
        }

        result = watch_future => {
            match result {
                Ok(()) => std::future::pending().await,
                Err(msg) => {
                    warn!("health check failed: {msg}");
                    if let Err(e) = process.kill_and_wait().await {
                        warn!("failed to kill process: {e}");
                    }
                    Ok(WatchResult::HealthCheckFailed(msg))
                }
            }
        }

        _ = timeout_future => {
            warn!("watch timeout reached after {:?}", start.elapsed());
            if let Err(e) = process.kill_and_wait().await {
                warn!("failed to kill process: {e}");
            }
            Ok(WatchResult::Timeout)
        }
    }
}

async fn run_health_checks(config: &WatchConfig) -> std::result::Result<(), String> {
    let mut join_set = JoinSet::new();

    let http_client = if !config.http.is_empty() {
        Some(Arc::new(
            build_http_client(config.http_timeout).map_err(|e| e.to_string())?,
        ))
    } else {
        None
    };

    for url in &config.http {
        let check = HttpCheck::new(url.clone(), Arc::clone(http_client.as_ref().unwrap()));
        let initial_delay = config.delay;
        let interval_duration = config.http_interval;
        join_set.spawn(async move {
            run_periodic_check(Box::new(check), initial_delay, interval_duration).await
        });
    }

    for addr in &config.tcp {
        let check = TcpCheck::new(addr.clone(), config.tcp_timeout);
        let initial_delay = config.delay;
        let interval_duration = config.tcp_interval;
        join_set.spawn(async move {
            run_periodic_check(Box::new(check), initial_delay, interval_duration).await
        });
    }

    for path in &config.files {
        let check = FileCheck::new(path);
        let initial_delay = config.delay;
        let interval_duration = config.file_interval;
        join_set.spawn(async move {
            run_periodic_check(Box::new(check), initial_delay, interval_duration).await
        });
    }

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(())) => continue,
            Ok(Err(msg)) => return Err(msg),
            Err(join_err) if join_err.is_panic() => {
                return Err("health check task panicked".to_string());
            }
            Err(_) => continue,
        }
    }

    Ok(())
}

async fn run_periodic_check(
    check: Box<dyn Check>,
    initial_delay: Duration,
    interval_duration: Duration,
) -> std::result::Result<(), String> {
    if !initial_delay.is_zero() {
        sleep(initial_delay).await;
    }

    let mut ticker = interval(interval_duration);
    ticker.tick().await;

    loop {
        let desc = check.description();
        match check.check().await {
            Ok(()) => debug!("{desc} healthy"),
            Err(msg) => return Err(msg),
        }
        ticker.tick().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingCheck {
        calls: Arc<AtomicUsize>,
    }

    impl Check for CountingCheck {
        fn check(&self) -> crate::check::CheckFuture<'_> {
            let calls = Arc::clone(&self.calls);
            Box::pin(async move {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
        }

        fn description(&self) -> &str {
            "counting"
        }
    }

    #[tokio::test]
    async fn watch_delay_defers_first_probe() {
        let calls = Arc::new(AtomicUsize::new(0));
        let task = tokio::spawn(run_periodic_check(
            Box::new(CountingCheck {
                calls: Arc::clone(&calls),
            }),
            Duration::from_millis(200),
            Duration::from_secs(60),
        ));

        sleep(Duration::from_millis(50)).await;
        assert_eq!(calls.load(Ordering::SeqCst), 0);

        sleep(Duration::from_millis(200)).await;
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        task.abort();
    }

    #[tokio::test]
    async fn zero_watch_delay_keeps_immediate_first_probe() {
        let calls = Arc::new(AtomicUsize::new(0));
        let task = tokio::spawn(run_periodic_check(
            Box::new(CountingCheck {
                calls: Arc::clone(&calls),
            }),
            Duration::ZERO,
            Duration::from_secs(60),
        ));

        tokio::time::timeout(Duration::from_millis(100), async {
            loop {
                if calls.load(Ordering::SeqCst) > 0 {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("first probe should run without an explicit watch delay");

        assert_eq!(calls.load(Ordering::SeqCst), 1);

        task.abort();
    }
}
