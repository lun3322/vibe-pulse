use std::{
    io::Read,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use serde_json::Value;
use tiny_http::{Method, Request, Response, Server, StatusCode};
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::PostMessageW,
};

use crate::model::{ClientKind, HookEvent};

pub const PORT: u16 = 17_321;
const ALL_IPV4_INTERFACES: &str = "0.0.0.0";
const IPV6_LOOPBACK: &str = "::1";
const MAX_BODY_BYTES: usize = 256 * 1024;
const RECEIVE_TIMEOUT: Duration = Duration::from_millis(100);

pub struct HookServer {
    stop: Arc<AtomicBool>,
    threads: Vec<JoinHandle<()>>,
}

impl HookServer {
    pub fn start(
        hwnd: HWND,
        message: u32,
        token: String,
        events: Sender<HookEvent>,
    ) -> Result<Self, String> {
        let servers = create_servers(PORT)?;
        let stop = Arc::new(AtomicBool::new(false));
        let window = hwnd.0 as usize;
        let threads = servers
            .into_iter()
            .map(|server| {
                let thread_stop = Arc::clone(&stop);
                let thread_token = token.clone();
                let thread_events = events.clone();
                thread::spawn(move || {
                    serve(
                        server,
                        window,
                        message,
                        &thread_token,
                        &thread_stop,
                        &thread_events,
                    );
                })
            })
            .collect();
        Ok(Self { stop, threads })
    }
}

impl Drop for HookServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        for thread in self.threads.drain(..) {
            let _ = thread.join();
        }
    }
}

fn create_servers(port: u16) -> Result<Vec<Server>, String> {
    let ipv4_address = format!("{ALL_IPV4_INTERFACES}:{port}");
    let ipv4 = create_server(&ipv4_address)?;
    let bound_port = ipv4
        .server_addr()
        .to_ip()
        .expect("TCP 监听地址必须包含端口")
        .port();
    let ipv6_address = format!("[{IPV6_LOOPBACK}]:{bound_port}");
    let ipv6 = create_server(&ipv6_address)?;
    Ok(vec![ipv4, ipv6])
}

fn create_server(address: &str) -> Result<Server, String> {
    Server::http(address).map_err(|error| format!("无法监听 HTTP Hook 地址 {address}：{error}"))
}

fn serve(
    server: Server,
    window: usize,
    message: u32,
    token: &str,
    stop: &AtomicBool,
    events: &Sender<HookEvent>,
) {
    while !stop.load(Ordering::Acquire) {
        match server.recv_timeout(RECEIVE_TIMEOUT) {
            Ok(Some(request)) => handle_request(request, window, message, token, events),
            Ok(None) => {}
            Err(_) => return,
        }
    }
}

fn handle_request(
    mut request: Request,
    window: usize,
    message: u32,
    token: &str,
    events: &Sender<HookEvent>,
) {
    let event = match parse_event(&mut request, token) {
        Ok(event) => event,
        Err(status) => {
            respond(request, status);
            return;
        }
    };
    if events.send(event).is_err() {
        respond(request, 503);
        return;
    }
    let hwnd = HWND(window as *mut _);
    if unsafe { PostMessageW(Some(hwnd), message, WPARAM(0), LPARAM(0)) }.is_err() {
        respond(request, 503);
        return;
    }
    respond(request, 204);
}

fn parse_event(request: &mut Request, token: &str) -> Result<HookEvent, u16> {
    if request.method() != &Method::Post || request.url() != "/hooks" {
        return Err(404);
    }
    if !authorized(request, token) {
        return Err(401);
    }
    if request
        .body_length()
        .is_some_and(|length| length > MAX_BODY_BYTES)
    {
        return Err(413);
    }
    let client = ClientKind::from_header(header(request, "X-Vibe-Client"));
    let host = header(request, "X-Vibe-Host")
        .map(str::to_owned)
        .or_else(|| {
            request
                .remote_addr()
                .map(|address| address.ip().to_string())
        })
        .unwrap_or_else(|| "未知主机".to_owned());
    let mut body = Vec::new();
    let read_result = request
        .as_reader()
        .take((MAX_BODY_BYTES + 1) as u64)
        .read_to_end(&mut body);
    if read_result.is_err() || body.len() > MAX_BODY_BYTES {
        return Err(413);
    }
    let value = serde_json::from_slice::<Value>(&body).map_err(|_| 400u16)?;
    HookEvent::from_json(client, host, &value).map_err(|_| 400u16)
}

