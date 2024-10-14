# gatekeeper

A simple HTTP proxy server.

## Getting Started

### Prerequisites

This project requires Rust and Cargo to be installed on your machine. You can download and install them from the [official Rust website](https://www.rust-lang.org/tools/install).

### Installation

1. Clone the repo
    ```shell
    git clone https://github.com/h2cone/gatekeeper
    ```

2. Navigate to the project directory
    ```shell
    cd gatekeeper
    ```

3. Build the project
    ```shell
    cargo build -r
    ```

## Usage

You can run the server via the following command:

```shell
RUST_LOG=INFO ./target/release/gatekeeper -c ./conf.yaml --ba 0.0.0.0:8008 --hcf 30 --ua 127.0.0.1:3000 --ua 127.0.0.1:8090
```

## Command Line Arguments

* `-c <CONF_FILE>` Sets a custom config file.
* `--ba <BIND_ADDRESS>` Sets the bind address for the server.
* `--hcf <HEALTH_CHECK_FREQUENCY>` Sets the health check frequency for the server.
* `--ua <UPSTREAM_ADDRESS>` Sets the upstreams for the server.
* `--cp <CONTEXT_PATH>` Sets the context path for the server.
* `-d` Enables daemon mode.
* For more information try `--help`.