# gatekeeper

A simple HTTP proxy server.

## Installation

Download the latest release from the [releases page](https://github.com/h2cone/gatekeeper/releases).

## Usage

You can run the server via the following command:

```shell
./gatekeeper --bind 0.0.0.0:8008 --hc-freq 30 --host one.one.one.one --tls --upstream 1.0.0.1:443 --upstream 1.1.1.1:443
```

## Command Line Arguments

```shell
Usage: gatekeeper [OPTIONS] --bind <BIND_ADDR> --upstream <UPSTREAMS>

Options:
      --bind <BIND_ADDR>             Bind address
      --cert <CERT_PATH>             Certificate file path [default: ""]
      --key <KEY_PATH>               Key file path [default: ""]
      --upstream <UPSTREAMS>         Upstream address
      --tls                          TLS for upstream
      --sni <SNI>                    SNI for upstream [default: ""]
      --hc-freq <HC_FREQ>            Health check frequency in seconds [default: 0]
      --host <HOST>                  Request host [default: ""]
      --idle-timeout <IDLE_TIMEOUT>  Upstream idle timeout in seconds [default: 0]
      --enable-h2                    Enable HTTP/2 for upstream connections
  -u, --upgrade                      This is the base set of command line arguments for a pingora-based service
  -d, --daemon                       Whether this server should run in the background
  -t, --test                         This flag is useful for upgrading service where the user wants to make sure the new service can start before shutting down the old server process.
  -c, --conf <CONF>                  The path to the configuration file.
  -h, --help                         Print help
  -V, --version                      Print version
```