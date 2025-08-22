# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

- **Build release**: `cargo build --release`
- **Build debug**: `cargo build`
- **Run tests**: `cargo test`
- **Run specific test**: `cargo test test_proxy_example_com`
- **Lint**: `cargo clippy`
- **Format**: `cargo fmt`

## Architecture Overview

This is a Rust-based HTTP proxy server built on the Pingora framework. The main components:

### Core Structure
- **main.rs**: Entry point, initializes Pingora server with load balancer and proxy service
- **cmd/parser.rs**: CLI argument parsing using Clap, defines `App` and `Gateway` structs
- **server/proxy.rs**: Proxy implementation using Pingora's `ProxyHttp` trait

### Key Features
- TCP health checking with configurable frequency
- Round-robin load balancing across multiple upstreams
- TLS support for upstream connections
- Host header manipulation
- HTTP/2 support for upstream connections
- Configurable idle timeouts

### Integration Tests
- **tests/int_test.rs**: Comprehensive integration tests covering:
  - Basic HTTP proxying
  - Multiple upstream servers
  - TLS upstream connections
  - Health check functionality
  - Dynamic port allocation for parallel test execution

### Dependencies
- **pingora**: Core proxy framework from Cloudflare
- **clap**: Command-line argument parsing
- **async-trait**: For async trait implementations
- **openssl**: TLS/SSL support
- **reqwest**: HTTP client for integration tests
- **tokio**: Async runtime

### Configuration
Command-line arguments control:
- Bind address (`--bind`)
- Upstream servers (`--upstream`)
- TLS settings (`--tls`, `--cert`, `--key`)
- Health check frequency (`--hc-freq`)
- Host header override (`--host`)
- SNI configuration (`--sni`)
- Idle timeout (`--idle-timeout`)
- HTTP/2 support (`--enable-h2`)

The proxy uses Pingora's built-in load balancing and health checking capabilities with TCP-based health checks.