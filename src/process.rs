use crate::error::{Error, Result};
use std::process::{ExitStatus, Stdio};
use tokio::process::{Child, Command};
use tracing::debug;

#[cfg(windows)]
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
#[cfg(windows)]
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject,
};

pub struct Process {
    child: Child,
    #[cfg(windows)]
    _job: JobHandle,
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

        let mut command = Command::new(program);
        command
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        #[cfg(target_os = "linux")]
        configure_parent_death(&mut command);

        let child = command.spawn().map_err(Error::ProcessSpawn)?;

        #[cfg(windows)]
        let job = assign_to_kill_on_close_job(&child)?;

        Ok(Self {
            child,
            #[cfg(windows)]
            _job: job,
        })
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

#[cfg(target_os = "linux")]
fn configure_parent_death(command: &mut Command) {
    unsafe {
        command.pre_exec(|| {
            if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            if libc::getppid() == 1 {
                return Err(std::io::Error::other("parent exited before child setup"));
            }
            Ok(())
        });
    }
}

#[cfg(windows)]
struct JobHandle(HANDLE);

#[cfg(windows)]
unsafe impl Send for JobHandle {}

#[cfg(windows)]
unsafe impl Sync for JobHandle {}

#[cfg(windows)]
impl Drop for JobHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

#[cfg(windows)]
fn assign_to_kill_on_close_job(child: &Child) -> Result<JobHandle> {
    let raw_child = child
        .raw_handle()
        .ok_or_else(|| Error::Io(std::io::Error::other("child process has no handle")))?;

    unsafe {
        let handle = CreateJobObjectW(std::ptr::null(), std::ptr::null());
        if handle.is_null() {
            return Err(Error::Io(std::io::Error::last_os_error()));
        }
        let job = JobHandle(handle);

        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

        if SetInformationJobObject(
            job.0,
            JobObjectExtendedLimitInformation,
            &info as *const _ as *const core::ffi::c_void,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        ) == 0
        {
            return Err(Error::Io(std::io::Error::last_os_error()));
        }

        if AssignProcessToJobObject(job.0, raw_child as HANDLE) == 0 {
            return Err(Error::Io(std::io::Error::last_os_error()));
        }

        Ok(job)
    }
}
