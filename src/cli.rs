use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "watchctl")]
#[command(about = "Process supervisor with wait, watch, and retry phases")]
#[command(version)]
pub struct Args {
    // WAIT PHASE
    #[arg(
        long = "wait-tcp",
        value_name = "HOST:PORT",
        action = clap::ArgAction::Append,
        help_heading = "Wait Phase",
        help = "Wait until this TCP endpoint accepts a connection (repeatable)"
    )]
    pub wait_tcp: Vec<String>,

    #[arg(
        long = "wait-tcp-timeout",
        value_name = "DURATION",
        default_value = "5s",
        help_heading = "Wait Phase",
        help = "Per-attempt timeout for each --wait-tcp check"
    )]
    pub wait_tcp_timeout: String,

    #[arg(
        long = "wait-http",
        value_name = "URL",
        action = clap::ArgAction::Append,
        help_heading = "Wait Phase",
        help = "Wait until this URL returns HTTP 2xx (repeatable)"
    )]
    pub wait_http: Vec<String>,

    #[arg(
        long = "wait-http-timeout",
        value_name = "DURATION",
        default_value = "5s",
        help_heading = "Wait Phase",
        help = "Per-request timeout for each --wait-http check"
    )]
    pub wait_http_timeout: String,

    #[arg(
        long = "wait-file",
        value_name = "PATH",
        action = clap::ArgAction::Append,
        help_heading = "Wait Phase",
        help = "Wait until this path exists (repeatable)"
    )]
    pub wait_file: Vec<String>,

    #[arg(
        long = "wait-delay",
        value_name = "DURATION",
        action = clap::ArgAction::Append,
        help_heading = "Wait Phase",
        help = "Sleep before checks begin (repeatable, applied in order)"
    )]
    pub wait_delay: Vec<String>,

    #[arg(
        long = "wait-timeout",
        value_name = "DURATION",
        default_value = "30s",
        help_heading = "Wait Phase",
        help = "Maximum total time for the wait phase"
    )]
    pub wait_timeout: String,

    // WATCH PHASE
    #[arg(
        long = "watch-http",
        value_name = "URL",
        action = clap::ArgAction::Append,
        help_heading = "Watch Phase",
        help = "Monitor this URL while command runs; non-2xx fails the watch phase (repeatable)"
    )]
    pub watch_http: Vec<String>,

    #[arg(
        long = "watch-http-interval",
        value_name = "DURATION",
        default_value = "10s",
        help_heading = "Watch Phase",
        help = "Interval between --watch-http checks (must be > 0)"
    )]
    pub watch_http_interval: String,

    #[arg(
        long = "watch-http-timeout",
        value_name = "DURATION",
        default_value = "5s",
        help_heading = "Watch Phase",
        help = "Per-request timeout for --watch-http checks"
    )]
    pub watch_http_timeout: String,

    #[arg(
        long = "watch-tcp",
        value_name = "HOST:PORT",
        action = clap::ArgAction::Append,
        help_heading = "Watch Phase",
        help = "Monitor this TCP endpoint while command runs (repeatable)"
    )]
    pub watch_tcp: Vec<String>,

    #[arg(
        long = "watch-tcp-interval",
        value_name = "DURATION",
        default_value = "10s",
        help_heading = "Watch Phase",
        help = "Interval between --watch-tcp checks (must be > 0)"
    )]
    pub watch_tcp_interval: String,

    #[arg(
        long = "watch-tcp-timeout",
        value_name = "DURATION",
        default_value = "5s",
        help_heading = "Watch Phase",
        help = "Per-attempt timeout for --watch-tcp checks"
    )]
    pub watch_tcp_timeout: String,

    #[arg(
        long = "watch-file",
        value_name = "PATH",
        action = clap::ArgAction::Append,
        help_heading = "Watch Phase",
        help = "Monitor that this path exists while command runs (repeatable)"
    )]
    pub watch_file: Vec<String>,

    #[arg(
        long = "watch-file-interval",
        value_name = "DURATION",
        default_value = "10s",
        help_heading = "Watch Phase",
        help = "Interval between --watch-file checks (must be > 0)"
    )]
    pub watch_file_interval: String,

    #[arg(
        long = "watch-timeout",
        value_name = "DURATION",
        help_heading = "Watch Phase",
        help = "Hard maximum runtime for each watch phase attempt"
    )]
    pub watch_timeout: Option<String>,

    // RETRY PHASE
    #[arg(
        long = "retry-times",
        value_name = "N",
        help_heading = "Retry Phase",
        help = "Number of retries after failure (0 = infinite, omitted = no retries)"
    )]
    pub retry_times: Option<u32>,

    #[arg(
        long = "retry-delay",
        value_name = "DURATION",
        default_value = "1s",
        help_heading = "Retry Phase",
        help = "Delay before each retry attempt"
    )]
    pub retry_delay: String,

    #[arg(
        long = "retry-backoff",
        help_heading = "Retry Phase",
        help = "Double retry delay after each retry (max 5m)"
    )]
    pub retry_backoff: bool,

    #[arg(
        long = "retry-if",
        value_name = "CODES",
        action = clap::ArgAction::Append,
        help_heading = "Retry Phase",
        help = "Retry only when exit code matches (e.g. 1,2,3); repeatable"
    )]
    pub retry_if: Vec<String>,

    #[arg(
        long = "retry-with-wait",
        help_heading = "Retry Phase",
        help = "Run the wait phase again before each retry"
    )]
    pub retry_with_wait: bool,

    // LOGGING
    #[arg(
        long = "log",
        value_name = "FILE",
        help_heading = "Logging",
        help = "Write watchctl internal logs to FILE"
    )]
    pub log: Option<String>,

    // COMMAND
    #[arg(
        last = true,
        required = true,
        help_heading = "Command",
        help = "Command and arguments to run (pass after --)"
    )]
    pub command: Vec<String>,
}

pub fn parse() -> Args {
    Args::parse()
}
