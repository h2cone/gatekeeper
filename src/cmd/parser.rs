use clap::Parser;
use pingora::lb::LoadBalancer;
use pingora::prelude::{Opt, RoundRobin};
use std::sync::Arc;

#[derive(Parser)]
#[command(
    author = "tw8ape@gmail.com",
    version,
    about = "A simple HTTP proxy server"
)]
pub struct App {
    /// Bind address
    #[arg(long = "bind")]
    pub bind_addr: String,
    /// Certificate file path
    #[arg(long = "cert", default_value = "")]
    pub cert_path: String,
    /// Key file path
    #[arg(long = "key", default_value = "")]
    pub key_path: String,

    #[command(flatten)]
    pub gateway: Gateway,

    #[command(flatten)]
    pub opt: Opt,
}

impl App {
    pub fn from_args() -> Self {
        App::parse()
    }
}

#[derive(Parser)]
pub struct Gateway {
    #[arg(skip = None)]
    pub lb: Option<Arc<LoadBalancer<RoundRobin>>>,

    /// Upstream address
    #[arg(long = "upstream", required = true)]
    pub upstreams: Vec<String>,
    /// TLS for upstream
    #[arg(long = "tls")]
    pub tls: bool,
    /// SNI for upstream
    #[arg(long = "sni", default_value = "")]
    pub sni: String,
    /// Health check frequency in seconds
    #[arg(long = "hc-freq", default_value = "0")]
    pub hc_freq: u64,
    /// Request host
    #[arg(long = "host", default_value = "")]
    pub host: String,
    /// Upstream idle timeout in seconds
    #[arg(long = "idle-timeout", default_value = "0")]
    pub idle_timeout: u64,
    /// Enable HTTP/2 for upstream connections
    #[arg(long = "enable-h2")]
    pub enable_h2: bool,
}
