#[derive(Debug)]
pub struct Termination {
    pub name: &'static str,
    pub exit_code: u8,
}

#[cfg(unix)]
pub async fn await_termination() -> Termination {
    use std::future::pending;
    use tokio::select;
    use tokio::signal::unix::{Signal, SignalKind, signal};

    async fn recv(handler: &mut Option<Signal>) {
        match handler {
            Some(s) => {
                s.recv().await;
            }
            None => pending::<()>().await,
        }
    }

    let mut interrupt = signal(SignalKind::interrupt()).ok();
    let mut terminate = signal(SignalKind::terminate()).ok();
    let mut hangup = signal(SignalKind::hangup()).ok();

    select! {
        _ = recv(&mut interrupt) => Termination { name: "SIGINT", exit_code: 130 },
        _ = recv(&mut terminate) => Termination { name: "SIGTERM", exit_code: 143 },
        _ = recv(&mut hangup) => Termination { name: "SIGHUP", exit_code: 129 },
    }
}

#[cfg(windows)]
pub async fn await_termination() -> Termination {
    use std::future::pending;
    use tokio::select;
    use tokio::signal::windows::{ctrl_break, ctrl_c, ctrl_close, ctrl_logoff, ctrl_shutdown};

    let mut interrupt = ctrl_c().ok();
    let mut brk = ctrl_break().ok();
    let mut close = ctrl_close().ok();
    let mut logoff = ctrl_logoff().ok();
    let mut shutdown = ctrl_shutdown().ok();

    select! {
        _ = async { match interrupt.as_mut() { Some(s) => { s.recv().await; }, None => pending::<()>().await } } => Termination { name: "CTRL_C", exit_code: 130 },
        _ = async { match brk.as_mut() { Some(s) => { s.recv().await; }, None => pending::<()>().await } } => Termination { name: "CTRL_BREAK", exit_code: 130 },
        _ = async { match close.as_mut() { Some(s) => { s.recv().await; }, None => pending::<()>().await } } => Termination { name: "CTRL_CLOSE", exit_code: 130 },
        _ = async { match logoff.as_mut() { Some(s) => { s.recv().await; }, None => pending::<()>().await } } => Termination { name: "CTRL_LOGOFF", exit_code: 130 },
        _ = async { match shutdown.as_mut() { Some(s) => { s.recv().await; }, None => pending::<()>().await } } => Termination { name: "CTRL_SHUTDOWN", exit_code: 130 },
    }
}
