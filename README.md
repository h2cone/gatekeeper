# gatekeeper

A simple HTTP proxy server.

## Getting Started

To get a local copy up and running, follow these simple steps.

### Prerequisites

This project requires Rust and Cargo to be installed on your machine. You can download and install them from the [official Rust website](https://www.rust-lang.org/tools/install).

### Installation

1. Clone the repo
```bash
git clone https://github.com/h2cone/gatekeeper
```

2. Navigate to the project directory
```bash
cd gatekeeper
```

3. Build the project
```bash
cargo build
```

## Usage

You can run the server using the `cargo run` command followed by the necessary flags. Here is an example:

```bash
cargo run -- -c conf.yaml --ba 0.0.0.0:8008 --pa 127.0.0.1:3000
```

## Command Line Arguments

- `-c <FILE>`: Sets a custom config file.
- `-d`: Enables daemon mode.
- `--ba <BIND_ADDRESS>`: Sets the bind address for the server.
- `--cp <CONTEXT_PATH>`: Sets the context path for the server.
- `--pa <PEER_ADDRESS>`: Sets the peer address for the server.
- ...
