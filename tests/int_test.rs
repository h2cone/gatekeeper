use openssl::asn1::Asn1Time;
use openssl::bn::{BigNum, MsbOption};
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private};
use openssl::rsa::Rsa;
use openssl::ssl::{SslAcceptor, SslMethod, SslStream};
use openssl::x509::extension::SubjectAlternativeName;
use openssl::x509::{X509NameBuilder, X509};
use socket2::{Domain, Protocol, Socket, Type};
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread::{self, sleep};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

struct ServerGuard {
    server: Child,
}

struct TempConfGuard {
    path: PathBuf,
}

impl TempConfGuard {
    fn with_max_retries(max_retries: usize) -> Self {
        let mut path = std::env::temp_dir();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System clock should be after UNIX_EPOCH")
            .as_nanos();
        path.push(format!(
            "gatekeeper-test-{}-{unique}.yaml",
            std::process::id()
        ));

        fs::write(&path, format!("version: 1\nmax_retries: {max_retries}\n"))
            .expect("Failed to write temporary server config");

        Self { path }
    }

    fn path(&self) -> &str {
        self.path
            .to_str()
            .expect("Temporary config path should be valid UTF-8")
    }
}

impl Drop for TempConfGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

impl ServerGuard {
    fn new(server: Child) -> Self {
        Self { server }
    }
}

impl Drop for ServerGuard {
    fn drop(&mut self) {
        let _ = self.server.kill();
    }
}

struct UpstreamGuard {
    addr: String,
    shutdown_tx: Option<Sender<()>>,
    handle: Option<thread::JoinHandle<()>>,
}

struct ReservedDeadUpstream {
    addr: String,
    _socket: Socket,
}

impl UpstreamGuard {
    fn new(addr: String, shutdown_tx: Sender<()>, handle: thread::JoinHandle<()>) -> Self {
        Self {
            addr,
            shutdown_tx: Some(shutdown_tx),
            handle: Some(handle),
        }
    }

    fn addr(&self) -> String {
        self.addr.clone()
    }
}

impl ReservedDeadUpstream {
    fn new() -> Self {
        let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))
            .expect("Failed to create reserved TCP socket");
        let bind_addr: SocketAddr = "127.0.0.1:0"
            .parse()
            .expect("Failed to parse reserved socket addr");
        socket
            .bind(&bind_addr.into())
            .expect("Failed to bind reserved TCP socket");
        let addr = socket
            .local_addr()
            .expect("Failed to read reserved socket addr")
            .as_socket()
            .expect("Reserved socket addr should be an IP socket")
            .to_string();

        Self {
            addr,
            _socket: socket,
        }
    }

    fn addr(&self) -> String {
        self.addr.clone()
    }
}

impl Drop for UpstreamGuard {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn start_server(
    bind_addr: &str,
    host: &str,
    upstreams: &[String],
    tls: bool,
    hc_freq: u64,
    sni: Option<&str>,
) -> ServerGuard {
    start_server_with_args(bind_addr, host, upstreams, tls, hc_freq, sni, &[])
}

fn start_server_with_args(
    bind_addr: &str,
    host: &str,
    upstreams: &[String],
    tls: bool,
    hc_freq: u64,
    sni: Option<&str>,
    extra_args: &[&str],
) -> ServerGuard {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_gatekeeper"));
    cmd.arg("--bind")
        .arg(bind_addr)
        .arg("--host")
        .arg(host)
        .arg("--hc-freq")
        .arg(hc_freq.to_string());

    if tls {
        cmd.arg("--tls");
    }

    if let Some(sni) = sni {
        if !sni.is_empty() {
            cmd.arg("--sni").arg(sni);
        }
    }

    for upstream in upstreams {
        cmd.arg("--upstream").arg(upstream);
    }

    cmd.args(extra_args);

    let server = cmd.spawn().expect("Failed to start server");
    ServerGuard::new(server)
}

fn wait_for_server_ready(addr: &str, timeout_secs: u64) -> bool {
    let start_time = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    while start_time.elapsed() < timeout {
        if TcpStream::connect(addr).is_ok() {
            sleep(Duration::from_millis(300));
            return true;
        }
        sleep(Duration::from_millis(100));
    }
    false
}

fn get_available_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to a temporary port");
    let addr = listener.local_addr().expect("Failed to get local address");
    addr.port()
}

#[cfg_attr(not(feature = "live_net"), allow(dead_code))]
fn wait_for_health_check(duration_secs: u64) {
    sleep(Duration::from_secs(duration_secs));
}

fn build_proxy_client(proxy_addr: &str) -> reqwest::Client {
    let proxy = reqwest::Proxy::http(format!("http://{}", proxy_addr))
        .expect("Failed to create HTTP proxy");
    reqwest::Client::builder()
        .proxy(proxy)
        .build()
        .expect("Failed to build client")
}

