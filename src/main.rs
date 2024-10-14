// @author h2cone

use async_trait::async_trait;
use clap::Parser;
use pingora::lb::{health_check, LoadBalancer};
use pingora::prelude::{background_service, HttpPeer, Opt, RoundRobin};
use pingora::proxy::{http_proxy_service, ProxyHttp, Session};
use pingora::server::Server;
use std::sync::Arc;
use std::time::Duration;

fn main() {
    env_logger::init();

    let app = App::from_args();
    let mut server = Server::new(Some(app.opt)).unwrap();
    server.bootstrap();

    let mut gateway = app.gateway;
    let mut lb = LoadBalancer::<RoundRobin>::try_from_iter(&gateway.upstreams).unwrap();

    if gateway.hc_freq > 0 {
        let hc = health_check::TcpHealthCheck::new();
        lb.set_health_check(hc);
        lb.health_check_frequency = Some(Duration::from_secs(gateway.hc_freq));

        let background = background_service("hc", lb);
        let task = background.task();
        gateway.lb = Some(task);

        server.add_service(background);
    } else {
        gateway.lb = Some(Arc::new(lb));
    }
    let mut proxy = http_proxy_service(&server.configuration, gateway);
    proxy.add_tcp(app.bind_addr.as_str());

    server.add_service(proxy);
    server.run_forever();
}

#[derive(Parser)]
pub struct App {
    /// Bind address
    #[clap(long = "ba")]
    bind_addr: String,

    #[clap(flatten)]
    gateway: Gateway,

    #[clap(flatten)]
    opt: Opt,
}

#[derive(Parser)]
pub struct Gateway {
    #[clap(skip = None)]
    lb: Option<Arc<LoadBalancer<RoundRobin>>>,
    /// Context path
    #[clap(long = "cp", default_value = "/")]
    ctx_path: String,
    /// Upstream address
    #[clap(long = "ua")]
    upstreams: Vec<String>,
    /// TLS
    #[clap(long)]
    tls: bool,
    /// SNI
    #[clap(long, default_value = "")]
    sni: String,
    // Health check frequency in seconds
    #[clap(long = "hcf", default_value = "0")]
    hc_freq: u64,
}

pub struct Ctx();

#[async_trait]
impl ProxyHttp for Gateway {
    type CTX = Ctx;

    fn new_ctx(&self) -> Self::CTX {
        Ctx()
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<Box<HttpPeer>> {
        let upstream = self.lb.as_ref().unwrap().select(b"", 256).unwrap();
        let peer = HttpPeer::new(upstream, self.tls, self.sni.to_string());
        return Ok(Box::new(peer));
    }

    async fn request_filter(
        &self,
        _session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<bool>
    where
        Self::CTX: Send + Sync,
    {
        if _session
            .req_header()
            .uri
            .path()
            .starts_with(self.ctx_path.as_str())
        {
            return Ok(false);
        }
        let _ = _session.respond_error(404).await;
        return Ok(true);
    }
}
