use crate::error::{Error, Result};
use std::process::{ExitStatus, Stdio};
use tokio::process::{Child, Command};
use tracing::debug;

pub struct Process {
    child: Child,
}

impl Process {
    pub fn spawn(command: &[String]) -> Result<Self> {
        let (program, args) = command.split_first().ok_or_else(|| {
            Error::ProcessSpawn(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "command cannot be empty",
            ))
        })?;

        debug!("spawning process: {program} {:?}", args);

        let child = Command::new(program)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(Error::ProcessSpawn)?;

        Ok(Self { child })
    }

    pub async fn wait(&mut self) -> Result<ExitStatus> {
        self.child.wait().await.map_err(Error::Io)
    }

    pub async fn kill_and_wait(&mut self) -> Result<()> {
        self.child.kill().await.map_err(Error::Io)?;
        self.child.wait().await.map_err(Error::Io)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn id(&self) -> Option<u32> {
        self.child.id()
    }
}
