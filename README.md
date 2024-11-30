# gatekeeper

A simple HTTP proxy server.

## Installation

Download the latest release from the [releases page](https://github.com/h2cone/gatekeeper/releases).

## Usage

You can run the server via the following command:

```shell
RUST_LOG=TRACE ./gatekeeper -c ./conf.yaml --ba 0.0.0.0:8008 --hcf 30 --rh one.one.one.one --tls --ua 1.0.0.1:443 --ua 1.1.1.1:443
```

## Command Line Arguments

```shell
USAGE:
    gatekeeper [OPTIONS] --ba <BIND_ADDR>

OPTIONS:
        --ba <BIND_ADDR>     Bind address
    -c, --conf <CONF>        The path to the configuration file.
        --cfp <CERT_PATH>    Certificate file path [default: ]
    -d, --daemon             Whether this server should run in the background
    -h, --help               Print help information
        --hcf <HC_FREQ>      Health check frequency in seconds [default: 0]
        --kfp <KEY_PATH>     Key file path [default: ]
        --rh <HOST>          Request host [default: ]
        --sni <SNI>          SNI for upstream [default: ]
    -t, --test               This flag is useful for upgrading service where the user wants to make
                             sure the new service can start before shutting down the old server
                             process.
        --tls                TLS for upstream
    -u, --upgrade            This is the base set of command line arguments for a pingora-based
                             service
        --ua <UPSTREAMS>     Upstream address
    -V, --version            Print version information
```