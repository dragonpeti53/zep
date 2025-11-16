use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use crate::server::{StreamReader, StreamWriter};
use bytes::Bytes;

/// Type alias of `HashMap<String, String>` for convenience.
pub type HeaderMap = HashMap<String, String>;
/// Type alias of `HashMap<String, String>` for convenience.
pub type ParamMap = HashMap<Arc<str>, String>;

/// Enum for quick and memory efficient method handling.
#[derive(Debug, Clone, PartialEq)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    Other(String),
}

impl From<&str> for Method {
    fn from(s: &str) -> Self {
        match s {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            s => Method::Other(s.to_string()),
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Method::GET => write!(f, "GET"),
            Method::POST => write!(f, "POST"),
            Method::PUT => write!(f, "PUT"),
            Method::DELETE => write!(f, "DELETE"),
            Method::Other(_) => write!(f, "OTHER"),
        }
    }
}

impl Method {
    /// Converts a `method` to its text representation in a &str type.
    pub fn to_str(&self) -> &str {
        match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
            Method::Other(_) => "OTHER",
        }
    }
}

/// Enum for quick and memory efficient HTTP version handling.
#[derive(Debug, Clone, PartialEq)]
pub enum Version {
    Http10,
    Http11,
    Http2,
    Http3,
    Other,
}

impl From<&str> for Version {
    fn from(s: &str) -> Self {
        match s {
            "HTTP/1.0" => Version::Http10,
            "HTTP/1.1" => Version::Http11,
            "HTTP/2.0" | "HTTP/2" => Version::Http2,
            "HTTP/3.0" | "HTTP/3" => Version::Http3,
            _ => Version::Other,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Version::Http10 => write!(f, "HTTP/1.0"),
            Version::Http11 => write!(f, "HTTP/1.1"),
            Version::Http2 => write!(f, "HTTP/2.0"),
            Version::Http3 => write!(f, "HTTP/3.0"),
            Version::Other => write!(f, "OTHER"),
        }
    }
}

impl Version {
    /// Converts a `Version` to its text representation in a &str type.
    pub fn to_str(&self) -> &str {
        match self {
            Version::Http10 => "HTTP/1.0",
            Version::Http11 => "HTTP/1.1",
            Version::Http2 => "HTTP/2.0",
            Version::Http3 => "HTTP/3.0",
            Version::Other => "OTHER",
        }
    }
}

/// Enum to conveniently handle status codes.
#[derive(Debug, Clone, PartialEq)]
pub enum StatusCode {
    Ok,
    NotFound,
    InternalServerError,
    BadRequest,
    Forbidden,
    Custom(u16),
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.as_u16();
        let reason = self.reason();
        write!(f, "{} {}", code, reason)
    }
}

impl StatusCode {
    fn as_u16(&self) -> u16 {
        match self {
            StatusCode::Ok => 200,
            StatusCode::NotFound => 404,
            StatusCode::InternalServerError => 500,
            StatusCode::BadRequest => 400,
            StatusCode::Forbidden => 403,
            StatusCode::Custom(c) => *c,
        }
    }

    fn reason(&self) -> &'static str {
        match self {
            StatusCode::Ok => "OK",
            StatusCode::NotFound => "Not Found",
            StatusCode::InternalServerError => "Internal Server Error",
            StatusCode::BadRequest => "Bad Request",
            StatusCode::Forbidden => "Forbidden",
            StatusCode::Custom(_) => "Custom Code",
        }
    }
}

/// Deserialized HTTP request in the form of a struct for easy handling in code.
/// Contains request method, path, version, headers, body, remote_addr(ip address of client), and parameters.
pub struct Request {
    pub method: Method,
    pub path: String,
    pub version: Version,
    pub headers: HeaderMap,
    pub body: Option<Bytes>,
    pub remote_addr: String,
    pub params: ParamMap,
    pub stream: Option<StreamReader>,
}

/// Deserialized HTTP response in the form of a struct for easy handling in code.
/// Contains status_code(status code), headers and body.
pub struct Response {
    pub status_code: StatusCode,
    pub headers: Option<HeaderMap>,
    pub body: Option<Bytes>,
    pub stream: Option<StreamWriter>,
}

impl PartialEq for Response {
    fn eq(&self, other: &Self) -> bool {
        if self.status_code == other.status_code
            && self.headers == other.headers
                && self.body == other.body {
                    return true;
                }
        false
    }
}

impl std::fmt::Debug for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Response")
            .field("status", &self.status_code)
            .field("headers", &self.headers)
            .field("body", &self.body)
            .finish()
    }
}

impl Response {
    /*
    /// Returns a new response.
    /// Requires a StatusCode, HeaderMap and generic body.
    pub fn new<B>(status_code: StatusCode, headers: Option<HeaderMap>, body: Option<B>) -> Self
    where
        B: Into<Bytes>
    {
        Response {
            status_code,
            headers,
            body: if let Some(body) = body {
                Some(body.into())
            } else {
                None
            },
            stream: None,
        }
    }*/

    pub fn new(status_code: StatusCode) -> Self {
        Response {
            status_code,
            headers: None,
            body: None,
            stream: None,
        }
    }

    pub fn body(&mut self, body: impl Into<Bytes>) {
        self.body = Some(body.into())
    }

    /// Helper function to conveniently return a 200 OK response with no headers.
    /// Requires generic body.
    pub fn ok<B>(body: B) -> Self
    where
        B: Into<Bytes>
    {
        Response {
            status_code: StatusCode::Ok,
            headers: None,
            body: Some(body.into()),
            stream: None,
        }
    }

    /// Helper function to return a 404 Not Found response.
    pub fn not_found() -> Self {
        Response {
            status_code: StatusCode::NotFound,
            headers: None,
            body: Some("404 Not Found".into()),
            stream: None,
        }
    }

    /// Helper function to return a 500 Internal Server Error response.
    pub fn error() -> Self {
        Response {
            status_code: StatusCode::InternalServerError,
            headers: None,
            body: None,
            stream: None,
        }
    }

    /// Appends a header to a response's headermap.
    /// Requires a key and value.
    pub fn header(mut self, key: &str, value: &str) -> Self {
        if let Some(ref mut headers) = self.headers {
            headers.insert(key.to_string(), value.to_string());
        } else {
            *&mut self.headers = Some({
                let mut headers = HeaderMap::new();
                headers.insert(key.to_string(), value.to_string());
                headers
            });
        }
        self
    }

    pub fn headermap(mut self, headers: HeaderMap) -> Self {
        self.headers = Some(headers);
        self
    }

    pub fn stream(status_code: StatusCode, stream: StreamWriter) -> Self {
        Response {
            status_code,
            headers: {
                let mut headermap = HeaderMap::new();
                let _ = headermap.insert("Transfer-Encoding".to_string(), "chunked".to_string());
                Some(headermap)
            },
            body: None,
            stream: Some(stream),
        }
        
    }
}

impl Default for Request {
    fn default() -> Self {
        Request {
            method: Method::GET,
            path: "".into(),
            version: Version::Http10,
            headers: HeaderMap::new(),
            body: None,
            remote_addr: "".into(),
            params: ParamMap::new(),
            stream: None,
        }
    }
}
