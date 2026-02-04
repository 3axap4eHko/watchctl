mod file;
mod http;
mod tcp;

pub use file::FileCheck;
pub use http::{HttpCheck, build_http_client};
pub use tcp::TcpCheck;

use std::future::Future;
use std::pin::Pin;

pub type CheckFuture<'a> = Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>>;

pub trait Check: Send + Sync {
    fn check(&self) -> CheckFuture<'_>;
    fn description(&self) -> &str;
}
