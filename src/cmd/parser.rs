// @author h2cone

use clap::Parser;
use pingora::lb::LoadBalancer;
use pingora::prelude::{Opt, RoundRobin};
use std::sync::Arc;

#[derive(Parser)]
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
    /// Context path
    #[clap(long = "cp", default_value = "/")]
    pub ctx_path: String,
    /// Upstream address
    #[clap(long = "ua")]
    pub upstreams: Vec<String>,
    /// TLS
    #[clap(long)]
    pub tls: bool,
    /// SNI
    #[clap(long, default_value = "")]
    pub sni: String,
    /// Health check frequency in seconds
    #[clap(long = "hcf", default_value = "0")]
    pub hc_freq: u64,
    /// Request host
    #[clap(long = "rh", default_value = "")]
    pub host: String,
}
