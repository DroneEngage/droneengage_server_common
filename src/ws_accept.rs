//! Accepting WebSocket connections the way Node's http server and ws do:
//! the request head read by hand (so a plain request can be answered or
//! left waiting like Node does), ws's handshake validation and error
//! responses, and `listen(port, host)` address resolution.

use std::net::SocketAddr;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::handshake::derive_accept_key;

const MAX_HEADER_BYTES: usize = 16 * 1024;

pub struct RequestHead {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
}

impl RequestHead {
    /// A header value; repeated headers are joined with ", " (Node).
    pub fn header(&self, p_name: &str) -> Option<String> {
        let v: Vec<&str> =
            self.headers.iter().filter(|(k, _)| k.eq_ignore_ascii_case(p_name)).map(|(_, v)| v.as_str()).collect();
        (!v.is_empty()).then(|| v.join(", "))
    }
}

/// Reads a request head; returns it and any bytes read past it. Err(0) is
/// a closed connection, else the HTTP error code to answer.
pub async fn read_head<S: AsyncRead + Unpin>(p_stream: &mut S) -> Result<(RequestHead, Vec<u8>), u16> {
    let mut v_buf: Vec<u8> = Vec::with_capacity(1024);
    let mut v_chunk = [0u8; 4096];
    loop {
        let n = p_stream.read(&mut v_chunk).await.map_err(|_| 0u16)?;
        if n == 0 {
            return Err(0);
        }
        v_buf.extend_from_slice(&v_chunk[..n]);
        let mut v_headers = [httparse::EMPTY_HEADER; 100];
        let mut v_req = httparse::Request::new(&mut v_headers);
        match v_req.parse(&v_buf) {
            Ok(httparse::Status::Complete(v_len)) => {
                let v_head = RequestHead {
                    method: v_req.method.unwrap_or("").to_string(),
                    url: v_req.path.unwrap_or("/").to_string(),
                    headers: v_req
                        .headers
                        .iter()
                        .map(|h| (h.name.to_string(), String::from_utf8_lossy(h.value).into_owned()))
                        .collect(),
                };
                return Ok((v_head, v_buf[v_len..].to_vec()));
            }
            Ok(httparse::Status::Partial) => {
                if v_buf.len() > MAX_HEADER_BYTES {
                    return Err(431);
                }
            }
            Err(httparse::Error::TooManyHeaders) => return Err(431),
            Err(_) => return Err(400),
        }
    }
}

/// ws's abortHandshake response.
pub fn http_error(p_code: u16, p_extra: &str) -> Vec<u8> {
    let v_reason = match p_code {
        400 => "Bad Request",
        405 => "Method Not Allowed",
        431 => "Request Header Fields Too Large",
        _ => "Bad Request",
    };
    format!(
        "HTTP/1.1 {} {}\r\nConnection: close\r\nContent-Type: text/html\r\nContent-Length: {}\r\n{}\r\n{}",
        p_code,
        v_reason,
        v_reason.len(),
        p_extra,
        v_reason
    )
    .into_bytes()
}

/// Validates an upgrade like ws's handleUpgrade; the 101 response or the
/// error response.
pub fn upgrade_response(p_head: &RequestHead) -> Result<Vec<u8>, Vec<u8>> {
    if p_head.method != "GET" {
        return Err(http_error(405, ""));
    }
    if !p_head.header("upgrade").map(|u| u.trim().eq_ignore_ascii_case("websocket")).unwrap_or(false) {
        return Err(http_error(400, ""));
    }
    let v_key = p_head.header("sec-websocket-key").unwrap_or_default();
    let v_key_ok = v_key.len() == 24
        && v_key.ends_with("==")
        && v_key[..22].bytes().all(|b| b.is_ascii_alphanumeric() || b == b'+' || b == b'/');
    if !v_key_ok {
        return Err(http_error(400, ""));
    }
    let v_version = p_head.header("sec-websocket-version").unwrap_or_default();
    if v_version.trim() != "13" && v_version.trim() != "8" {
        return Err(http_error(400, "Sec-WebSocket-Version: 13, 8\r\n"));
    }
    let mut v_resp = format!(
        "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n",
        derive_accept_key(v_key.as_bytes())
    );
    if let Some(v_protocols) = p_head.header("sec-websocket-protocol") {
        if let Some(v_first) = v_protocols.split(',').map(str::trim).find(|p| !p.is_empty()) {
            v_resp.push_str(&format!("Sec-WebSocket-Protocol: {}\r\n", v_first));
        }
    }
    v_resp.push_str("\r\n");
    Ok(v_resp.into_bytes())
}

/// req.socket.remoteAddress as Node prints it.
pub fn remote_address(p_addr: &SocketAddr) -> String {
    p_addr.ip().to_string()
}

/// listen(port, host) like Node: a missing host is all interfaces.
pub async fn bind_listener(p_ip: &str, p_port: &str) -> Result<TcpListener, String> {
    let v_port: u16 = if p_port == "undefined" || p_port.is_empty() {
        0
    } else {
        p_port.trim().parse().map_err(|_| format!("options.port should be >= 0 and < 65536. Received {}.", p_port))?
    };
    let v_any = p_ip == "undefined" || p_ip.is_empty();
    let v_host = if v_any { "::" } else { p_ip };
    let v_host = v_host.trim_start_matches('[').trim_end_matches(']');
    let v_addrs: Vec<SocketAddr> =
        tokio::net::lookup_host((v_host, v_port)).await.map_err(|e| e.to_string())?.collect();
    let mut v_last = String::from("no address");
    for v_addr in v_addrs {
        match TcpListener::bind(v_addr).await {
            Ok(l) => return Ok(l),
            Err(e) => {
                v_last = if e.kind() == std::io::ErrorKind::AddrInUse {
                    format!("listen EADDRINUSE: address already in use {}", v_addr)
                } else {
                    format!("listen {}: {}", v_addr, e)
                }
            }
        }
    }
    // like Node: no IPv6 on this host, so all IPv4 interfaces
    if v_any && v_last.contains("os error 97") {
        if let Ok(l) = TcpListener::bind(("0.0.0.0", v_port)).await {
            return Ok(l);
        }
    }
    Err(v_last)
}
