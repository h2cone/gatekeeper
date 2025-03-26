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
USAGE:
    gatekeeper [OPTIONS] --bind <BIND_ADDR> --upstream <UPSTREAMS>

OPTIONS:
        --bind <BIND_ADDR>        Bind address
    -c, --conf <CONF>             The path to the configuration file.
        --cert <CERT_PATH>        Certificate file path [default: ]
    -d, --daemon                  Whether this server should run in the background
    -h, --help                    Print help information
        --hc-freq <HC_FREQ>       Health check frequency in seconds [default: 0]
        --host <HOST>             Request host [default: ]
        --key <KEY_PATH>          Key file path [default: ]
        --sni <SNI>               SNI for upstream [default: ]
    -t, --test                    This flag is useful for upgrading service where the user wants to
                                  make sure the new service can start before shutting down the old
                                  server process.
        --tls                     TLS for upstream
    -u, --upgrade                 This is the base set of command line arguments for a pingora-based
                                  service
        --upstream <UPSTREAMS>    Upstream address
    -V, --version                 Print version information
```