use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "watchctl")]
#[command(about = "Process supervisor with wait, watch, and retry phases")]
#[command(version)]
pub struct Args {
    // WAIT PHASE
    #[arg(long = "wait-tcp", value_name = "HOST:PORT", action = clap::ArgAction::Append)]
    pub wait_tcp: Vec<String>,

    #[arg(
        long = "wait-tcp-timeout",
        value_name = "DURATION",
        default_value = "5s"
    )]
    pub wait_tcp_timeout: String,

    #[arg(long = "wait-http", value_name = "URL", action = clap::ArgAction::Append)]
    pub wait_http: Vec<String>,

    #[arg(
        long = "wait-http-timeout",
        value_name = "DURATION",
        default_value = "5s"
    )]
    pub wait_http_timeout: String,

    #[arg(long = "wait-file", value_name = "PATH", action = clap::ArgAction::Append)]
    pub wait_file: Vec<String>,

    #[arg(long = "wait-delay", value_name = "DURATION", action = clap::ArgAction::Append)]
    pub wait_delay: Vec<String>,

    #[arg(long = "wait-timeout", value_name = "DURATION", default_value = "30s")]
    pub wait_timeout: String,

    // WATCH PHASE
    #[arg(long = "watch-http", value_name = "URL", action = clap::ArgAction::Append)]
    pub watch_http: Vec<String>,

    #[arg(
        long = "watch-http-interval",
        value_name = "DURATION",
        default_value = "10s"
    )]
    pub watch_http_interval: String,

    #[arg(
        long = "watch-http-timeout",
        value_name = "DURATION",
        default_value = "5s"
    )]
    pub watch_http_timeout: String,

    #[arg(long = "watch-tcp", value_name = "HOST:PORT", action = clap::ArgAction::Append)]
    pub watch_tcp: Vec<String>,

    #[arg(
        long = "watch-tcp-interval",
        value_name = "DURATION",
        default_value = "10s"
    )]
    pub watch_tcp_interval: String,

    #[arg(
        long = "watch-tcp-timeout",
        value_name = "DURATION",
        default_value = "5s"
    )]
    pub watch_tcp_timeout: String,

    #[arg(long = "watch-file", value_name = "PATH", action = clap::ArgAction::Append)]
    pub watch_file: Vec<String>,

    #[arg(
        long = "watch-file-interval",
        value_name = "DURATION",
        default_value = "10s"
    )]
    pub watch_file_interval: String,

    #[arg(long = "watch-timeout", value_name = "DURATION")]
    pub watch_timeout: Option<String>,

    // RETRY PHASE
    #[arg(long = "retry-times", value_name = "N", default_value = "0")]
    pub retry_times: u32,

    #[arg(long = "retry-delay", value_name = "DURATION", default_value = "1s")]
    pub retry_delay: String,

    #[arg(long = "retry-backoff")]
    pub retry_backoff: bool,

    #[arg(long = "retry-if", value_name = "CODES", action = clap::ArgAction::Append)]
    pub retry_if: Vec<String>,

    #[arg(long = "retry-with-wait")]
    pub retry_with_wait: bool,

    // LOGGING
    #[arg(
        long = "log",
        value_name = "FILE",
        help = "Log watchctl messages to file"
    )]
    pub log: Option<String>,

    // COMMAND
    #[arg(last = true, required = true)]
    pub command: Vec<String>,
}

pub fn parse() -> Args {
    Args::parse()
}
