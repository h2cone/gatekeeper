use reqwest;
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command};
use std::thread::sleep;
use std::time::{Duration, Instant};

struct ServerGuard {
    server: Child,
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

const EXAMPLE_COM_IP: &str = "96.7.128.198";
const EXAMPLE_NET_IP: &str = "23.215.0.135";
const EXAMPLE_ORG_IP: &str = "23.215.0.132";
const EXAMPLE_EDU_IP: &str = "96.7.129.25";

fn start_server(bind_addr: &str, host: &str, upstreams: &[&str], tls: bool) -> ServerGuard {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_gatekeeper"));
    cmd.arg("--bind")
        .arg(bind_addr)
        .arg("--host")
        .arg(host)
        .arg("--hc-freq")
        .arg("5");

    if tls {
        cmd.arg("--tls");
    }

    for upstream in upstreams {
        cmd.arg("--upstream").arg(upstream);
    }

    let server = cmd.spawn().expect("Failed to start server");
    ServerGuard::new(server)
}

/// Waits for the server to start and be ready to accept connections
///
/// Parameters:
/// * `addr` - Server address (e.g., "127.0.0.1:8081")
/// * `timeout_secs` - Maximum wait time (in seconds)
///
/// Returns:
/// * `true` - If the server starts successfully
/// * `false` - If the server fails to start within the timeout period
fn wait_for_server_ready(addr: &str, timeout_secs: u64) -> bool {
    let start_time = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    while start_time.elapsed() < timeout {
        if TcpStream::connect(addr).is_ok() {
            // Give the server a little extra time to complete initialization
            sleep(Duration::from_millis(500));
            return true;
        }
        sleep(Duration::from_millis(100));
    }
    false
}

/// Get an available port
fn get_available_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to a temporary port");
    let addr = listener.local_addr().expect("Failed to get local address");
    addr.port()
}

/// Wait for health check to complete
///
/// This function waits long enough for health checks to detect unhealthy upstream servers
fn wait_for_health_check(duration_secs: u64) {
    // The health check frequency is 5 seconds, wait for at least two check cycles plus some extra time
    let wait_time = Duration::from_secs(duration_secs);
    sleep(wait_time);
}

#[tokio::test]
async fn test_proxy_example_com() {
    // Get a dynamically assigned port
    let port = get_available_port();
    let proxy_addr = format!("127.0.0.1:{}", port);

    let _server = start_server(
        &proxy_addr,
        "example.com",
        &[&format!("{}:80", EXAMPLE_COM_IP)],
        false,
    );

    // Wait for the server to be ready
    assert!(
        wait_for_server_ready(&proxy_addr, 10),
        "The server failed to start within the specified time"
    );

    // Create a proxy client
    let proxy = reqwest::Proxy::http(&format!("http://{}", proxy_addr))
        .expect("Failed to create HTTP proxy");

    let client = reqwest::Client::builder()
        .proxy(proxy)
        .build()
        .expect("Failed to build client");

    // Send a request through the proxy
    let resp = client
        .get("http://example.com")
        .send()
        .await
        .expect("Failed to send request");

    assert!(resp.status().is_success());
    let body = resp.text().await.expect("Failed to read response body");
    assert!(body.contains("<h1>Example Domain</h1>"));

    // Server will be automatically killed when _server goes out of scope
}

#[tokio::test]
async fn test_proxy_multiple_upstreams() {
    let port = get_available_port();
    let proxy_addr = format!("127.0.0.1:{}", port);

    let _server = start_server(
        &proxy_addr,
        "example.org",
        &[
            &format!("{}:80", EXAMPLE_ORG_IP),
            &format!("{}:80", EXAMPLE_NET_IP),
        ],
        false,
    );

    // Wait for the server to be ready
    assert!(
        wait_for_server_ready(proxy_addr.as_str(), 10),
        "The server failed to start within the specified time"
    );

    let proxy = reqwest::Proxy::http(&format!("http://{}", proxy_addr))
        .expect("Failed to create HTTP proxy");

    let client = reqwest::Client::builder()
        .proxy(proxy)
        .build()
        .expect("Failed to build client");

    let resp = client
        .get("http://example.org")
        .send()
        .await
        .expect("Failed to send request");

    assert!(resp.status().is_success());

    // Server will be automatically killed when _server goes out of scope
}

#[tokio::test]
async fn test_proxy_with_tls() {
    let port = get_available_port();
    let proxy_addr = format!("127.0.0.1:{}", port);

    let _server = start_server(
        &proxy_addr,
        "example.edu",
        &[&format!("{}:443", EXAMPLE_EDU_IP)],
        true,
    );

    // Wait for the server to be ready
    assert!(
        wait_for_server_ready(proxy_addr.as_str(), 10),
        "The server failed to start within the specified time"
    );

    let proxy = reqwest::Proxy::http(&format!("http://{}", proxy_addr))
        .expect("Failed to create HTTP proxy");

    let client = reqwest::Client::builder()
        .proxy(proxy)
        // Accepting invalid certificates in a test environment
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to build client");

    let resp = client
        .get("https://example.edu")
        .send()
        .await
        .expect("Failed to send request");

    assert!(resp.status().is_success());

    // Server will be automatically killed when _server goes out of scope
}

#[tokio::test]
async fn test_health_check() {
    let port = get_available_port();
    let proxy_addr = format!("127.0.0.1:{}", port);

    let _server = start_server(
        &proxy_addr,
        "example.net",
        &["1.2.3.4:80", &format!("{}:80", EXAMPLE_NET_IP)],
        false,
    );

    // Wait for the server to be ready
    assert!(
        wait_for_server_ready(proxy_addr.as_str(), 15),
        "The server failed to start within the specified time"
    );

    // Wait for health check to complete, set to 12 seconds to allow at least two check cycles to complete
    wait_for_health_check(12);

    let proxy = reqwest::Proxy::http(&format!("http://{}", proxy_addr))
        .expect("Failed to create HTTP proxy");

    let client = reqwest::Client::builder()
        .proxy(proxy)
        .build()
        .expect("Failed to build client");

    // Add retry logic to prevent health check from not being fully effective
    let mut attempts = 3;
    let mut resp = None;

    while attempts > 0 {
        match client.get("http://example.net").send().await {
            Ok(response) if response.status().is_success() => {
                resp = Some(response);
                break;
            }
            Ok(_) | Err(_) => {
                attempts -= 1;
                // If the request fails, wait a few seconds for the health check to have a chance to take effect
                if attempts > 0 {
                    sleep(Duration::from_secs(5));
                }
            }
        }
    }

    let resp = resp.expect("All request attempts failed");
    assert!(resp.status().is_success());

    // Server will be automatically killed when _server goes out of scope
}
