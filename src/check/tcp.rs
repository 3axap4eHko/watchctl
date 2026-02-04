use super::{Check, CheckFuture};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

pub struct TcpCheck {
    addr: String,
    description: String,
    timeout: Duration,
}

impl TcpCheck {
    pub fn new(addr: String, timeout: Duration) -> Self {
        let description = format!("tcp:{addr}");
        Self {
            addr,
            description,
            timeout,
        }
    }
}

impl Check for TcpCheck {
    fn check(&self) -> CheckFuture<'_> {
        Box::pin(async move {
            match timeout(self.timeout, TcpStream::connect(&self.addr)).await {
                Ok(Ok(_)) => Ok(()),
                Ok(Err(e)) => Err(format!("tcp connect to {}: {e}", self.addr)),
                Err(_) => Err(format!("tcp connect to {} timed out", self.addr)),
            }
        })
    }

    fn description(&self) -> &str {
        &self.description
    }
}
