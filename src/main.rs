mod cmd;
mod server;

use pingora::lb::{health_check, LoadBalancer};
use pingora::prelude::{background_service, RoundRobin};
use pingora::proxy::http_proxy_service;
use pingora::server::Server;
use std::sync::Arc;
use std::time::Duration;

use cmd::parser::App;

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
