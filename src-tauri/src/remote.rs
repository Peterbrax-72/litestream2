//! Wireless remote control server (desktop only).
//!
//! A tiny HTTP + WebSocket server that lives inside the LiteStream desktop app.
//! A phone on the same Wi-Fi opens `http://<pc-address>:<port>/`, enters the PIN,
//! then receives live Preview/Program frames and sends control commands.
//!
//! Everything here is plain `std` plus `tungstenite`. Commands from the phone are
//! queued as opaque JSON text for the desktop UI to poll; the UI pushes state and
//! JPEG frames back. Nothing is reachable until the user turns the server on, and
//! every WebSocket must send the correct PIN first.

use std::collections::VecDeque;
use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tungstenite::protocol::WebSocketConfig;
use tungstenite::{accept_with_config, Message};

const MAX_CLIENTS: usize = 8;
const MAX_QUEUE: usize = 200;
const MAX_MESSAGE: usize = 16 * 1024;
const FAIL_LIMIT: usize = 8;
const FAIL_WINDOW: Duration = Duration::from_secs(60);

enum Out {
    Text(String),
    Bin(Vec<u8>),
}

struct Shared {
    pin: Mutex<String>,
    page: &'static str,
    queue: Mutex<VecDeque<String>>,
    clients: Mutex<Vec<(u64, SyncSender<Out>)>>,
    state: Mutex<String>,
    fails: Mutex<VecDeque<Instant>>,
    stop: AtomicBool,
    next_id: AtomicU64,
}

pub struct Remote {
    shared: Mutex<Option<(Arc<Shared>, u16)>>,
}

impl Default for Remote {
    fn default() -> Self {
        Remote { shared: Mutex::new(None) }
    }
}

impl Drop for Remote {
    fn drop(&mut self) {
        self.stop();
    }
}

fn constant_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut diff = (a.len() != b.len()) as u8;
    for i in 0..a.len().max(b.len()) {
        diff |= a.get(i).copied().unwrap_or(0) ^ b.get(i).copied().unwrap_or(1);
    }
    diff == 0
}

/// The address a phone on the same network should use to reach this PC.
pub fn local_ip() -> String {
    for probe in ["8.8.8.8:80", "192.168.1.1:80", "10.0.0.1:80", "172.16.0.1:80"] {
        if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
            if sock.connect(probe).is_ok() {
                if let Ok(addr) = sock.local_addr() {
                    let ip = addr.ip().to_string();
                    if ip != "0.0.0.0" && !ip.starts_with("127.") {
                        return ip;
                    }
                }
            }
        }
    }
    "127.0.0.1".to_string()
}


/// Every IPv4 address this PC has on a network, best first. Works with no internet,
/// for example on a router with no uplink, a phone hotspot, or this PC's own Mobile Hotspot.
pub fn local_ips() -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let routed = local_ip();
    if !routed.starts_with("127.") {
        out.push(routed);
    }
    for ip in interface_ips() {
        if !out.contains(&ip) {
            out.push(ip);
        }
    }
    if out.is_empty() {
        out.push("127.0.0.1".to_string());
    }
    out
}

fn interface_ips() -> Vec<String> {
    let text = interface_text();
    let mut ips = parse_ips(&text);
    // private LAN ranges first, link-local (169.254.x.x) last
    ips.sort_by_key(|ip| if ip.starts_with("169.254.") { 2 } else if is_private(ip) { 0 } else { 1 });
    ips
}

fn is_private(ip: &str) -> bool {
    match ip.parse::<std::net::Ipv4Addr>() {
        Ok(a) => a.is_private(),
        Err(_) => false,
    }
}

/// Pull usable IPv4 addresses out of `ipconfig` / `ip addr` / `hostname -I` output.
fn parse_ips(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        let lower = line.to_ascii_lowercase();
        let wanted = lower.contains("ipv4") || lower.contains("inet ") || !line.contains(':') && !lower.contains("mask") && !lower.contains("gateway");
        if !wanted || lower.contains("mask") || lower.contains("gateway") || lower.contains("dhcp") || lower.contains("dns") {
            continue;
        }
        for token in line.split(|c: char| c.is_whitespace() || c == ':' || c == '(' || c == ')' || c == '/') {
            if let Ok(a) = token.parse::<std::net::Ipv4Addr>() {
                if !a.is_loopback() && !a.is_unspecified() && !a.is_broadcast() {
                    let s = a.to_string();
                    if !out.contains(&s) {
                        out.push(s);
                    }
                }
                break;
            }
        }
    }
    out
}

