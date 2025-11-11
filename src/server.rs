use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};


use crate::route::Router;
use crate::types::{Request, Response};

pub struct Server {
    addr: String,
    router: Router,
}

impl Server {
    pub fn new(addr: &str, router: Router) -> Self {
        Server {
            addr: addr.to_string(),
            router,
        }
    }

    pub async fn run(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        println!("Server running on {}", &self.addr);

        loop {
            let (mut socket, _) = listener.accept().await?;
            let router = self.router.clone();

            tokio::spawn(async move {
                if let Some(req) = parse_request(&mut socket).await {
                    let resp = router.handle(req);
                    let resp_bytes = serialize_response(resp);
                    let _ = socket.write_all(&resp_bytes).await;
                }

                
            });
        }
    }
}

async fn parse_request(socket: &mut tokio::net::TcpStream) -> Option<Request> {
    let mut buffer = vec![0u8; 16384];
    let n = socket.read(&mut buffer).await.ok()?;
    if n == 0 {
        return None;
    }

    let req_text = String::from_utf8_lossy(&buffer[..n]);
    let mut lines = req_text.lines();

    let request_line = lines.next()?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next()?.to_string();
    let path = parts.next()?.to_string();
    let version = parts.next()?.to_string();

    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(": ") {
            headers.push((key.to_string(), value.to_string()));
        }
    }

    let mut body = Vec::new();
    if let Some((_, value)) = headers.iter().find(|(k, _)| k.to_lowercase() == "content-length") {
        if let Ok(len) = value.parse::<usize>() {
            let body_len = len.min(buffer.len() - n);
            body.extend_from_slice(&buffer[n..n + body_len]);
        }
    }

    Some(Request {
        method,
        path,
        version,
        headers,
        body,
    })
}

fn serialize_response(resp: Response) -> Vec<u8> {
    let mut response = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\n\r\n",
        resp.status_code,
        resp.reason,
        resp.body.len()
    ).into_bytes();

    response.extend(resp.body);
    response
}

