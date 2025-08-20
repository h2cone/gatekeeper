Repository Guidelines

## Project Structure & Module Organization

- `src/main.rs` - Entry point and CLI argument handling
- `src/cmd/` - Command-line argument parsing and configuration
  - `parser.rs` - CLI argument definitions
  - `mod.rs` - Command module exports
- `src/server/` - Core proxy server implementation
  - `proxy.rs` - HTTP proxy logic
  - `mod.rs` - Server module exports
- `tests/int_test.rs` - Integration tests
- `conf.yaml` - Default configuration file

## Build, Test, and Development Commands

```bash
cargo build          # Build the project
cargo test          # Run all tests
cargo run -- --help # Run with help flag
cargo run -- --bind 0.0.0.0:8008 --upstream 1.1.1.1:443 # Run with custom config
```

## Coding Style & Naming Conventions

- **Language**: Rust 2021 edition
- **Formatting**: Use `cargo fmt` (standard Rust style)
- **Linting**: Use `cargo clippy` for lint suggestions
- **Naming**: snake_case for variables/functions, CamelCase for types
- **Indentation**: 4 spaces (default Rust)

## Testing Guidelines

- **Framework**: Built-in Rust testing framework
- **Location**: `tests/` directory for integration tests
- **Naming**: Test functions prefixed with `test_`
- **Run tests**: `cargo test` runs all unit and integration tests
- **Test coverage**: Aim for critical path coverage in proxy functionality

## Commit & Pull Request Guidelines

### Commit Messages
- Use conventional format: `type(scope): description`
- Examples from history:
  - `Update env_logger and openssl dependencies`
  - `Refactor command line arguments for clarity`

### Pull Requests
- Include clear description of changes
- Reference any related issues
- Ensure tests pass: `cargo test`
- Format code: `cargo fmt`
- Run linting: `cargo clippy`

## Security & Configuration Tips

- TLS certificates: Use `--cert` and `--key` flags for HTTPS
- Health checks: Configure `--hc-freq` for upstream monitoring
- Configuration: Use `conf.yaml` for persistent settings
- Upstream security: Validate upstream certificates when using `--tls`
