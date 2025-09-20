# Repository Guidelines

## Project Structure & Module Organization
Source lives under `src/`. `src/main.rs` boots the CLI and wires configuration. Command parsing sits in `src/cmd/` with `parser.rs` describing flags and defaults. Core proxy behavior is in `src/server/`, where `proxy.rs` houses HTTP forwarding and middleware hooks. Integration coverage lands in `tests/int_test.rs` and shared configs default to `conf.yaml`. Add new modules beside peers and expose them through the local `mod.rs`.

## Build, Test, and Development Commands
- `cargo build` — compile the proxy and surface warnings.
- `cargo run -- --help` — inspect CLI usage and available flags.
- `cargo run -- --bind 0.0.0.0:8008 --upstream 1.1.1.1:443` — launch against a sample upstream.
- `cargo test` — execute unit and integration suites before pushes.
- `cargo clippy --all-targets --all-features` — lint for common mistakes.

## Coding Style & Naming Conventions
Target Rust 2021 with four-space indents. Favor snake_case for functions, vars, and files; CamelCase for types and traits. Run `cargo fmt` prior to commits to ensure canonical formatting, and address clippy’s actionable warnings. Keep public API docs crisp with `///` comments where behavior is non-obvious.

## Testing Guidelines
Use the built-in Rust test harness. Place integration cases under `tests/`, prefix functions with `test_`, and mimic real proxy flows (e.g., upstream failures, TLS handshakes). Aim for coverage around request routing, header rewriting, and error paths. Run `cargo test` locally and ensure new fixtures are deterministic.

## Commit & Pull Request Guidelines
Commit messages follow `type(scope): description` (e.g., `feat(server): add upstream retry`). Group related changes, squash noisy fixups, and keep commits buildable. PRs should summarize intent, call out config or protocol impacts, link issues, and note any manual validation. Verify `cargo fmt`, `cargo clippy`, and `cargo test` prior to requesting review.

## Security & Configuration Tips
Use `--cert`/`--key` when terminating TLS, and enable `--tls` to validate upstream certificates. Tune health probes with `--hc-freq` for critical services. Persist operational defaults in `conf.yaml`, committing only sanitized templates.
