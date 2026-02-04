use super::{Check, CheckFuture};
use std::path::PathBuf;

pub struct FileCheck {
    path: PathBuf,
    description: String,
}

impl FileCheck {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let description = format!("file:{}", path.display());
        Self { path, description }
    }
}

impl Check for FileCheck {
    fn check(&self) -> CheckFuture<'_> {
        Box::pin(async move {
            if tokio::fs::metadata(&self.path).await.is_ok() {
                Ok(())
            } else {
                Err(format!("file {} does not exist", self.path.display()))
            }
        })
    }

    fn description(&self) -> &str {
        &self.description
    }
}
