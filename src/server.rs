use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::route::Router;
use crate::types::{Request, Response, Method, Version, HeaderMap, ParamMap};

/// Server that wraps the whole HTTP server in itself.
pub struct Server {
    addr: String,
    router: Router,
}

impl Server {
    /// Returns a new Server struct.
    /// Requires an address and router.
    /// 
    /// # Example:
    /// ```
    /// use zep::{Router, Server};
    /// 
    /// let mut router = Router::new();
    /// let server = Server::new("0.0.0.0:8080", router);
    /// ```
    pub fn new(addr: &str, router: Router) -> Self {
        Server {
            addr: addr.to_string(),
            router,
        }
    }

    /// Starts listening and handling requests on the address we defined in new().
    /// Returns an std::io::Result enum if there was an error.
    /// 
    /// # Example:
    /// ```no_run
    /// use zep::{tokio, Router, Server};
    /// 
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut router = Router::new();
    ///     let server = Server::new("0.0.0.0:8080", router);
    ///     let _ = server.run().await;
    /// }
    /// ```
    pub async fn run(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        println!("Server running on {}", &self.addr);

        loop {
            let (mut socket, remote_addr) = listener.accept().await?;
            let router = self.router.clone();

            tokio::spawn(async move {
                if let Some(req) = parse_request(&mut socket, remote_addr).await {
                    let resp = router.handle(req).await;
                    let resp_bytes = serialize_response(resp);
                    let _ = socket.write_all(&resp_bytes).await;
                }

                
            });
        }
    }
}

async fn parse_request(socket: &mut tokio::net::TcpStream, remote_addr: std::net::SocketAddr) -> Option<Request> {
    let mut buffer = vec![0u8; 16384];
    let n = socket.read(&mut buffer).await.ok()?;
    if n == 0 {
        return None;
    }

    let req_text = String::from_utf8_lossy(&buffer[..n]);
    let mut lines = req_text.lines();

    let request_line = lines.next()?;
    let mut parts = request_line.split_whitespace();
    let method = Method::from(parts.next()?); 
    let path = parts.next()?.to_string();
    let version = Version::from(parts.next()?);

    let mut headers = HeaderMap::new();
    for line in lines {
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(": ") {
            headers.insert(key.to_string(), value.to_string());
        }
    }

    let mut body = Vec::new();
    if let Some((_, value)) = headers.iter().find(|(k, _)| k.to_lowercase() == "content-length")
        && let Ok(len) = value.parse::<usize>() {
            let body_len = len.min(buffer.len() - n);
            body.extend_from_slice(&buffer[n..n + body_len]);
        }

    let remote_addr = remote_addr.to_string();

    let params = ParamMap::new();

    Some(Request {
        method,
        path,
        version,
        headers,
        body,
        remote_addr,
        params,
    })
}

fn serialize_response(resp: Response) -> Vec<u8> {
    let mut response = format!("HTTP/1.1 {}\r\n", resp.status_code).into_bytes();
    for (key, value) in &resp.headers {
        response.extend(format!("{}: {}\r\n", key, value).as_bytes());
    }
    if !resp.headers.contains_key("Content-Length") {
        response.extend(format!("Content-Length: {}\r\n", resp.body.len()).as_bytes());
    }
    response.extend(b"\r\n");
    response.extend(&resp.body);
    response
}