fn authorized(request: &Request, token: &str) -> bool {
    let expected = format!("Bearer {token}");
    header(request, "Authorization").is_some_and(|value| secure_equal(value, &expected))
}

fn secure_equal(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.bytes()
        .zip(right.bytes())
        .fold(0u8, |difference, (left, right)| difference | (left ^ right))
        == 0
}

fn header<'a>(request: &'a Request, name: &'static str) -> Option<&'a str> {
    request
        .headers()
        .iter()
        .find(|header| header.field.equiv(name))
        .map(|header| header.value.as_str())
}

fn respond(request: Request, status: u16) {
    let _ = request.respond(Response::empty(StatusCode(status)));
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpStream},
        sync::{Arc, Barrier},
        thread,
        time::Duration,
    };

    use tiny_http::{Response, Server, StatusCode};

    use super::{create_servers, secure_equal};

    const REQUESTS_PER_ADDRESS: usize = 8;
    const TEST_TIMEOUT: Duration = Duration::from_secs(2);

    #[test]
    fn binds_all_ipv4_interfaces_and_handles_ipv4_and_ipv6_requests() {
        let servers = create_servers(0).unwrap();
        let bound_addresses = servers
            .iter()
            .map(|server| server.server_addr().to_ip().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(bound_addresses[0].ip(), IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        assert_eq!(bound_addresses[1].ip(), IpAddr::V6(Ipv6Addr::LOCALHOST));
        let addresses = servers.iter().map(client_address).collect::<Vec<_>>();
        assert_eq!(addresses[0].port(), addresses[1].port());
        let workers = servers
            .into_iter()
            .map(|server| {
                thread::spawn(move || {
                    for _ in 0..REQUESTS_PER_ADDRESS {
                        let request = server.recv_timeout(TEST_TIMEOUT).unwrap().unwrap();
                        request.respond(Response::empty(StatusCode(204))).unwrap();
                    }
                })
            })
            .collect::<Vec<_>>();
        let client_count = addresses.len() * REQUESTS_PER_ADDRESS;
        let barrier = Arc::new(Barrier::new(client_count + 1));
        let mut clients = Vec::with_capacity(client_count);
        for address in addresses {
            for _ in 0..REQUESTS_PER_ADDRESS {
                let client_barrier = Arc::clone(&barrier);
                clients.push(thread::spawn(move || {
                    client_barrier.wait();
                    send_request(address);
                }));
            }
        }
        barrier.wait();
        for client in clients {
            client.join().unwrap();
        }
        for worker in workers {
            worker.join().unwrap();
        }
    }

    #[test]
    fn token_comparison_requires_exact_value() {
        assert!(secure_equal("Bearer secret", "Bearer secret"));
        assert!(!secure_equal("Bearer secret", "Bearer Secret"));
        assert!(!secure_equal("Bearer secret", "Bearer secret-extra"));
    }

    fn client_address(server: &Server) -> SocketAddr {
        let address = server.server_addr().to_ip().unwrap();
        match address {
            SocketAddr::V4(address) => {
                SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), address.port())
            }
            SocketAddr::V6(address) => {
                SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), address.port())
            }
        }
    }

    fn send_request(address: SocketAddr) {
        let mut stream = TcpStream::connect_timeout(&address, TEST_TIMEOUT).unwrap();
        stream.set_read_timeout(Some(TEST_TIMEOUT)).unwrap();
        stream
            .write_all(
                b"POST /hooks HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        assert!(response.starts_with("HTTP/1.1 204"));
    }
}
