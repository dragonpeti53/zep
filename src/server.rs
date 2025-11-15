use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncRead, ReadBuf};
use tokio::net::TcpListener;
use std::sync::Arc;
use std::pin::Pin;
use bytes::BytesMut;
use std::task::{Context, Poll};
use crate::route::Router;
use crate::types::{HeaderMap, Method, ParamMap, Request, Response, Version};

/// Server that wraps the whole HTTP server in itself.
pub struct Server {
    addr: &'static str,
    router: Arc<Router>,
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
    pub fn new(addr: &'static str, router: Router) -> Self {
        Server { addr, router: Arc::from(router) }
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
            let (socket, remote_addr) = listener.accept().await?;
            let router = self.router.clone();
            let (read, mut write) = socket.into_split();

            tokio::spawn(async move {
                if let Some(req) = parse_request(remote_addr, read).await {
                    let resp = router.handle_request(req).await;
                    let resp_bytes = serialize_response(resp);
                    let _ = write.write_all(&resp_bytes).await;
                }
            });
        }
    }
}

async fn parse_request(
    remote_addr: std::net::SocketAddr,
    mut reader: tokio::net::tcp::OwnedReadHalf,
) -> Option<Request> {
    let mut buffer = BytesMut::with_capacity(16384);
    let n = reader.read_buf(&mut buffer).await.ok()?;
    if n == 0 { return None; }

    let headers_end = find_headers_end(&buffer)?;
    let header_bytes = &buffer[..headers_end];

    let header_str = std::str::from_utf8(header_bytes).ok()?;
    let mut lines = header_str.lines();

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

    let remote_addr = remote_addr.to_string();

    let params = ParamMap::new();

    let leftover = buffer.split_off(headers_end + 4);

    let is_chunked = headers.iter().any(|(k, v)| {
        k.eq_ignore_ascii_case("transfer-encoding") &&
        v.split(',').any(|s| s.trim().eq_ignore_ascii_case("chunked"))
    });

    /*let body = if !is_chunked {
        if let Some((_, value)) =
            headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
        {
            if let Ok(len) = value.parse::<usize>() {
                let mut body = Vec::with_capacity(len);
                let already = leftover.len();
                let use_from_leftover = std::cmp::min(len, already);
                if use_from_leftover > 0 {
                    body.extend_from_slice(&leftover[..use_from_leftover]);
                }
                let to_read = len.saturating_sub(body.len());
                if to_read > 0 {
                    let mut tmp = vec![0u8; to_read];
                    // remaining bytes come from the reader
                    reader.read_exact(&mut tmp).await.ok()?;
                    body.extend_from_slice(&tmp);
                }
                Some(body)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };*/

    let stream = if is_chunked {
        Some(StreamReader::new(leftover, reader))
    } else {
        None
    };

    let body = {
        if let Some((_, value)) = headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == "content-length")
            && let Ok(len) = value.parse::<usize>()
        {
            let mut body = Vec::new();
            let body_len = len.min(buffer.len() - n);
            body.extend_from_slice(&buffer[n..n + body_len]);
            Some(body)
        } else { None }
    };

    

    Some(Request {
        method,
        path,
        version,
        headers,
        body,
        remote_addr,
        params,
        stream,
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

fn find_headers_end(buf: &BytesMut) -> Option<usize> {
    memchr::memmem::find(buf, b"\r\n\r\n")
}

///used for streamed file reading
pub struct StreamReader {
    leftover: BytesMut,
    pos: usize,
    reader: tokio::net::tcp::OwnedReadHalf,
}

impl StreamReader {
    pub(crate) fn new(leftover: BytesMut, reader: tokio::net::tcp::OwnedReadHalf) -> Self {
        StreamReader { leftover, pos: 0, reader }
    }
}

impl AsyncRead for StreamReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        if self.pos < self.leftover.len() {
            let rem = &self.leftover[self.pos..];
            let take = std::cmp::min(rem.len(), buf.remaining());
            buf.put_slice(&rem[..take]);
            self.pos = take;
            return Poll::Ready(Ok(()));
        }

        Pin::new(&mut self.reader).poll_read(cx, buf)
    }
}