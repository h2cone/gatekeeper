use async_trait::async_trait;
use pingora::http::RequestHeader;
use pingora::prelude::{HttpPeer, Opt};
use pingora::proxy::{http_proxy_service, ProxyHttp, Session};
use pingora::server::Server;
use structopt::StructOpt;

fn main() {
    let app = App::from_args();
    let mut server = Server::new(Some(app.opt)).unwrap();
    server.bootstrap();

    let mut proxy = http_proxy_service(&server.configuration, app.gateway);
    proxy.add_tcp(app.bind_addr.as_str());
    server.add_service(proxy);

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
    /// Context path
    #[structopt(long = "cp", default_value = "/")]
    ctx_path: String,
    /// Peer address
    #[structopt(long = "pa")]
    peer_addr: String,
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

    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut Self::CTX) -> pingora::Result<Box<HttpPeer>> {
        let peer = HttpPeer::new(self.peer_addr.as_str(), self.tls, self.sni.to_string());
        return Ok(Box::new(peer));
    }

    async fn request_filter(&self, _session: &mut Session, _ctx: &mut Self::CTX) -> pingora::Result<bool> where Self::CTX: Send + Sync {
        if self.ctx_path.as_str() == "/" || check_uri(&_session.req_header(), self.ctx_path.as_str()) {
            return Ok(false);
        }
        let _ = _session.respond_error(404).await;
        return Ok(true);
    }
}

fn check_uri(req_header: &RequestHeader, prefix: &str) -> bool {
    req_header.uri.path().starts_with(prefix)
}
