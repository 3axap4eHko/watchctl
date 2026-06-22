#[derive(Debug)]
pub struct Termination {
    pub name: &'static str,
    pub exit_code: u8,
}

#[cfg(unix)]
use tokio::signal::unix::Signal;

#[cfg(unix)]
async fn recv_opt(handler: &mut Option<Signal>) {
    use std::future::pending;
    match handler {
        Some(s) => {
            s.recv().await;
        }
        None => pending::<()>().await,
    }
}

#[cfg(unix)]
pub struct TerminationListener {
    interrupt: Option<Signal>,
    terminate: Option<Signal>,
    hangup: Option<Signal>,
}

#[cfg(unix)]
impl TerminationListener {
    pub fn new() -> Self {
        use tokio::signal::unix::{SignalKind, signal};
        Self {
            interrupt: signal(SignalKind::interrupt()).ok(),
            terminate: signal(SignalKind::terminate()).ok(),
            hangup: signal(SignalKind::hangup()).ok(),
        }
    }

    pub async fn recv(&mut self) -> Termination {
        use tokio::select;
        let interrupt = &mut self.interrupt;
        let terminate = &mut self.terminate;
        let hangup = &mut self.hangup;
        select! {
            _ = recv_opt(interrupt) => Termination { name: "SIGINT", exit_code: 130 },
            _ = recv_opt(terminate) => Termination { name: "SIGTERM", exit_code: 143 },
            _ = recv_opt(hangup) => Termination { name: "SIGHUP", exit_code: 129 },
        }
    }
}

#[cfg(windows)]
use tokio::signal::windows::{CtrlBreak, CtrlC, CtrlClose, CtrlLogoff, CtrlShutdown};

#[cfg(windows)]
pub struct TerminationListener {
    interrupt: Option<CtrlC>,
    brk: Option<CtrlBreak>,
    close: Option<CtrlClose>,
    logoff: Option<CtrlLogoff>,
    shutdown: Option<CtrlShutdown>,
}

#[cfg(windows)]
impl TerminationListener {
    pub fn new() -> Self {
        use tokio::signal::windows::{ctrl_break, ctrl_c, ctrl_close, ctrl_logoff, ctrl_shutdown};
        Self {
            interrupt: ctrl_c().ok(),
            brk: ctrl_break().ok(),
            close: ctrl_close().ok(),
            logoff: ctrl_logoff().ok(),
            shutdown: ctrl_shutdown().ok(),
        }
    }

    pub async fn recv(&mut self) -> Termination {
        use std::future::pending;
        use tokio::select;
        let interrupt = &mut self.interrupt;
        let brk = &mut self.brk;
        let close = &mut self.close;
        let logoff = &mut self.logoff;
        let shutdown = &mut self.shutdown;
        select! {
            _ = async { match interrupt { Some(s) => { s.recv().await; }, None => pending::<()>().await } } => Termination { name: "CTRL_C", exit_code: 130 },
            _ = async { match brk { Some(s) => { s.recv().await; }, None => pending::<()>().await } } => Termination { name: "CTRL_BREAK", exit_code: 130 },
            _ = async { match close { Some(s) => { s.recv().await; }, None => pending::<()>().await } } => Termination { name: "CTRL_CLOSE", exit_code: 130 },
            _ = async { match logoff { Some(s) => { s.recv().await; }, None => pending::<()>().await } } => Termination { name: "CTRL_LOGOFF", exit_code: 130 },
            _ = async { match shutdown { Some(s) => { s.recv().await; }, None => pending::<()>().await } } => Termination { name: "CTRL_SHUTDOWN", exit_code: 130 },
        }
    }
}

impl Default for TerminationListener {
    fn default() -> Self {
        Self::new()
    }
}