#[allow(unused_mut)]
fn interface_text() -> String {
    use std::process::{Command, Stdio};
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let mut c = Command::new("ipconfig");
        c.creation_flags(0x08000000).stdout(Stdio::piped()).stderr(Stdio::null());
        if let Ok(o) = c.output() {
            return String::from_utf8_lossy(&o.stdout).to_string();
        }
        String::new()
    }
    #[cfg(not(windows))]
    {
        for (prog, args) in [("hostname", vec!["-I"]), ("ip", vec!["-4", "-o", "addr"])] {
            if let Ok(o) = Command::new(prog).args(&args).stdout(Stdio::piped()).stderr(Stdio::null()).output() {
                if o.status.success() && !o.stdout.is_empty() {
                    return String::from_utf8_lossy(&o.stdout).to_string();
                }
            }
        }
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_windows_ipconfig() {
        let t = "Wireless LAN adapter Wi-Fi:
   Connection-specific DNS Suffix  . :
   IPv4 Address. . . . . . . . . . . : 192.168.43.17(Preferred)
   Subnet Mask . . . . . . . . . . . : 255.255.255.0
   Default Gateway . . . . . . . . . : 192.168.43.1
Ethernet adapter vEthernet (WSL):
   Autoconfiguration IPv4 Address. . : 169.254.12.9(Preferred)
   Adresse IPv4. . . . . . . . . . . : 172.20.80.1
";
        assert_eq!(parse_ips(t), vec!["192.168.43.17", "169.254.12.9", "172.20.80.1"]);
    }

    #[test]
    fn parses_linux_outputs() {
        assert_eq!(parse_ips("192.168.1.5 10.0.0.2 
"), vec!["192.168.1.5"]);
        assert_eq!(parse_ips("2: wlan0    inet 192.168.1.5/24 brd 192.168.1.255 scope global wlan0
"), vec!["192.168.1.5"]);
    }
}

impl Remote {
    /// Start listening. Returns the port actually used. `page` is the HTML served at `/`.
    pub fn start(&self, pin: String, port: u16, page: &'static str) -> Result<u16, String> {
        let mut slot = self.shared.lock().map_err(|_| "remote state unavailable")?;
        if pin.len() < 4 || !pin.chars().all(|c| c.is_ascii_digit()) {
            return Err("PIN must be at least 4 digits".into());
        }
        if let Some((s, p)) = slot.as_ref() {
            let mut cur = s.pin.lock().map_err(|_| "remote state unavailable")?;
            if *cur != pin {
                *cur = pin;
                // A new PIN signs every connected phone out.
                if let Ok(mut c) = s.clients.lock() {
                    c.clear();
                }
            }
            return Ok(*p);
        }
        let listener = TcpListener::bind(("0.0.0.0", port))
            .or_else(|_| TcpListener::bind(("0.0.0.0", 0)))
            .map_err(|e| format!("Could not open the remote port: {e}"))?;
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();
        listener.set_nonblocking(true).map_err(|e| e.to_string())?;
        let shared = Arc::new(Shared {
            pin: Mutex::new(pin),
            page,
            queue: Mutex::new(VecDeque::new()),
            clients: Mutex::new(Vec::new()),
            state: Mutex::new(String::new()),
            fails: Mutex::new(VecDeque::new()),
            stop: AtomicBool::new(false),
            next_id: AtomicU64::new(1),
        });
        let accept_shared = shared.clone();
        thread::spawn(move || {
            while !accept_shared.stop.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let s = accept_shared.clone();
                        thread::spawn(move || handle(stream, s));
                    }
                    Err(e) if e.kind() == ErrorKind::WouldBlock => thread::sleep(Duration::from_millis(40)),
                    Err(_) => thread::sleep(Duration::from_millis(200)),
                }
            }
        });
        *slot = Some((shared, port));
        Ok(port)
    }

    pub fn stop(&self) {
        if let Ok(mut slot) = self.shared.lock() {
            if let Some((s, _)) = slot.take() {
                s.stop.store(true, Ordering::Relaxed);
                if let Ok(mut c) = s.clients.lock() {
                    c.clear();
                }
            }
        }
    }

    /// (listening port if running, connected phone count)
    pub fn status(&self) -> (Option<u16>, usize) {
        match self.shared.lock() {
            Ok(slot) => match slot.as_ref() {
                Some((s, p)) => (Some(*p), s.clients.lock().map(|c| c.len()).unwrap_or(0)),
                None => (None, 0),
            },
            Err(_) => (None, 0),
        }
    }

    /// Commands sent by phones since the last poll.
    pub fn poll(&self) -> Vec<String> {
        if let Ok(slot) = self.shared.lock() {
            if let Some((s, _)) = slot.as_ref() {
                if let Ok(mut q) = s.queue.lock() {
                    return q.drain(..).collect();
                }
            }
        }
        Vec::new()
    }

    pub fn push_state(&self, json: String) {
        if let Ok(slot) = self.shared.lock() {
            if let Some((s, _)) = slot.as_ref() {
                if let Ok(mut st) = s.state.lock() {
                    *st = json.clone();
                }
                broadcast(s, || Out::Text(format!("state:{json}")));
            }
        }
    }

    /// kind: 1 = Program frame, 2 = Preview frame. `jpeg` is the raw JPEG.
    pub fn push_frame(&self, kind: u8, jpeg: Vec<u8>) {
        if let Ok(slot) = self.shared.lock() {
            if let Some((s, _)) = slot.as_ref() {
                let mut data = Vec::with_capacity(jpeg.len() + 1);
                data.push(kind);
                data.extend_from_slice(&jpeg);
                broadcast(s, || Out::Bin(data.clone()));
            }
        }
    }
}

fn broadcast(s: &Shared, make: impl Fn() -> Out) {
    if let Ok(clients) = s.clients.lock() {
        for (_, tx) in clients.iter() {
            let _ = tx.try_send(make()); // a slow phone just drops frames
        }
    }
}

fn http_reply(stream: &mut TcpStream, status: &str, ctype: &str, body: &[u8]) {
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
    let _ = stream.flush();
}

fn handle(mut stream: TcpStream, s: Arc<Shared>) {
    let _ = stream.set_nodelay(true);
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    // Peek at the request head without consuming it, so tungstenite can still do the handshake.
    let mut buf = [0u8; 4096];
    let started = Instant::now();
    let n = loop {
        match stream.peek(&mut buf) {
            Ok(0) => return,
            Ok(n) => {
                if buf[..n].windows(4).any(|w| w == b"\r\n\r\n") || n == buf.len() {
                    break n;
                }
                if started.elapsed() > Duration::from_secs(5) {
                    return;
                }
                thread::sleep(Duration::from_millis(10));
            }
            Err(_) => return,
        }
    };
    let head = String::from_utf8_lossy(&buf[..n]).to_string();
    let lower = head.to_ascii_lowercase();
    if lower.contains("upgrade: websocket") {
        websocket(stream, s);
        return;
    }
    // Plain HTTP: drain the request, then answer.
    let _ = stream.read(&mut buf);
    let path = head.split_whitespace().nth(1).unwrap_or("/").split('?').next().unwrap_or("/").to_string();
    match path.as_str() {
        "/" | "/index.html" | "/remote" => http_reply(&mut stream, "200 OK", "text/html; charset=utf-8", s.page.as_bytes()),
        "/ping" => http_reply(&mut stream, "200 OK", "text/plain", b"litestream"),
        _ => http_reply(&mut stream, "404 Not Found", "text/plain", b"Not found"),
    }
    let _ = stream.shutdown(Shutdown::Both);
}

fn locked_out(s: &Shared) -> bool {
    let mut f = match s.fails.lock() {
        Ok(f) => f,
        Err(_) => return true,
    };
    while f.front().map(|t| t.elapsed() > FAIL_WINDOW).unwrap_or(false) {
        f.pop_front();
    }
    f.len() >= FAIL_LIMIT
}

fn websocket(stream: TcpStream, s: Arc<Shared>) {
    let mut cfg = WebSocketConfig::default();
    cfg.max_message_size = Some(MAX_MESSAGE);
    cfg.max_frame_size = Some(MAX_MESSAGE);
    let mut ws = match accept_with_config(stream, Some(cfg)) {
        Ok(ws) => ws,
        Err(_) => return,
    };
    let _ = ws.get_ref().set_read_timeout(Some(Duration::from_millis(40)));
    let _ = ws.get_ref().set_write_timeout(Some(Duration::from_secs(3)));

    // 1) authenticate: first text message must be "pin:123456"
    let auth_started = Instant::now();
    loop {
        if s.stop.load(Ordering::Relaxed) || auth_started.elapsed() > Duration::from_secs(8) {
            let _ = ws.close(None);
            return;
        }
        match ws.read() {
            Ok(Message::Text(t)) => {
                if locked_out(&s) {
                    let _ = ws.send(Message::Text("err:locked".into()));
                    let _ = ws.close(None);
                    return;
                }
                let ok = t.strip_prefix("pin:").map(|p| s.pin.lock().map(|pin| constant_eq(p.trim(), &pin)).unwrap_or(false)).unwrap_or(false);
                if ok {
                    break;
                }
                if let Ok(mut f) = s.fails.lock() {
                    f.push_back(Instant::now());
                }
                thread::sleep(Duration::from_millis(600));
                let _ = ws.send(Message::Text("err:badpin".into()));
                let _ = ws.close(None);
                return;
            }
            Ok(Message::Close(_)) | Err(tungstenite::Error::ConnectionClosed) | Err(tungstenite::Error::AlreadyClosed) => return,
            Ok(_) => {}
            Err(tungstenite::Error::Io(e)) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(_) => return,
        }
    }

    // 2) register
    let (tx, rx): (SyncSender<Out>, Receiver<Out>) = sync_channel(4);
    let id = s.next_id.fetch_add(1, Ordering::Relaxed);
    {
        let mut clients = match s.clients.lock() {
            Ok(c) => c,
            Err(_) => return,
        };
        if clients.len() >= MAX_CLIENTS {
            drop(clients);
            let _ = ws.send(Message::Text("err:full".into()));
            let _ = ws.close(None);
            return;
        }
        clients.push((id, tx));
    }
    let _ = ws.send(Message::Text("ok".into()));
    if let Ok(st) = s.state.lock() {
        if !st.is_empty() {
            let _ = ws.send(Message::Text(format!("state:{st}")));
        }
    }

    // 3) serve
    'run: loop {
        if s.stop.load(Ordering::Relaxed) {
            break;
        }
        match ws.read() {
            Ok(Message::Text(t)) => {
                if t == "ping" {
                    if ws.send(Message::Text("pong".into())).is_err() {
                        break;
                    }
                } else if t.starts_with('{') {
                    if let Ok(mut q) = s.queue.lock() {
                        if q.len() >= MAX_QUEUE {
                            q.pop_front();
                        }
                        q.push_back(t.to_string());
                    }
                }
            }
            Ok(Message::Close(_)) | Err(tungstenite::Error::ConnectionClosed) | Err(tungstenite::Error::AlreadyClosed) => break,
            Ok(_) => {}
            Err(tungstenite::Error::Io(e)) if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(_) => break,
        }
        loop {
            match rx.try_recv() {
                Ok(Out::Text(t)) => {
                    if ws.send(Message::Text(t.into())).is_err() {
                        break 'run;
                    }
                }
                Ok(Out::Bin(b)) => {
                    if ws.send(Message::Binary(b.into())).is_err() {
                        break 'run;
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break 'run,
            }
        }
    }
    if let Ok(mut clients) = s.clients.lock() {
        clients.retain(|(cid, _)| *cid != id);
    }
    let _ = ws.close(None);
}
