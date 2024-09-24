// @author h2cone

use async_trait::async_trait;
use pingora::http::RequestHeader;
use pingora::lb::{health_check, LoadBalancer};
use pingora::prelude::{background_service, HttpPeer, Opt, RoundRobin};
use pingora::proxy::{http_proxy_service, ProxyHttp, Session};
use pingora::server::Server;
use std::sync::Arc;
use std::time::Duration;
use structopt::StructOpt;

fn main() {
    env_logger::init();

    let app = App::from_args();
    let mut server = Server::new(Some(app.opt)).unwrap();
    server.bootstrap();

    let mut gateway = app.gateway;
    let mut lb = LoadBalancer::<RoundRobin>::try_from_iter(&gateway.upstreams).unwrap();

    let hc = health_check::TcpHealthCheck::new();
    lb.set_health_check(hc);
    lb.health_check_frequency = Some(Duration::from_secs(1));

    let background = background_service("health-check", lb);
    let task = background.task();
    gateway.lb = Some(task);

    let mut proxy = http_proxy_service(&server.configuration, gateway);
    proxy.add_tcp(app.bind_addr.as_str());

    server.add_service(proxy);
    server.add_service(background);
    server.run_forever();
}

#[derive(StructOpt)]
pub struct App {
    /// Bind address
    #[structopt(long = "ba")]
    bind_addr: String,

    #[structopt(flatten)]
    gateway: Gateway,

    #[structopt(flatten)]
    opt: Opt,
}

#[derive(StructOpt)]
pub struct Gateway {
    #[structopt(skip = None)]
    lb: Option<Arc<LoadBalancer<RoundRobin>>>,
    /// Context path
    #[structopt(long = "cp", default_value = "/")]
    ctx_path: String,
    /// Upstream address
    #[structopt(long = "ua")]
    upstreams: Vec<String>,
    /// TLS
    #[structopt(long)]
    tls: bool,
    /// SNI
    #[structopt(long, default_value = "")]
    sni: String,
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
        if self.ctx_path.as_str() == "/"
            || check_uri(&_session.req_header(), self.ctx_path.as_str())
        {
            return Ok(false);
        }
        let _ = _session.respond_error(404).await;
        return Ok(true);
    }
}

fn check_uri(req_header: &RequestHeader, prefix: &str) -> bool {
    req_header.uri.path().starts_with(prefix)
}