async fn wait_until_all_requests_hit_upstream(
    client: &reqwest::Client,
    url: &str,
    upstream_name: &str,
    attempts: usize,
    timeout_secs: u64,
) {
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    let expected = format!("upstream={}", upstream_name);
    let mut last_observation = String::new();

    while Instant::now() < deadline {
        let mut only_expected_upstream = true;

        for _ in 0..attempts {
            match client.get(url).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    let body = match resp.text().await {
                        Ok(body) => body,
                        Err(err) => {
                            last_observation = format!("failed to read body: {err}");
                            only_expected_upstream = false;
                            break;
                        }
                    };

                    if !status.is_success() || !body.contains(&expected) {
                        last_observation = format!("status={}, body={body}", status.as_u16());
                        only_expected_upstream = false;
                        break;
                    }
                }
                Err(err) => {
                    last_observation = format!("request failed: {err}");
                    only_expected_upstream = false;
                    break;
                }
            }
        }

        if only_expected_upstream {
            return;
        }

        sleep(Duration::from_millis(250));
    }

    panic!(
        "proxy never routed exclusively to {upstream_name}; last observation: {last_observation}"
    );
}

fn read_http_request(stream: &mut impl Read) -> Vec<u8> {
    let mut request = Vec::new();
    let mut buf = [0_u8; 1024];
    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                request.extend_from_slice(&buf[..n]);
                if request.windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
            }
            Err(err)
                if err.kind() == ErrorKind::WouldBlock || err.kind() == ErrorKind::TimedOut =>
            {
                break;
            }
            Err(_) => break,
        }
    }
    request
}

fn header_value(request: &[u8], name: &str) -> String {
    let request_str = String::from_utf8_lossy(request);
    for line in request_str.lines() {
        if line.trim().is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            if k.trim().eq_ignore_ascii_case(name) {
                return v.trim().to_string();
            }
        }
    }
    String::new()
}

fn write_http_response(stream: &mut impl Write, upstream_name: &str, request: &[u8]) {
    let host = header_value(request, "host");
    let body = format!("upstream={};host={}", upstream_name, host);
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn handle_plain_connection(stream: &mut TcpStream, upstream_name: &str) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    let request = read_http_request(stream);
    write_http_response(stream, upstream_name, &request);
}

fn handle_tls_connection(stream: &mut SslStream<TcpStream>, upstream_name: &str) {
    let request = read_http_request(stream);
    write_http_response(stream, upstream_name, &request);
}

fn should_shutdown(shutdown_rx: &Receiver<()>) -> bool {
    match shutdown_rx.try_recv() {
        Ok(()) => true,
        Err(TryRecvError::Disconnected) => true,
        Err(TryRecvError::Empty) => false,
    }
}

fn spawn_http_upstream(upstream_name: &str) -> UpstreamGuard {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to start local HTTP upstream");
    listener
        .set_nonblocking(true)
        .expect("Failed to set listener nonblocking");
    let addr = listener.local_addr().expect("Failed to get HTTP addr");
    let upstream_name = upstream_name.to_string();
    let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>();

    let handle = thread::spawn(move || loop {
        if should_shutdown(&shutdown_rx) {
            break;
        }

        match listener.accept() {
            Ok((mut stream, _)) => handle_plain_connection(&mut stream, &upstream_name),
            Err(err) if err.kind() == ErrorKind::WouldBlock => sleep(Duration::from_millis(20)),
            Err(_) => break,
        }
    });

    UpstreamGuard::new(addr.to_string(), shutdown_tx, handle)
}

fn generate_self_signed_cert() -> (PKey<Private>, X509) {
    let rsa = Rsa::generate(2048).expect("Failed to generate RSA key");
    let pkey = PKey::from_rsa(rsa).expect("Failed to build private key");

    let mut name = X509NameBuilder::new().expect("Failed to create X509NameBuilder");
    name.append_entry_by_nid(Nid::COMMONNAME, "localhost")
        .expect("Failed to set CN");
    let name = name.build();

    let mut builder = openssl::x509::X509Builder::new().expect("Failed to create X509Builder");
    builder.set_version(2).expect("Failed to set version");
    let mut serial = BigNum::new().expect("Failed to create serial");
    serial
        .rand(64, MsbOption::MAYBE_ZERO, false)
        .expect("Failed to randomize serial");
    let serial = serial.to_asn1_integer().expect("Failed to convert serial");
    builder
        .set_serial_number(&serial)
        .expect("Failed to set serial");
    builder
        .set_subject_name(&name)
        .expect("Failed to set subject name");
    builder
        .set_issuer_name(&name)
        .expect("Failed to set issuer name");
    builder.set_pubkey(&pkey).expect("Failed to set public key");
    let not_before = Asn1Time::days_from_now(0).expect("Failed to set not_before");
    let not_after = Asn1Time::days_from_now(30).expect("Failed to set not_after");
    builder
        .set_not_before(not_before.as_ref())
        .expect("Failed to apply not_before");
    builder
        .set_not_after(not_after.as_ref())
        .expect("Failed to apply not_after");

    let context = builder.x509v3_context(None, None);
    let san = SubjectAlternativeName::new()
        .dns("localhost")
        .build(&context)
        .expect("Failed to build SAN");
    builder
        .append_extension(san)
        .expect("Failed to append SAN extension");

    builder
        .sign(&pkey, MessageDigest::sha256())
        .expect("Failed to sign cert");
    (pkey, builder.build())
}

