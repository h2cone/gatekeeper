// @author h2cone

use clap::Parser;
use pingora::lb::LoadBalancer;
use pingora::prelude::{Opt, RoundRobin};
use std::sync::Arc;

#[derive(Parser)]
#[clap(
    author = "h2cone, tw8ape@gmail.com",
    version,
    about = "A simple HTTP proxy server"
)]
pub struct App {
    /// Bind address
    #[clap(long = "ba")]
    pub bind_addr: String,
    /// Certificate file path
    #[clap(long = "cfp", default_value = "")]
    pub cert_path: String,
    /// Key file path
    #[clap(long = "kfp", default_value = "")]
    pub key_path: String,

    #[clap(flatten)]
    pub gateway: Gateway,

    #[clap(flatten)]
    pub opt: Opt,
}

impl App {
    pub fn from_args() -> Self {
        App::parse()
    }
}

#[derive(Parser)]
pub struct Gateway {
    #[clap(skip = None)]
    pub lb: Option<Arc<LoadBalancer<RoundRobin>>>,

    /// Upstream address
    #[clap(long = "ua")]
    pub upstreams: Vec<String>,
    /// TLS for upstream
    #[clap(long)]
    pub tls: bool,
    /// SNI for upstream
    #[clap(long, default_value = "")]
    pub sni: String,
    /// Health check frequency in seconds
    #[clap(long = "hcf", default_value = "0")]
    pub hc_freq: u64,
    /// Request host
    #[clap(long = "rh", default_value = "")]
    pub host: String,
}
