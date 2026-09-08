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
const MAX_BODY_BYTES: usize = 256 * 1024;
const RECEIVE_TIMEOUT: Duration = Duration::from_millis(100);

pub struct HookServer {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl HookServer {
    pub fn start(
        hwnd: HWND,
        message: u32,
        token: String,
        allow_lan: bool,
        events: Sender<HookEvent>,
    ) -> Result<Self, String> {
        let address = if allow_lan {
            format!("0.0.0.0:{PORT}")
        } else {
            format!("127.0.0.1:{PORT}")
        };
        let server = Server::http(&address)
            .map_err(|error| format!("无法监听 HTTP Hook 地址 {address}：{error}"))?;
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let window = hwnd.0 as usize;
        let thread = thread::spawn(move || {
            serve(server, window, message, &token, &thread_stop, &events);
        });
        Ok(Self {
            stop,
            thread: Some(thread),
        })
    }
}

impl Drop for HookServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
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
    use super::secure_equal;

    #[test]
    fn token_comparison_requires_exact_value() {
        assert!(secure_equal("Bearer secret", "Bearer secret"));
        assert!(!secure_equal("Bearer secret", "Bearer Secret"));
        assert!(!secure_equal("Bearer secret", "Bearer secret-extra"));
    }
}