fn spawn_tls_upstream(upstream_name: &str) -> UpstreamGuard {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to start local TLS upstream");
    listener
        .set_nonblocking(true)
        .expect("Failed to set listener nonblocking");
    let addr = listener.local_addr().expect("Failed to get TLS addr");
    let upstream_name = upstream_name.to_string();
    let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>();

    let (key, cert) = generate_self_signed_cert();
    let mut acceptor =
        SslAcceptor::mozilla_intermediate(SslMethod::tls()).expect("Failed to init acceptor");
    acceptor
        .set_private_key(&key)
        .expect("Failed to set private key");
    acceptor
        .set_certificate(&cert)
        .expect("Failed to set certificate");
    acceptor.check_private_key().expect("Invalid cert/key pair");
    let acceptor = acceptor.build();

    let handle = thread::spawn(move || loop {
        if should_shutdown(&shutdown_rx) {
            break;
        }

        match listener.accept() {
            Ok((stream, _)) => {
                let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
                if let Ok(mut tls_stream) = acceptor.accept(stream) {
                    handle_tls_connection(&mut tls_stream, &upstream_name);
                }
            }
            Err(err) if err.kind() == ErrorKind::WouldBlock => sleep(Duration::from_millis(20)),
            Err(_) => break,
        }
    });

    UpstreamGuard::new(addr.to_string(), shutdown_tx, handle)
}

#[tokio::test]
async fn test_proxy_local_http() {
    let upstream = spawn_http_upstream("primary");
    let proxy_addr = format!("127.0.0.1:{}", get_available_port());
    let override_host = "local.test";
    let request_host = "client.local";
    let _server = start_server(
        &proxy_addr,
        override_host,
        &[upstream.addr()],
        false,
        0,
        None,
    );

    assert!(
        wait_for_server_ready(&proxy_addr, 10),
        "The proxy failed to start in time"
    );

    let client = build_proxy_client(&proxy_addr);
    let resp = client
        .get(format!("http://{request_host}/json"))
        .send()
        .await
        .expect("Failed to send request");

    assert!(resp.status().is_success());
    let body = resp.text().await.expect("Failed to read body");
    assert!(
        body.contains("upstream=primary"),
        "unexpected body: {}",
        body
    );
    assert!(
        body.contains(&format!("host={override_host}")),
        "unexpected body: {}",
        body
    );
}

#[tokio::test]
async fn test_proxy_multiple_upstreams() {
    let upstream_a = spawn_http_upstream("a");
    let upstream_b = spawn_http_upstream("b");
    let proxy_addr = format!("127.0.0.1:{}", get_available_port());
    let _server = start_server(
        &proxy_addr,
        "multi.local",
        &[upstream_a.addr(), upstream_b.addr()],
        false,
        0,
        None,
    );

    assert!(
        wait_for_server_ready(&proxy_addr, 10),
        "The proxy failed to start in time"
    );

    let client = build_proxy_client(&proxy_addr);
    let mut saw_a = false;
    let mut saw_b = false;

    for _ in 0..8 {
        let resp = client
            .get("http://multi.local/")
            .send()
            .await
            .expect("Failed to send request");
        assert!(resp.status().is_success());
        let body = resp.text().await.expect("Failed to read body");
        saw_a |= body.contains("upstream=a");
        saw_b |= body.contains("upstream=b");
        if saw_a && saw_b {
            break;
        }
    }

    assert!(saw_a, "did not receive response from upstream a");
    assert!(saw_b, "did not receive response from upstream b");
}

