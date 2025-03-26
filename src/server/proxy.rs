use async_trait::async_trait;
use pingora::http::RequestHeader;
use pingora::prelude::HttpPeer;
use pingora::proxy::{ProxyHttp, Session};

use crate::cmd::parser::Gateway;

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
        return Ok(false);
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
