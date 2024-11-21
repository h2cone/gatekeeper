// @author h2cone

mod cmd;

use async_trait::async_trait;
use pingora::http::RequestHeader;
use pingora::lb::{health_check, LoadBalancer};
use pingora::prelude::{background_service, HttpPeer, RoundRobin};
use pingora::proxy::{http_proxy_service, ProxyHttp, Session};
use pingora::server::Server;
use std::sync::Arc;
use std::time::Duration;

use cmd::parser::{App, Gateway};

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

    if app.cert_path.is_empty() {
        proxy.add_tcp(app.bind_addr.as_str());
    } else {
        let tls_settings =
            pingora::listeners::TlsSettings::intermediate(&app.cert_path, &app.key_path).unwrap();
        proxy.add_tls_with_settings(&app.bind_addr, None, tls_settings);
    }
    server.add_service(proxy);
    server.run_forever();
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
        let mut peer = HttpPeer::new(upstream, self.tls, self.sni.to_string());
        peer.options.set_http_version(2, 1);
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

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> pingora::Result<()>
    where
        Self::CTX: Send + Sync,
    {
        if self.host.is_empty() {
            return Ok(());
        }
        upstream_request
            .insert_header("Host", self.host.as_str())
            .unwrap();
        Ok(())
    }
}