#[tokio::test]
async fn test_proxy_with_tls() {
    let upstream = spawn_tls_upstream("tls");
    let proxy_addr = format!("127.0.0.1:{}", get_available_port());
    let override_host = "tls.local";
    let request_host = "client.tls.local";
    let _server = start_server(
        &proxy_addr,
        override_host,
        &[upstream.addr()],
        true,
        0,
        None,
    );

    assert!(
        wait_for_server_ready(&proxy_addr, 10),
        "The proxy failed to start in time"
    );

    let client = build_proxy_client(&proxy_addr);
    let resp = client
        .get(format!("http://{request_host}/"))
        .send()
        .await
        .expect("Failed to send request");

    assert!(resp.status().is_success());
    let body = resp.text().await.expect("Failed to read body");
    assert!(body.contains("upstream=tls"), "unexpected body: {}", body);
    assert!(
        body.contains(&format!("host={override_host}")),
        "unexpected body: {}",
        body
    );
}

#[tokio::test]
async fn test_health_check() {
    let healthy = spawn_http_upstream("healthy");
    let unhealthy = ReservedDeadUpstream::new();
    let proxy_addr = format!("127.0.0.1:{}", get_available_port());
    let conf = TempConfGuard::with_max_retries(1);
    let conf_args = ["-c", conf.path()];
    let _server = start_server_with_args(
        &proxy_addr,
        "health.local",
        &[unhealthy.addr(), healthy.addr()],
        false,
        1,
        None,
        &conf_args,
    );

    assert!(
        wait_for_server_ready(&proxy_addr, 15),
        "The proxy failed to start in time"
    );

    let client = build_proxy_client(&proxy_addr);
    wait_until_all_requests_hit_upstream(&client, "http://health.local/", "healthy", 6, 10).await;
}

#[cfg(feature = "live_net")]
mod live_net {
    use super::*;

    const EXAMPLE_COM_IP: &str = "96.7.128.198";
    const EXAMPLE_NET_IP: &str = "23.215.0.135";
    const EXAMPLE_ORG_IP: &str = "23.215.0.132";
    const EXAMPLE_EDU_IP: &str = "96.7.129.25";

    #[tokio::test]
    async fn test_live_net_proxy_example_com() {
        let proxy_addr = format!("127.0.0.1:{}", get_available_port());
        let _server = start_server(
            &proxy_addr,
            "example.com",
            &[format!("{}:80", EXAMPLE_COM_IP)],
            false,
            5,
            None,
        );

        assert!(
            wait_for_server_ready(&proxy_addr, 10),
            "The proxy failed to start in time"
        );

        let client = build_proxy_client(&proxy_addr);
        let resp = client
            .get("http://example.com")
            .send()
            .await
            .expect("Failed to send request");

        assert!(resp.status().is_success());
        let body = resp.text().await.expect("Failed to read response body");
        assert!(body.contains("<h1>Example Domain</h1>"));
    }

    #[tokio::test]
    async fn test_live_net_proxy_multiple_upstreams() {
        let proxy_addr = format!("127.0.0.1:{}", get_available_port());
        let _server = start_server(
            &proxy_addr,
            "example.org",
            &[
                format!("{}:80", EXAMPLE_ORG_IP),
                format!("{}:80", EXAMPLE_NET_IP),
            ],
            false,
            5,
            None,
        );

        assert!(
            wait_for_server_ready(&proxy_addr, 10),
            "The proxy failed to start in time"
        );

        let client = build_proxy_client(&proxy_addr);
        let resp = client
            .get("http://example.org")
            .send()
            .await
            .expect("Failed to send request");

        assert!(resp.status().is_success());
    }

    #[tokio::test]
    async fn test_live_net_proxy_with_tls() {
        let proxy_addr = format!("127.0.0.1:{}", get_available_port());
        let _server = start_server(
            &proxy_addr,
            "example.edu",
            &[format!("{}:443", EXAMPLE_EDU_IP)],
            true,
            5,
            None,
        );

        assert!(
            wait_for_server_ready(&proxy_addr, 10),
            "The proxy failed to start in time"
        );

        let client = build_proxy_client(&proxy_addr);
        let resp = client
            .get("https://example.edu")
            .send()
            .await
            .expect("Failed to send request");

        assert!(resp.status().is_success());
    }

    #[tokio::test]
    async fn test_live_net_health_check() {
        let proxy_addr = format!("127.0.0.1:{}", get_available_port());
        let _server = start_server(
            &proxy_addr,
            "example.net",
            &["1.2.3.4:80".to_string(), format!("{}:80", EXAMPLE_NET_IP)],
            false,
            5,
            None,
        );

        assert!(
            wait_for_server_ready(&proxy_addr, 15),
            "The proxy failed to start in time"
        );

        wait_for_health_check(12);
        let client = build_proxy_client(&proxy_addr);
        let mut attempts = 3;
        let mut ok = false;

        while attempts > 0 {
            if let Ok(resp) = client.get("http://example.net").send().await {
                if resp.status().is_success() {
                    ok = true;
                    break;
                }
            }
            attempts -= 1;
            if attempts > 0 {
                sleep(Duration::from_secs(5));
            }
        }

        assert!(ok, "All live_net attempts failed");
    }
}
