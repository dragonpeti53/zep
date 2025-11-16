use tokio::io::{AsyncReadExt, AsyncWriteExt, AsyncRead, ReadBuf, BufReader, AsyncBufReadExt};
use tokio::net::TcpListener;
use std::sync::Arc;
use std::pin::Pin;
use bytes::{BytesMut};
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
            let (socket, remote_addr) = match listener.accept().await {
                Ok(value) => value,
                Err(e) => {
                    eprintln!("Error while accepting connection: {}", e);
                    continue;
                }
            };
            let router = self.router.clone();
            let (read, mut write) = socket.into_split();

            tokio::spawn(async move {
                if let Err(e) = async {
                    let req = parse_request(remote_addr, read).await?;

                    let resp = router.handle_request(req).await;
                    let resp_bytes = serialize_response(&resp);
                    write.write_all(&resp_bytes).await?;

                    if let Some(stream) = resp.stream {
                        stream_resp(write, stream).await?;
                    } else {
                        write.shutdown().await?;
                    }

                    Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
                }
                .await {
                    eprintln!("Error, conn: {}, err: {}", remote_addr, e);
                }
            });
        }
    }
}

async fn parse_request(
    remote_addr: std::net::SocketAddr,
    mut reader: tokio::net::tcp::OwnedReadHalf,
) -> Result<Request, ParsingError> {
    let mut buffer = BytesMut::with_capacity(16_384);

    let n = reader.read_buf(&mut buffer).await?;
    if n == 0 { return Err(ParsingError::InvalidRequest("Connection closed while parsing request")) }

    let headers_end = find_headers_end(&buffer)
        .ok_or(ParsingError::InvalidRequest("Invalid headers"))?;
    
    let header_bytes = &buffer[..headers_end];
    let header_str = std::str::from_utf8(header_bytes)?;

    let mut lines = header_str.lines();
    let request_line = lines.next()
        .ok_or(ParsingError::InvalidRequest("Empty request line"))?;

    let mut parts = request_line.split_whitespace();

    let method = Method::from(
        parts.next()
            .ok_or(ParsingError::InvalidRequest("Missing method"))?
    );
    let path = parts.next()
        .ok_or(ParsingError::InvalidRequest("Missing path"))?
        .to_string();
    let version = Version::from(
        parts.next()
            .ok_or(ParsingError::InvalidRequest("Missing version"))?
    );

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
            let mut body = BytesMut::new();
            let body_len = len.min(buffer.len() - n);
            body.extend_from_slice(&buffer[n..n + body_len]);
            Some(body.freeze())
        } else { None }
    };

    

    Ok(Request {
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

fn serialize_response(resp: &Response) -> Vec<u8> {
    let mut response = format!("HTTP/1.1 {}\r\n", resp.status_code).into_bytes();
    if let Some(headers) = &resp.headers {
        for (key, value) in headers {
            response.extend(format!("{}: {}\r\n", key, value).as_bytes());
        }
        response.extend(b"\r\n");
        if let Some(body) = &resp.body {
        if !headers.contains_key("Content-Length") {
                response.extend(format!("Content-Length: {}\r\n", body.len()).as_bytes());
            }
            response.extend(body);
        }
    }
    response
}

fn find_headers_end(buf: &BytesMut) -> Option<usize> {
    memchr::memmem::find(buf, b"\r\n\r\n")
}

async fn stream_resp(mut write: tokio::net::tcp::OwnedWriteHalf, mut stream: StreamWriter)
-> std::io::Result<()> {
    while let Some(chunk) = stream.next_chunk().await {
        if let Err(e) = write.write_all(&chunk).await {
            if e.kind() == std::io::ErrorKind::ConnectionReset
                || e.kind() == std::io::ErrorKind::BrokenPipe 
            {
                return Ok(());
            } else {
                return Err(e);
            }
        }
    }
    let _ = write.shutdown().await;
    Ok(())
}

///used for streamed file reading
pub struct StreamReader {
    leftover: BytesMut,
    pos: usize,
    bufreader: tokio::io::BufReader<tokio::net::tcp::OwnedReadHalf>,
}

impl StreamReader {
    pub(crate) fn new(leftover: BytesMut, reader: tokio::net::tcp::OwnedReadHalf) -> Self {
        StreamReader { leftover, pos: 0, bufreader: BufReader::new(reader) }
    }

    pub async fn next_chunk<B: AsMut<[u8]>>(&mut self) -> std::io::Result<Option<Vec<u8>>> { 
        let mut size_line = String::new();
        let n = self.bufreader.read_line(&mut size_line).await?;
        if n == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "unexpected eof reading chunk size",
            ));
        }

        let size_hex = size_line
            .trim_end_matches(&['\r', '\n'][..])
            .split(';')
            .next()
            .unwrap_or("0")
            .trim();

        let size = usize::from_str_radix(size_hex, 16).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid chunk size: {}", e),
            )
        })?;

        if size == 0 {
            loop {
                let mut trailer = String::new();
                let n = self.bufreader.read_line(&mut trailer).await?;
                if n == 0 || trailer == "\r\n" || trailer.trim().is_empty() {
                    break;
                }
            }
            return Ok(None);
        }

        let mut payload = vec![0u8; size];
        self.bufreader.read_exact(&mut payload).await?;

        let mut crlf = [0u8; 2];
        self.bufreader.read_exact(&mut crlf).await?;
        if &crlf != b"\r\n" {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "missing CRLF after chunk",
            ));
        }

        Ok(Some(payload))
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

        Pin::new(&mut self.bufreader.get_mut()).poll_read(cx, buf)
    }
}

pub struct StreamWriter {
    reader: BufReader<Box<dyn AsyncRead + Unpin + Send>>,
}

impl StreamWriter {
    pub fn new<R>(stream: R) -> Self
    where
        R: AsyncRead + Unpin + 'static + Send,
    {
        Self {
            reader: BufReader::new(Box::new(stream)),
        }
    }

    pub async fn next_chunk(&mut self) -> Option<Vec<u8>> {
        const MAX_CHUNK_SIZE: usize = 64 * 1024;
        let mut buf = vec![0u8; MAX_CHUNK_SIZE];

        match self.reader.read(&mut buf).await {
            Ok(0) => {
                Some(b"0\r\n\r\n".to_vec())
            }
            Ok(n) => {
                buf.truncate(n);
                let mut chunk = Vec::new();

                chunk.extend_from_slice(format!("{:X}\r\n", n).as_bytes());

                chunk.extend_from_slice(&buf);

                chunk.extend_from_slice(b"\r\n");

                Some(chunk)
            }
            Err(_) => None,
        }
    }
}

#[derive(Debug)]
enum ParsingError {
    Io(std::io::Error),
    Utf8(std::str::Utf8Error),
    InvalidRequest(&'static str),
}

impl From<std::io::Error> for ParsingError {
    fn from(e: std::io::Error) -> Self {
        ParsingError::Io(e)
    }
}

impl From<std::str::Utf8Error> for ParsingError {
    fn from(e: std::str::Utf8Error) -> Self {
        ParsingError::Utf8(e)
    }
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingError::Io(e) => write!(f, "IO error: {}", e),
            ParsingError::Utf8(e) => write!(f, "UTF8 error: {}", e),
            ParsingError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
        }
    }
}

impl std::error::Error for ParsingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParsingError::Io(e) => Some(e),
            ParsingError::Utf8(e) => Some(e),
            ParsingError::InvalidRequest(_) => None,
        }
    }
}