# watchctl

[![CI](https://github.com/3axap4eHko/watchctl/actions/workflows/ci.yml/badge.svg)](https://github.com/3axap4eHko/watchctl/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/watchctl.svg)](https://crates.io/crates/watchctl)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)

Process supervisor with wait, watch, and retry phases.

## Overview

watchctl runs a command through three optional phases:

1. **Wait Phase** - Block until dependencies are ready (TCP ports, HTTP endpoints, files exist)
2. **Watch Phase** - Run the command while monitoring health; terminate if checks fail
3. **Retry Phase** - Automatically restart failed commands with configurable backoff

If the wait phase times out, watchctl exits without starting the command.
If a health check fails during watch phase, the process is terminated without retry.

## Installation

Requires Rust 1.85+ (edition 2024).

### From source

```bash
cargo install --path .
```

### From releases

Download the appropriate binary from [releases](https://github.com/3axap4eHko/watchctl/releases).

## Quick Start

Wait for a database, then run your app with automatic restart:

```bash
watchctl \
  --wait-tcp db:5432 \
  --retry-times 3 \
  -- ./my-app
```

## Usage

```bash
watchctl [OPTIONS] -- <COMMAND> [ARGS...]
```

### Wait Phase

Wait for conditions before starting the command. HTTPS URLs are supported.

```bash
# Wait for TCP port
watchctl --wait-tcp localhost:5432 -- ./my-app

# Wait for HTTP endpoint
watchctl --wait-http http://localhost:8080/health -- ./my-app

# Wait for file
watchctl --wait-file /var/run/ready -- ./my-app

# Wait with delay
watchctl --wait-delay 5s -- ./my-app

# Combine conditions (all must pass)
watchctl --wait-tcp localhost:5432 --wait-http http://localhost:8080/health -- ./my-app
```

### Watch Phase

Monitor health while running. HTTPS URLs are supported.

```bash
# Watch HTTP endpoint
watchctl --watch-http http://localhost:8080/health --watch-http-interval 30s -- ./my-app

# Watch TCP port
watchctl --watch-tcp localhost:8080 -- ./my-app

# Watch file existence
watchctl --watch-file /var/run/healthy -- ./my-app

# Set maximum runtime
watchctl --watch-timeout 1h -- ./my-app
```

### Retry Phase

Restart on failure:

```bash
# Retry up to 3 times
watchctl --retry-times 3 -- ./my-app

# Retry with delay
watchctl --retry-times 3 --retry-delay 5s -- ./my-app

# Retry with exponential backoff
watchctl --retry-times 5 --retry-delay 1s --retry-backoff -- ./my-app

# Retry only on specific exit codes
watchctl --retry-times 3 --retry-if 1,2,3 -- ./my-app

# Re-run wait phase before each retry
watchctl --retry-times 3 --retry-with-wait --wait-tcp localhost:5432 -- ./my-app
```

### Logging

By default, watchctl produces no output (clean stdio passthrough). Use `--log` to write watchctl messages to a file:

```bash
watchctl --log /var/log/watchctl.log --wait-tcp localhost:5432 -- ./my-app
```

The log level can be controlled via the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug watchctl --log /var/log/watchctl.log -- ./my-app
```

## Options

Options marked with `*` can be specified multiple times.

### Wait Phase

| Option | Description | Default |
|--------|-------------|---------|
| `--wait-tcp <HOST:PORT>` * | Wait for TCP port | - |
| `--wait-tcp-timeout <DURATION>` | TCP connection timeout | 5s |
| `--wait-http <URL>` * | Wait for HTTP 2xx | - |
| `--wait-http-timeout <DURATION>` | HTTP request timeout | 5s |
| `--wait-file <PATH>` * | Wait for file existence | - |
| `--wait-delay <DURATION>` * | Wait delay | - |
| `--wait-timeout <DURATION>` | Total wait phase timeout | 30s |

### Watch Phase

| Option | Description | Default |
|--------|-------------|---------|
| `--watch-http <URL>` * | Health check HTTP endpoint | - |
| `--watch-http-interval <DURATION>` | HTTP check interval | 10s |
| `--watch-http-timeout <DURATION>` | HTTP request timeout | 5s |
| `--watch-tcp <HOST:PORT>` * | Health check TCP port | - |
| `--watch-tcp-interval <DURATION>` | TCP check interval | 10s |
| `--watch-tcp-timeout <DURATION>` | TCP connection timeout | 5s |
| `--watch-file <PATH>` * | Health check file existence | - |
| `--watch-file-interval <DURATION>` | File check interval | 10s |
| `--watch-timeout <DURATION>` | Maximum runtime | - |

### Retry Phase

| Option | Description | Default |
|--------|-------------|---------|
| `--retry-times <N>` | Number of retries | 0 |
| `--retry-delay <DURATION>` | Delay between retries | 1s |
| `--retry-backoff` | Double delay after each retry (max 5m) | false |
| `--retry-if <CODES>` * | Retry only on these exit codes | any non-zero |
| `--retry-with-wait` | Re-run wait phase before retry | false |

### General

| Option | Description |
|--------|-------------|
| `--log <FILE>` | Log watchctl messages to file |
| `--help` | Print help information |
| `--version` | Print version information |

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Command completed successfully |
| 1-255 | Command's exit code (clamped to this range) |
| 1 | Wait timeout, health check failure, or watch timeout |

## Duration Format

Durations support these suffixes:
- `ms` - milliseconds (e.g., `500ms`)
- `s` - seconds (e.g., `30s`)
- `m` - minutes (e.g., `5m`)
- `h` - hours (e.g., `1h`)

Compound durations (e.g., `1h30m`) are not supported.

## Examples

### Wait for PostgreSQL before starting app

```bash
watchctl --wait-tcp localhost:5432 --wait-timeout 60s -- ./my-app
```

### Supervised service with health checks

```bash
watchctl \
  --wait-tcp localhost:5432 \
  --watch-http http://localhost:8080/health \
  --watch-http-interval 30s \
  --retry-times 5 \
  --retry-delay 5s \
  --retry-backoff \
  --log /var/log/watchctl.log \
  -- ./my-service
```

### Docker entrypoint

```dockerfile
ENTRYPOINT ["watchctl", "--wait-tcp", "db:5432", "--"]
CMD ["./app"]
```

## License

License [The MIT License](./LICENSE)
Copyright (c) 2026 Ivan Zakharchanka
